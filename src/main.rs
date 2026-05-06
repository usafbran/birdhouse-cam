/// Birdhouse Camera - ESP32-CAM Bird Detection for Home Assistant
///
/// This application runs on an ESP32-CAM module inside a birdhouse.
/// It captures images, detects bird activity via motion detection,
/// optionally classifies bird species using an external server, and
/// sends alerts to Home Assistant via MQTT.
///
/// Architecture:
///   Camera Capture -> Motion Detection -> [Optional Classification] -> MQTT -> Home Assistant
///
/// Home Assistant entities (auto-discovered via MQTT):
///   - binary_sensor.bird_detected   : ON when a bird is detected
///   - sensor.bird_species           : Last classified bird species
///   - sensor.motion_score           : Current motion detection score
///   - camera.birdhouse_camera       : Latest snapshot from the camera
///   - sensor.device_info            : Diagnostic info (uptime, heap, RSSI)

mod camera;
mod config;
mod detection;
mod homeassistant;
mod mqtt;
mod wifi;

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
};
use log::{error, info, warn};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    // Bind the ESP-IDF patches and logger
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("========================================");
    info!("  Birdhouse Camera v{}", env!("CARGO_PKG_VERSION"));
    info!("  Device: {}", config::DEVICE_ID);
    info!("========================================");

    // Initialize system peripherals
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Step 1: Connect to WiFi
    info!("--- Connecting to WiFi ---");
    let mut wifi = wifi::WifiConnection::new(peripherals.modem, sys_loop, nvs)?;
    wifi.connect()?;
    info!("WiFi connected successfully");

    // Step 2: Initialize camera
    info!("--- Initializing Camera ---");
    let cam = camera::Camera::new()?;
    info!("Camera ready");

    // Step 3: Connect to MQTT broker
    info!("--- Connecting to MQTT ---");
    let mut mqtt_manager = mqtt::MqttManager::new()?;

    // Wait briefly for MQTT connection to establish
    std::thread::sleep(Duration::from_secs(2));

    // Step 4: Publish Home Assistant discovery
    info!("--- Publishing HA Discovery ---");
    homeassistant::publish_discovery(&mut mqtt_manager)?;

    // Step 5: Announce we're online
    mqtt_manager.publish_availability(true)?;

    // Step 6: Initialize detection
    let mut motion_detector = detection::MotionDetector::new();
    let classifier = detection::RemoteClassifier::new();

    // Main loop
    info!("--- Entering main detection loop ---");
    let start_time = Instant::now();
    let mut last_capture = Instant::now();
    let mut last_alert = Instant::now() - Duration::from_millis(config::COOLDOWN_PERIOD_MS);
    let mut last_diagnostics = Instant::now();
    let capture_interval = Duration::from_millis(config::CAPTURE_INTERVAL_MS);
    let cooldown = Duration::from_millis(config::COOLDOWN_PERIOD_MS);
    let diagnostics_interval = Duration::from_secs(60);

    loop {
        // Check WiFi and reconnect if needed
        if !wifi.is_connected() {
            warn!("WiFi connection lost");
            if let Err(e) = wifi.reconnect() {
                error!("WiFi reconnection failed: {:?}", e);
                std::thread::sleep(Duration::from_secs(10));
                continue;
            }
        }

        // Capture and analyze at the configured interval
        if last_capture.elapsed() >= capture_interval {
            last_capture = Instant::now();

            match cam.capture_jpeg() {
                Ok(frame) => {
                    let frame_data = frame.data().to_vec();
                    let result = motion_detector.detect(&frame_data);

                    // Publish motion score
                    if let Err(e) = mqtt_manager.publish_motion_score(result.motion_score) {
                        warn!("Failed to publish motion score: {:?}", e);
                    }

                    if result.motion_detected {
                        info!(
                            "Bird activity detected! Motion score: {:.1}%",
                            result.motion_score
                        );

                        // Respect cooldown period to avoid alert flooding
                        if last_alert.elapsed() >= cooldown {
                            last_alert = Instant::now();

                            // Publish bird detected
                            if let Err(e) = mqtt_manager.publish_bird_detected(true) {
                                error!("Failed to publish bird detection: {:?}", e);
                            }

                            // Publish the camera image
                            if let Err(e) = mqtt_manager.publish_image(&frame_data) {
                                warn!("Failed to publish camera image: {:?}", e);
                            }

                            // Attempt species classification if server is configured
                            if let Some(ref clf) = classifier {
                                match clf.classify(&frame_data) {
                                    Ok((species, confidence)) => {
                                        info!(
                                            "Bird classified: {} ({:.0}%)",
                                            species,
                                            confidence * 100.0
                                        );
                                        if let Err(e) =
                                            mqtt_manager.publish_species(&species, confidence)
                                        {
                                            warn!("Failed to publish species: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Classification failed: {:?}", e);
                                        if let Err(e) = mqtt_manager
                                            .publish_species("unknown (detection only)", 0.0)
                                        {
                                            warn!("Failed to publish species: {:?}", e);
                                        }
                                    }
                                }
                            } else {
                                // No classifier configured, just report detection
                                if let Err(e) =
                                    mqtt_manager.publish_species("detected (no classifier)", 0.0)
                                {
                                    warn!("Failed to publish species: {:?}", e);
                                }
                            }
                        }
                    } else {
                        // No motion - publish bird not detected
                        if let Err(e) = mqtt_manager.publish_bird_detected(false) {
                            warn!("Failed to publish bird state: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Frame capture failed: {:?}", e);
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }

        // Publish diagnostics periodically
        if last_diagnostics.elapsed() >= diagnostics_interval {
            last_diagnostics = Instant::now();

            let uptime = start_time.elapsed().as_secs();
            let free_heap = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };

            let attrs = serde_json::json!({
                "uptime_seconds": uptime,
                "free_heap_bytes": free_heap,
                "device_id": config::DEVICE_ID,
                "firmware_version": env!("CARGO_PKG_VERSION"),
                "capture_interval_ms": config::CAPTURE_INTERVAL_MS,
                "motion_threshold": config::MOTION_PIXEL_PERCENT
            });

            if let Err(e) = mqtt_manager.publish_attributes(&attrs) {
                warn!("Failed to publish diagnostics: {:?}", e);
            }
        }

        // Small sleep to yield to the RTOS scheduler
        std::thread::sleep(Duration::from_millis(100));
    }
}
