/// MQTT client module for Home Assistant communication.
///
/// Handles connection to an MQTT broker, publishing sensor data,
/// and camera images for Home Assistant consumption.
use anyhow::Result;
use embedded_svc::mqtt::client::{EventPayload, QoS};
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config;

pub struct MqttManager {
    client: EspMqttClient<'static>,
    connected: Arc<Mutex<bool>>,
}

impl MqttManager {
    pub fn new() -> Result<Self> {
        let broker_url = if config::MQTT_USER.is_empty() {
            format!("mqtt://{}:{}", config::MQTT_HOST, config::MQTT_PORT)
        } else {
            format!(
                "mqtt://{}:{}@{}:{}",
                config::MQTT_USER,
                config::MQTT_PASS,
                config::MQTT_HOST,
                config::MQTT_PORT
            )
        };

        info!("Connecting to MQTT broker: {}", config::MQTT_HOST);

        let lwt_topic = format!(
            "{}/{}/availability",
            config::MQTT_TOPIC_BASE,
            config::DEVICE_ID
        );

        let mqtt_config = MqttClientConfiguration {
            client_id: Some(config::DEVICE_ID),
            keep_alive_interval: Some(Duration::from_secs(30)),
            lwt: Some(esp_idf_svc::mqtt::client::LwtConfiguration {
                topic: &lwt_topic,
                payload: b"offline",
                qos: QoS::AtLeastOnce,
                retain: true,
            }),
            ..Default::default()
        };

        let connected = Arc::new(Mutex::new(false));
        let connected_clone = connected.clone();

        let client = EspMqttClient::new_cb(&broker_url, &mqtt_config, move |event| {
            match event.payload() {
                EventPayload::Connected(_) => {
                    info!("MQTT connected");
                    if let Ok(mut c) = connected_clone.lock() {
                        *c = true;
                    }
                }
                EventPayload::Disconnected => {
                    warn!("MQTT disconnected");
                    if let Ok(mut c) = connected_clone.lock() {
                        *c = false;
                    }
                }
                EventPayload::Error(e) => {
                    error!("MQTT error: {:?}", e);
                }
                _ => {}
            }
        })?;

        info!("MQTT client created");

        Ok(Self { client, connected })
    }

    pub fn is_connected(&self) -> bool {
        self.connected.lock().map(|c| *c).unwrap_or(false)
    }

    /// Publish a message to an MQTT topic.
    pub fn publish(&mut self, topic: &str, payload: &[u8], qos: QoS, retain: bool) -> Result<()> {
        self.client
            .enqueue(topic, qos, retain, payload)
            .map_err(|e| anyhow::anyhow!("MQTT publish failed: {:?}", e))?;
        Ok(())
    }

    /// Publish the device availability status.
    pub fn publish_availability(&mut self, online: bool) -> Result<()> {
        let topic = format!(
            "{}/{}/availability",
            config::MQTT_TOPIC_BASE,
            config::DEVICE_ID
        );
        let payload = if online { "online" } else { "offline" };
        self.publish(&topic, payload.as_bytes(), QoS::AtLeastOnce, true)
    }

    /// Publish bird detection state.
    pub fn publish_bird_detected(&mut self, detected: bool) -> Result<()> {
        let topic = format!(
            "{}/{}/bird_detected",
            config::MQTT_TOPIC_BASE,
            config::DEVICE_ID
        );
        let payload = if detected { "ON" } else { "OFF" };
        self.publish(&topic, payload.as_bytes(), QoS::AtLeastOnce, false)
    }

    /// Publish bird species classification result.
    pub fn publish_species(&mut self, species: &str, confidence: f32) -> Result<()> {
        let topic = format!("{}/{}/species", config::MQTT_TOPIC_BASE, config::DEVICE_ID);

        let payload = serde_json::json!({
            "species": species,
            "confidence": confidence
        });

        self.publish(
            &topic,
            payload.to_string().as_bytes(),
            QoS::AtLeastOnce,
            false,
        )
    }

    /// Publish a JPEG camera image for the HA camera entity.
    pub fn publish_image(&mut self, jpeg_data: &[u8]) -> Result<()> {
        let topic = format!("{}/{}/camera", config::MQTT_TOPIC_BASE, config::DEVICE_ID);
        self.publish(&topic, jpeg_data, QoS::AtMostOnce, false)
    }

    /// Publish motion detection score as a sensor value.
    pub fn publish_motion_score(&mut self, score: f32) -> Result<()> {
        let topic = format!(
            "{}/{}/motion_score",
            config::MQTT_TOPIC_BASE,
            config::DEVICE_ID
        );
        let payload = format!("{:.1}", score);
        self.publish(&topic, payload.as_bytes(), QoS::AtMostOnce, false)
    }

    /// Publish device attributes (uptime, free heap, WiFi RSSI, etc.).
    pub fn publish_attributes(&mut self, attributes: &serde_json::Value) -> Result<()> {
        let topic = format!(
            "{}/{}/attributes",
            config::MQTT_TOPIC_BASE,
            config::DEVICE_ID
        );
        self.publish(
            &topic,
            attributes.to_string().as_bytes(),
            QoS::AtMostOnce,
            true,
        )
    }
}
