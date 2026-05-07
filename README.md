# Birdhouse Camera

[![CI](https://github.com/usafbran/birdhouse-cam/actions/workflows/ci.yml/badge.svg)](https://github.com/usafbran/birdhouse-cam/actions/workflows/ci.yml)

An ESP32-CAM application written in Rust that detects and classifies birds inside a birdhouse, sending real-time alerts to Home Assistant via MQTT.

## Features

- **Motion Detection**: On-device frame differencing detects bird activity without needing a powerful processor
- **Bird Classification**: Optional offloading to a local classification server for species identification (MobileNetV2, EfficientNet, etc.)
- **Home Assistant Integration**: Full MQTT auto-discovery — the device and all entities appear automatically in HA
- **Camera Snapshots**: Publishes JPEG snapshots to Home Assistant when birds are detected
- **Diagnostic Reporting**: Uptime, free heap, and device status published as HA diagnostic entities
- **WiFi Reconnection**: Automatic reconnection with exponential backoff
- **Configurable Sensitivity**: Tunable motion thresholds and cooldown periods

## Home Assistant Entities

Once powered on and connected, these entities auto-register in Home Assistant:

| Entity | Type | Description |
|--------|------|-------------|
| `binary_sensor.bird_detected` | Binary Sensor | `ON` when a bird is detected |
| `sensor.bird_species` | Sensor | Last classified species name |
| `sensor.motion_score` | Sensor | Motion detection score (0–100%) |
| `camera.birdhouse_camera` | Camera | Latest snapshot from inside the birdhouse |
| `sensor.device_info` | Diagnostic | Uptime, heap, firmware version |

## Hardware Requirements

- **ESP32-CAM** module (AI-Thinker variant recommended) with OV2640 camera
- **PSRAM** — required for camera frame buffers (most ESP32-CAM boards include 4MB PSRAM)
- USB-to-serial programmer (FTDI or similar) for flashing
- 5V power supply (USB or battery with regulator)
- WiFi network in range of the birdhouse

### Pin Mapping (AI-Thinker ESP32-CAM)

| Function | GPIO |
|----------|------|
| PWDN | 32 |
| XCLK | 0 |
| SDA | 26 |
| SCL | 27 |
| D7–D0 | 35, 34, 39, 36, 21, 19, 18, 5 |
| VSYNC | 25 |
| HREF | 23 |
| PCLK | 22 |
| Flash LED | 4 |

## Software Prerequisites

1. **Rust (ESP fork)** — Install via [espup](https://github.com/esp-rs/espup):

   ```bash
   cargo install espup
   espup install
   . $HOME/export-esp.sh
   ```

2. **ESP-IDF toolchain** — The build system downloads ESP-IDF v5.3.2 automatically via `esp-idf-sys`

3. **espflash** — For flashing firmware:

   ```bash
   cargo install espflash
   ```

4. **MQTT Broker** — Mosquitto or the built-in HA MQTT broker

## Configuration

Set these environment variables before building:

```bash
# Required
export WIFI_SSID="YourWiFiNetwork"
export WIFI_PASS="YourWiFiPassword"
export MQTT_HOST="192.168.1.100"        # Your Home Assistant / MQTT broker IP

# Optional
export MQTT_USER="mqtt_user"            # MQTT username (if auth enabled)
export MQTT_PASS="mqtt_password"        # MQTT password
export DEVICE_NAME="Backyard Birdhouse" # Friendly name in HA
export DEVICE_ID="birdhouse_cam_01"     # Unique device identifier
export CLASSIFICATION_SERVER="http://192.168.1.50:8080"  # Bird classifier URL
```

## Building & Flashing

```bash
# Build (using make)
make build          # Debug build
make release        # Optimized release build

# Or directly with cargo
cargo build --release

# Flash to ESP32-CAM (put board in download mode first)
make flash          # Builds release and flashes
# Or: cargo run --release
```

### Putting the ESP32-CAM in Download Mode

1. Connect GPIO0 to GND
2. Press the reset button
3. Release GPIO0 after flashing begins

## Architecture

```
┌──────────────┐    ┌─────────────────┐    ┌──────────────────┐
│  OV2640      │    │  ESP32-CAM      │    │  Home Assistant  │
│  Camera      │───>│                 │    │                  │
│              │    │  ┌────────────┐ │    │  ┌────────────┐  │
└──────────────┘    │  │ Motion     │ │    │  │ MQTT       │  │
                    │  │ Detection  │ │    │  │ Integration│  │
                    │  └─────┬──────┘ │    │  └─────┬──────┘  │
                    │        │        │    │        │         │
                    │  ┌─────▼──────┐ │    │  ┌─────▼──────┐  │
                    │  │ MQTT       │─┼────┼─>│ Entities   │  │
                    │  │ Publisher  │ │WiFi│  │ & Alerts   │  │
                    │  └─────┬──────┘ │    │  └────────────┘  │
                    │        │        │    │                  │
                    │  ┌─────▼──────┐ │    └──────────────────┘
                    │  │ Classifier │ │
                    │  │ (optional) │─┼───> Classification Server
                    │  └────────────┘ │     (Raspberry Pi / PC)
                    └─────────────────┘
```

### Detection Pipeline

1. **Capture**: OV2640 captures JPEG frames at the configured interval (default 2s)
2. **Motion Detection**: Compares frame hashes and JPEG sizes against previous frames
3. **Alert**: If motion exceeds the threshold and cooldown has elapsed:
   - Publishes `bird_detected = ON` to MQTT
   - Publishes the JPEG snapshot to the camera topic
   - Optionally sends the frame to a classification server for species ID
4. **Recovery**: After the cooldown period, the system is ready for the next detection

## Home Assistant Configuration

### Prerequisites

1. Enable MQTT in Home Assistant (Settings → Devices & Services → Add Integration → MQTT)
2. Ensure MQTT discovery is enabled (it is by default)
3. The device will auto-discover — no YAML configuration needed

### Example Automation: Bird Alert

```yaml
automation:
  - alias: "Bird Detected Alert"
    trigger:
      - platform: state
        entity_id: binary_sensor.birdhouse_cam_01_bird_detected
        to: "on"
    action:
      - service: notify.mobile_app_your_phone
        data:
          title: "Bird Spotted!"
          message: >
            A {{ states('sensor.birdhouse_cam_01_species') }} was detected
            at the birdhouse!
          data:
            image: "/api/camera_proxy/camera.birdhouse_cam_01_camera"
```

### Example Automation: Daily Bird Count

```yaml
automation:
  - alias: "Count Daily Birds"
    trigger:
      - platform: state
        entity_id: binary_sensor.birdhouse_cam_01_bird_detected
        to: "on"
    action:
      - service: counter.increment
        target:
          entity_id: counter.daily_bird_count
```

## Bird Classification Server

The device supports optional species classification by sending captured images
to an HTTP server. See [`model/README.md`](model/README.md) for setup instructions,
including a ready-to-use FastAPI server with a MobileNetV2 backbone.

## Tuning Detection Sensitivity

Adjust these constants in `src/config.rs` or via environment variables:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MOTION_PIXEL_PERCENT` | 5.0 | Percentage threshold for motion detection |
| `CAPTURE_INTERVAL_MS` | 2000 | Time between frame captures (ms) |
| `COOLDOWN_PERIOD_MS` | 30000 | Minimum time between alerts (ms) |
| `CAMERA_JPEG_QUALITY` | 12 | JPEG quality (0–63, lower = better) |
| `CAMERA_FRAME_SIZE` | 10 (SVGA) | Resolution enum value |

## Birdhouse Installation Tips

- Mount the camera facing the entrance hole or perch
- Ensure the ESP32-CAM is protected from moisture (conformal coating recommended)
- Use a weatherproof enclosure for the electronics
- Position the WiFi antenna for best signal (external antenna recommended)
- Power via a weatherproof USB cable or solar panel with battery

## Development

### Setup

```bash
# Install the ESP Rust toolchain and dev tools
make setup

# Activate the toolchain
. $HOME/export-esp.sh

# Install pre-commit hooks
make hooks
```

### Workflow

```bash
make fmt          # Format code
make lint         # Run clippy linter
make check        # Type check
make ci           # Run all checks (fmt + lint + build)
make size         # Report binary size
make doc          # Generate and open docs
```

### CI/CD

GitHub Actions runs on every push and PR:

| Job | Description |
|-----|-------------|
| **Format Check** | Verifies `cargo fmt` compliance |
| **Clippy Lint** | Runs pedantic clippy lints with `-Dwarnings` |
| **Build (dev)** | Debug build for `xtensa-esp32-espidf` |
| **Build (release)** | Release build with LTO |
| **Binary Size Report** | Reports the firmware binary size |

### Pre-commit Hooks

The project uses [pre-commit](https://pre-commit.com/) to enforce quality before commits:

- Trailing whitespace and EOF fixes
- YAML/TOML validation
- `cargo fmt` formatting
- `cargo clippy` linting
- `cargo check` type checking

## Project Structure

```
birdhouse-cam/
├── .cargo/
│   └── config.toml              # Cargo build target and environment config
├── .github/
│   └── workflows/
│       └── ci.yml               # GitHub Actions CI pipeline
├── .pre-commit-config.yaml      # Pre-commit hook definitions
├── Cargo.toml                   # Dependencies, lints, and ESP-IDF components
├── Makefile                     # Dev commands (build, flash, lint, etc.)
├── build.rs                     # Build script for ESP-IDF integration
├── bindings.h                   # C header for esp32-camera FFI bindings
├── partitions.csv               # Flash partition layout
├── rust-toolchain.toml          # Rust ESP toolchain specification
├── rustfmt.toml                 # Code formatting rules
├── sdkconfig.defaults           # ESP-IDF SDK configuration
├── src/
│   ├── main.rs                  # Application entry point and main loop
│   ├── config.rs                # Compile-time configuration constants
│   ├── wifi.rs                  # WiFi station mode connection management
│   ├── camera.rs                # OV2640 camera driver (FFI to esp32-camera)
│   ├── detection.rs             # Motion detection and remote classification
│   ├── mqtt.rs                  # MQTT client for publishing to broker
│   └── homeassistant.rs         # HA MQTT auto-discovery message publishing
└── model/
    └── README.md                # Bird classification model setup guide
```

## License

MIT
