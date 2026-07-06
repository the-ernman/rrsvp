# RRSVP — project task runner
# Install: cargo +stable install just

# List all available commands
default:
    @just --list

# Build the desktop application (debug)
build-desktop:
    cargo build -p rrsvp-desktop

# Build the desktop application (release)
build-desktop-release:
    cargo build -p rrsvp-desktop --release

# Run the desktop application
run-desktop:
    cargo run -p rrsvp-desktop

# Requires espup + espflash + ldproxy installed via SETUP.md.
# Environment is sourced automatically inside each recipe.

# Build embedded firmware (release)
build-embedded:
    #!/usr/bin/env bash
    set -e
    source ~/export-esp.sh
    # Python 3.14 on macOS does not trust system certs by default.
    # Find the certifi bundle that embuild installs and export it so
    # ESP-IDF download steps can reach GitHub/dl.espressif.com.
    _cert=$(find "$HOME/.espressif" "{{justfile_directory()}}/.embuild" -name "cacert.pem" 2>/dev/null | head -1)
    if [ -z "$_cert" ]; then
        _cert=$(python3 -c "import certifi; print(certifi.where())" 2>/dev/null || true)
    fi
    [ -n "$_cert" ] && export SSL_CERT_FILE="$_cert" && export REQUESTS_CA_BUNDLE="$_cert"
    cd "{{justfile_directory()}}/crates/embedded" && cargo +esp build --release

# Flash firmware to connected ESP32-S3 and open serial monitor
flash:
    #!/usr/bin/env bash
    set -e
    source ~/export-esp.sh
    _cert=$(find "$HOME/.espressif" "{{justfile_directory()}}/.embuild" -name "cacert.pem" 2>/dev/null | head -1)
    if [ -z "$_cert" ]; then
        _cert=$(python3 -c "import certifi; print(certifi.where())" 2>/dev/null || true)
    fi
    [ -n "$_cert" ] && export SSL_CERT_FILE="$_cert" && export REQUESTS_CA_BUNDLE="$_cert"
    cd "{{justfile_directory()}}/crates/embedded" && cargo +esp run --release

# Open serial monitor on the connected device (without flashing)
monitor:
    espflash monitor

# Check core + desktop compile without errors (no ESP toolchain required)
check:
    cargo check -p rrsvp-core -p rrsvp-desktop

# Format all source files
fmt:
    cargo fmt --all

# Lint core + desktop
clippy:
    cargo clippy -p rrsvp-core -p rrsvp-desktop -- -D warnings

# Run tests for core + desktop
test:
    cargo test -p rrsvp-core -p rrsvp-desktop
