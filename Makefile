TARGET = xtensa-esp32-espidf
BUILD_STD = -Zbuild-std=std,panic_abort
ESPFLASH_PORT ?= /dev/ttyUSB0

.PHONY: build release flash monitor clean fmt lint check doc size

# Build debug binary
build:
	cargo build --target $(TARGET) $(BUILD_STD)

# Build release binary (optimized)
release:
	cargo build --release --target $(TARGET) $(BUILD_STD)

# Flash release firmware to ESP32-CAM
flash: release
	espflash flash --monitor --port $(ESPFLASH_PORT) \
		target/$(TARGET)/release/birdhouse-cam

# Monitor serial output
monitor:
	espflash monitor --port $(ESPFLASH_PORT)

# Format all Rust source files
fmt:
	cargo fmt

# Check formatting without modifying files
fmt-check:
	cargo fmt -- --check

# Run clippy linter
lint:
	cargo clippy --no-deps --target $(TARGET) $(BUILD_STD) -- -Dwarnings

# Type check without building
check:
	cargo check --target $(TARGET) $(BUILD_STD)

# Generate documentation
doc:
	cargo doc --no-deps --target $(TARGET) $(BUILD_STD) --open

# Report binary size
size: release
	@echo "=== Binary Size Report ==="
	@find target/$(TARGET)/release -name "birdhouse-cam" -not -path "*/deps/*" \
		-not -path "*/build/*" -exec stat -c '%s bytes (%n)' {} \;

# Run all CI checks locally
ci: fmt-check lint build

# Install development tools
setup:
	cargo install espup espflash ldproxy
	espup install
	@echo ""
	@echo "Run '. $$HOME/export-esp.sh' to activate the ESP toolchain"
	@echo "Then install pre-commit hooks with: pip install pre-commit && pre-commit install"

# Install pre-commit hooks
hooks:
	pip install pre-commit
	pre-commit install

# Clean build artifacts
clean:
	cargo clean
	rm -rf .embuild managed_components

# Full rebuild from clean state
rebuild: clean build

help:
	@echo "Available targets:"
	@echo "  build      - Build debug binary"
	@echo "  release    - Build optimized release binary"
	@echo "  flash      - Flash release firmware to ESP32-CAM"
	@echo "  monitor    - Monitor serial output"
	@echo "  fmt        - Format source code"
	@echo "  fmt-check  - Check formatting (no changes)"
	@echo "  lint       - Run clippy linter"
	@echo "  check      - Type check only"
	@echo "  ci         - Run all CI checks locally"
	@echo "  setup      - Install development tools"
	@echo "  hooks      - Install pre-commit hooks"
	@echo "  size       - Report binary size"
	@echo "  doc        - Build and open documentation"
	@echo "  clean      - Remove build artifacts"
	@echo "  rebuild    - Clean and rebuild"
