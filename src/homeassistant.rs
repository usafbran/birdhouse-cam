/// Home Assistant auto-discovery module.
///
/// Publishes MQTT discovery messages so the device and its entities
/// automatically appear in Home Assistant without manual configuration.
///
/// Entities registered:
/// - Binary sensor: bird_detected (occupancy class)
/// - Sensor: bird_species (text)
/// - Sensor: motion_score (percentage)
/// - Camera: birdhouse snapshot
/// - Sensor: device_attributes (diagnostics)
use anyhow::Result;
use embedded_svc::mqtt::client::QoS;
use log::info;

use crate::config;
use crate::mqtt::MqttManager;

/// Device information for HA discovery payloads.
fn device_payload() -> serde_json::Value {
    serde_json::json!({
        "identifiers": [config::DEVICE_ID],
        "name": config::DEVICE_NAME,
        "model": "ESP32-CAM Birdhouse",
        "manufacturer": "DIY",
        "sw_version": env!("CARGO_PKG_VERSION"),
        "configuration_url": format!("http://{}.local", config::DEVICE_ID)
    })
}

fn availability_topic() -> String {
    format!(
        "{}/{}/availability",
        config::MQTT_TOPIC_BASE,
        config::DEVICE_ID
    )
}

/// Publish all Home Assistant MQTT discovery messages.
pub fn publish_discovery(mqtt: &mut MqttManager) -> Result<()> {
    info!("Publishing Home Assistant MQTT discovery messages...");

    publish_bird_detected_discovery(mqtt)?;
    publish_species_discovery(mqtt)?;
    publish_motion_score_discovery(mqtt)?;
    publish_camera_discovery(mqtt)?;
    publish_attributes_discovery(mqtt)?;

    info!("Home Assistant discovery messages published");
    Ok(())
}

/// Binary sensor: bird detected (ON/OFF).
fn publish_bird_detected_discovery(mqtt: &mut MqttManager) -> Result<()> {
    let unique_id = format!("{}_bird_detected", config::DEVICE_ID);
    let payload = serde_json::json!({
        "name": "Bird Detected",
        "unique_id": unique_id,
        "device_class": "occupancy",
        "state_topic": format!("{}/{}/bird_detected", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "payload_on": "ON",
        "payload_off": "OFF",
        "availability_topic": availability_topic(),
        "device": device_payload(),
        "icon": "mdi:bird"
    });

    let topic = format!(
        "{}/binary_sensor/{}/bird_detected/config",
        config::HA_DISCOVERY_PREFIX,
        config::DEVICE_ID
    );

    mqtt.publish(
        &topic,
        payload.to_string().as_bytes(),
        QoS::AtLeastOnce,
        true,
    )?;
    info!("Published discovery: binary_sensor/bird_detected");
    Ok(())
}

/// Sensor: bird species classification result.
fn publish_species_discovery(mqtt: &mut MqttManager) -> Result<()> {
    let unique_id = format!("{}_species", config::DEVICE_ID);
    let payload = serde_json::json!({
        "name": "Bird Species",
        "unique_id": unique_id,
        "state_topic": format!("{}/{}/species", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "value_template": "{{ value_json.species }}",
        "json_attributes_topic": format!("{}/{}/species", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "availability_topic": availability_topic(),
        "device": device_payload(),
        "icon": "mdi:bird"
    });

    let topic = format!(
        "{}/sensor/{}/species/config",
        config::HA_DISCOVERY_PREFIX,
        config::DEVICE_ID
    );

    mqtt.publish(
        &topic,
        payload.to_string().as_bytes(),
        QoS::AtLeastOnce,
        true,
    )?;
    info!("Published discovery: sensor/species");
    Ok(())
}

/// Sensor: motion detection score (0-100%).
fn publish_motion_score_discovery(mqtt: &mut MqttManager) -> Result<()> {
    let unique_id = format!("{}_motion_score", config::DEVICE_ID);
    let payload = serde_json::json!({
        "name": "Motion Score",
        "unique_id": unique_id,
        "state_topic": format!("{}/{}/motion_score", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "unit_of_measurement": "%",
        "state_class": "measurement",
        "availability_topic": availability_topic(),
        "device": device_payload(),
        "icon": "mdi:motion-sensor"
    });

    let topic = format!(
        "{}/sensor/{}/motion_score/config",
        config::HA_DISCOVERY_PREFIX,
        config::DEVICE_ID
    );

    mqtt.publish(
        &topic,
        payload.to_string().as_bytes(),
        QoS::AtLeastOnce,
        true,
    )?;
    info!("Published discovery: sensor/motion_score");
    Ok(())
}

/// Camera entity: latest birdhouse snapshot.
fn publish_camera_discovery(mqtt: &mut MqttManager) -> Result<()> {
    let unique_id = format!("{}_camera", config::DEVICE_ID);
    let payload = serde_json::json!({
        "name": "Birdhouse Camera",
        "unique_id": unique_id,
        "topic": format!("{}/{}/camera", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "availability_topic": availability_topic(),
        "device": device_payload(),
        "icon": "mdi:camera-outline"
    });

    let topic = format!(
        "{}/camera/{}/snapshot/config",
        config::HA_DISCOVERY_PREFIX,
        config::DEVICE_ID
    );

    mqtt.publish(
        &topic,
        payload.to_string().as_bytes(),
        QoS::AtLeastOnce,
        true,
    )?;
    info!("Published discovery: camera/snapshot");
    Ok(())
}

/// Diagnostic sensor: device attributes (uptime, heap, RSSI).
fn publish_attributes_discovery(mqtt: &mut MqttManager) -> Result<()> {
    let unique_id = format!("{}_diagnostics", config::DEVICE_ID);
    let payload = serde_json::json!({
        "name": "Device Info",
        "unique_id": unique_id,
        "state_topic": format!("{}/{}/attributes", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "value_template": "{{ value_json.uptime_seconds }}",
        "unit_of_measurement": "s",
        "json_attributes_topic": format!("{}/{}/attributes", config::MQTT_TOPIC_BASE, config::DEVICE_ID),
        "entity_category": "diagnostic",
        "availability_topic": availability_topic(),
        "device": device_payload(),
        "icon": "mdi:information-outline"
    });

    let topic = format!(
        "{}/sensor/{}/diagnostics/config",
        config::HA_DISCOVERY_PREFIX,
        config::DEVICE_ID
    );

    mqtt.publish(
        &topic,
        payload.to_string().as_bytes(),
        QoS::AtLeastOnce,
        true,
    )?;
    info!("Published discovery: sensor/diagnostics");
    Ok(())
}
