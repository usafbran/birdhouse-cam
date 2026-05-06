/// Application configuration for the Birdhouse Camera.
///
/// Configuration values are set at compile time via environment variables
/// or can be stored in NVS (Non-Volatile Storage) for runtime updates.

/// WiFi credentials
pub const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
    Some(v) => v,
    None => "changeme",
};
pub const WIFI_PASS: &str = match option_env!("WIFI_PASS") {
    Some(v) => v,
    None => "changeme",
};

/// MQTT broker configuration
pub const MQTT_HOST: &str = match option_env!("MQTT_HOST") {
    Some(v) => v,
    None => "homeassistant.local",
};
pub const MQTT_PORT: u16 = 1883;
pub const MQTT_USER: &str = match option_env!("MQTT_USER") {
    Some(v) => v,
    None => "",
};
pub const MQTT_PASS: &str = match option_env!("MQTT_PASS") {
    Some(v) => v,
    None => "",
};

/// Device identification
pub const DEVICE_NAME: &str = match option_env!("DEVICE_NAME") {
    Some(v) => v,
    None => "birdhouse_cam",
};
pub const DEVICE_ID: &str = match option_env!("DEVICE_ID") {
    Some(v) => v,
    None => "birdhouse_cam_01",
};

/// Home Assistant MQTT discovery prefix
pub const HA_DISCOVERY_PREFIX: &str = "homeassistant";

/// MQTT topic base for this device
pub const MQTT_TOPIC_BASE: &str = "birdhouse";

/// Camera settings
pub const CAMERA_FRAME_SIZE: u32 = 10; // FRAMESIZE_UXGA = 13, SVGA = 9, VGA = 8, QVGA = 5
pub const CAMERA_JPEG_QUALITY: u32 = 12; // 0-63, lower = better quality
pub const CAMERA_FB_COUNT: u32 = 2;

/// Detection settings
pub const MOTION_THRESHOLD: u32 = 30; // Pixel difference threshold for motion
pub const MOTION_PIXEL_PERCENT: f32 = 5.0; // Percentage of changed pixels to trigger
pub const CAPTURE_INTERVAL_MS: u64 = 2000; // Interval between captures
pub const COOLDOWN_PERIOD_MS: u64 = 30_000; // Minimum time between alerts

/// Classification server (optional, for offloading bird species identification)
pub const CLASSIFICATION_SERVER_URL: &str = match option_env!("CLASSIFICATION_SERVER") {
    Some(v) => v,
    None => "",
};

/// ESP32-CAM AI-Thinker pin definitions
pub mod pins {
    pub const PWDN: i32 = 32;
    pub const RESET: i32 = -1; // Not connected
    pub const XCLK: i32 = 0;
    pub const SIOD: i32 = 26; // SDA
    pub const SIOC: i32 = 27; // SCL
    pub const Y9: i32 = 35;
    pub const Y8: i32 = 34;
    pub const Y7: i32 = 39;
    pub const Y6: i32 = 36;
    pub const Y5: i32 = 21;
    pub const Y4: i32 = 19;
    pub const Y3: i32 = 18;
    pub const Y2: i32 = 5;
    pub const VSYNC: i32 = 25;
    pub const HREF: i32 = 23;
    pub const PCLK: i32 = 22;
    pub const FLASH_LED: i32 = 4;
}
