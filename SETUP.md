# RRSVP Developer Setup

## Prerequisites (all platforms)

- **Rust** via [rustup](https://rustup.rs/)
- **just** task runner: `cargo +stable install just`

---

## Desktop Development

No additional tooling required. The repo root uses the stable Rust toolchain.

```sh
just run-desktop
```

---

## ESP32-S3 Development

### 1 - Install the Xtensa Rust toolchain

```sh
cargo +stable install espup --version 0.17.1
espup install --toolchain-version 1.88.0.0
source ~/export-esp.sh
```

Add `source ~/export-esp.sh` to your shell profile so it is available in every new terminal:

```sh
echo 'source ~/export-esp.sh' >> ~/.bashrc
```

> **Toolchain requirement**: Xtensa Rust **1.88.0.0 or newer** is required. Older versions (1.85, 1.79) have MSRV conflicts with transitive dependencies.

### 2 - Install flash and linker tools

```sh
cargo +stable install espflash --version 4.4.0
cargo +stable install ldproxy  --version 0.3.4
```

### 3 - Build

```sh
just build-embedded
```

`just build-embedded` sources `~/export-esp.sh` and handles the Python SSL cert workaround automatically. No manual environment setup is needed before running it.

The **first build** downloads ESP-IDF v5.3 (~600 MB) and compiles it. This takes 5–20 minutes. All subsequent builds are incremental.

### 4 - Flash

Connect the Waveshare ESP32-S3-Touch-LCD-3.49 via USB, then:

```sh
just flash
```

### 5 - Serial monitor (no reflash)

```sh
just monitor
```

---

## Notes

- The `sdkconfig` file in `crates/embedded/` is auto-generated and git-ignored. Edit `sdkconfig.defaults` to change board configuration.
- ESP-IDF is cached in `~/.espressif/` and the project-local `.embuild/` directory.
- `crates/embedded/patches/esp-idf-hal` contains a local patch for `esp-idf-hal 0.46.2` that fixes a bindgen 0.71 struct naming regression in the RMT driver. Do not remove this patch.

---

## AXS15231B Display Driver Status

`crates/embedded/src/display.rs` is a stub. The AXS15231B has no upstream crates.io driver. A full driver will require:

1. QSPI bus initialisation on the correct GPIO pins (see board schematic).
2. Custom vendor init sequence (source from Waveshare reference firmware).
3. An `embedded-graphics` `DrawTarget<Color = Rgb565>` implementation backed by PSRAM frame buffer + DMA via the ESP-IDF `esp_lcd` component.
