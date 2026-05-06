/// WiFi connectivity module for the Birdhouse Camera.
///
/// Handles connecting to a WiFi network in station mode with
/// automatic reconnection support.
use anyhow::{bail, Result};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use log::{info, warn};

use crate::config;

pub struct WifiConnection {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl WifiConnection {
    pub fn new(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self> {
        let wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

        Ok(Self { wifi })
    }

    pub fn connect(&mut self) -> Result<()> {
        let ssid = config::WIFI_SSID;
        let password = config::WIFI_PASS;

        if ssid == "changeme" {
            bail!("WiFi SSID not configured! Set WIFI_SSID environment variable.");
        }

        info!("Configuring WiFi with SSID: {}", ssid);

        let auth_method = if password.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        };

        self.wifi
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: ssid
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("SSID too long"))?,
                password: password
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Password too long"))?,
                auth_method,
                ..Default::default()
            }))?;

        self.wifi.start()?;
        info!("WiFi started, scanning for networks...");

        self.wifi.connect()?;
        info!("WiFi connected to {}", ssid);

        self.wifi.wait_netif_up()?;

        let ip_info = self.wifi.wifi().sta_netif().get_ip_info()?;
        info!("WiFi DHCP info: {:?}", ip_info);
        info!("IP address: {}", ip_info.ip);

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.wifi.is_connected().unwrap_or(false)
    }

    pub fn reconnect(&mut self) -> Result<()> {
        warn!("WiFi disconnected, attempting reconnection...");

        for attempt in 1..=5 {
            info!("Reconnection attempt {}/5", attempt);
            match self.wifi.connect() {
                Ok(_) => {
                    self.wifi.wait_netif_up()?;
                    info!("WiFi reconnected successfully");
                    return Ok(());
                }
                Err(e) => {
                    warn!("Reconnection attempt {} failed: {:?}", attempt, e);
                    std::thread::sleep(std::time::Duration::from_secs(2_u64.pow(attempt)));
                }
            }
        }

        bail!("Failed to reconnect to WiFi after 5 attempts")
    }
}
