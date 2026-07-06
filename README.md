# RRSVP Rapid Serial Visual Presentation Reader

> A Rust rewrite of the beloved [RSVP Nano](https://github.com/ionutdecebal/rsvpnano) C++ project — faster, safer, and built for the long haul.

[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Target: ESP32-S3](https://img.shields.io/badge/target-ESP32--S3-green?logo=espressif)](https://www.espressif.com/en/products/socs/esp32-s3)
[![no_std core](https://img.shields.io/badge/core-no__std-lightgrey)](crates/core)

RRSVP is an open-source embedded RSVP reader running on the Waveshare ESP32-S3-Touch-LCD-3.49. It displays text one word at a time using Rapid Serial Visual Presentation, a technique that reduces eye movement and lets you read significantly faster. A desktop preview app is included for development and testing without hardware.

---

## Table of Contents

- [Features](#-features)
- [Hardware](#-hardware)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Desktop (no hardware required)](#desktop-no-hardware-required)
  - [Embedded (ESP32-S3)](#embedded-esp32-s3)
- [Usage](#-usage)
  - [Controls](#controls)
  - [Reading Modes](#reading-modes)
- [Configuration & Settings](#-configuration--settings)
- [File Format](#-file-format-rsvp)
- [Project Structure](#-project-structure)
- [Contributing](#-contributing)
- [License](#-license)

---

## Features

- **Full RSVP pacing engine**: timing adapts per word based on length, complexity, syllable count, and punctuation
- **ORP highlighting**: Optimal Recognition Point marker with 7 color options
- **Dual reading modes**: RSVP (word-by-word) and Scroll (traditional scrolling text)
- **Focus mode**: configurable overlay: None / Bars / Bars + Line / Line
- **4 font sizes**: S / M / L / XL
- **Phantom words**: display the next word faintly to aid anticipation
- **Dark & light mode**
- **WPM range 10–1000**: fine and coarse stepping
- **6 UI languages**: English, Spanish, French, German, Romanian, Polish
- **Power management**: hold POWER 3 s to deep sleep; press to wake
- **SD card library**: load `.rsvp` books from a FAT32 microSD card
- **Custom `.rsvp` format**: compact single-byte Latin encoding supporting 100+ extended characters
- **Desktop preview app**: egui/eframe window at 640×172 for rapid UI iteration without hardware

---

## Hardware

RRSVP currently targets the **Waveshare ESP32-S3-Touch-LCD-3.49** (held landscape).

| Component | Detail |
|---|---|
| SoC | ESP32-S3R8, dual-core Xtensa LX7 |
| Display | 3.49″ AXS15231B QSPI LCD, 172×640 px |
| Touch | I2C capacitive touch |
| Storage | microSD (FAT32) |
| Power button | GPIO16 hold 3 s to sleep, press to wake |
| BOOT button | GPIO0 |
| Backlight | TCA9554 I2C GPIO expander |

**Purchase links:**
- [Waveshare ESP32-S3-Touch-LCD-3.49 (kit)](https://www.waveshare.com/esp32-s3-touch-lcd-3.49.htm)
- [Waveshare Wiki / Schematic](https://docs.waveshare.com/ESP32-S3-Touch-LCD-3.49?variant=ESP32-S3-Touch-LCD-3.49B-EN)

---

## Getting Started

### Prerequisites

Install the following before building anything:

- **Rust** (stable) via [rustup](https://rustup.rs/)
- **`just`** task runner:

```sh
cargo +stable install just
```

### Desktop (no hardware required)

No extra tooling needed. Run the desktop preview app with:

```sh
just run-desktop
```

This opens a 640×172 egui window that mirrors the embedded UI — useful for iterating on layout and reading settings without a device.

### Embedded (ESP32-S3)

#### Step 1 - Install the Xtensa Rust toolchain

```sh
cargo +stable install espup --version 0.17.1
espup install --toolchain-version 1.88.0.0
source ~/export-esp.sh
```

Add the export to your shell profile so it loads automatically:

```sh
echo 'source ~/export-esp.sh' >> ~/.bashrc
```

> **Minimum toolchain version:** Xtensa Rust **1.88.0.0** or newer. Older versions have MSRV conflicts with transitive dependencies.

#### Step 2 - Install flash and linker tools

```sh
cargo +stable install espflash --version 4.4.0
cargo +stable install ldproxy  --version 0.3.4
```

#### Step 3 - Build

```sh
just build-embedded
```

> The first build downloads ESP-IDF v5.3 (~600 MB) and compiles it. This takes **some time**. All subsequent builds are incremental.

The `just build-embedded` recipe sources `~/export-esp.sh` and handles SSL certificate workarounds automatically — no manual environment prep is needed.

#### Step 4 - Flash

Connect the device via USB, then:

```sh
just flash
```

#### Step 5 - Serial monitor (without reflashing)

```sh
just monitor
```

---

## Usage

### Controls

The screen is divided into three horizontal zones (header, body, footer). Touch targets vary by zone:

**Header row**

| Zone | Action |
|---|---|
| Left tap `< MENU` | Return to the main menu |
| Center | Displays status info (book title, progress) |
| Right tap | Toggle reading mode (RSVP ↔ Scroll) |

**Body**

| Tap zone | RSVP mode | Scroll mode |
|---|---|---|
| Left third | Rewind to sentence start | Scroll back |
| Right third | Skip forward | Scroll forward |

**Footer row**

| Zone | Action |
|---|---|
| Left quarter | WPM − (RSVP) / scroll back 10 lines (Scroll) |
| Center-right (between center and play/pause) | WPM + (RSVP) / scroll forward 10 lines (Scroll) |
| Right quarter | Play / Pause |

The footer is divided into three zones: **[WPM−] [WPM+] [▶/⏸]** the rightmost quarter is the play/pause button.

**Hardware buttons**

| Button | Action |
|---|---|
| POWER (GPIO16) hold 3 s | Deep sleep |
| POWER (GPIO16) press | Wake from sleep |

### Reading Modes

| Mode | Description |
|---|---|
| **RSVP** | Words flash one at a time at the center of the screen at your chosen WPM |
| **Scroll** | Traditional scrolling text view, swipe or use footer buttons to navigate |

Switch modes via the right-hand tap zone in the header, or from the Settings menu.

---

## ️ Configuration & Settings

Access settings from the main menu. Available options:

| Setting | Options |
|---|---|
| Reading speed (WPM) | 10–1000 (fine / coarse stepping) |
| Font size | S / M / L / XL |
| ORP color | 7 color options |
| Focus mode | None / Bars / Bars + Line / Line |
| Phantom words | On / Off |
| Dark mode | Dark / Light |
| Language | EN / ES / FR / DE / RO / PL |

Settings are persisted to the device and survive power cycles.

---

## File Format (`.rsvp`)

RRSVP uses a compact custom binary format designed to fit within the constraints of embedded flash and SD card reads.

- **Encoding:** single-byte Latin encoding covering 100+ extended characters (accented vowels, common punctuation, etc.)
- **Structure:** metadata header + packed word stream
- **Source formats:** convert from `.epub`, `.txt`, `.md`, `.html` using the companion web converter at [ionutdecebal.github.io/rsvpnano](https://ionutdecebal.github.io/rsvpnano/)

Place converted `.rsvp` files on a FAT32 microSD card under `/books/books/` or `/books/articles/`. The device indexes them automatically on first open.

---

## Project Structure

This project is a Cargo workspace with three crates:

```
crates/
├── core/       — Pure Rust, no_std-compatible
│                 Pacing engine, text encoding, settings keys
│                 No platform dependencies — usable on embedded and desktop
│
├── embedded/   — ESP32-S3 firmware
│                 esp-idf-svc 0.52.1, embedded-graphics 0.8
│                 Display driver, touch input, SD card, UI screens
│
└── desktop/    — Desktop preview app
                  egui / eframe, 640×172 window
                  Mirrors the embedded UI for rapid iteration
```

### Available `just` commands

```sh
just                       # List all commands
just run-desktop           # Run the desktop preview app
just build-desktop         # Build desktop (debug)
just build-desktop-release # Build desktop (release)
just build-embedded        # Build ESP32-S3 firmware (release)
just flash                 # Flash firmware and open serial monitor
just monitor               # Open serial monitor (no flash)
just check                 # Check core + desktop (no ESP toolchain needed)
just fmt                   # Format all source files
just clippy                # Lint core + desktop
just test                  # Run tests for core + desktop
```

---

## Contributing

RRSVP is a Rust rewrite of the original **[RSVP Nano](https://github.com/ionutdecebal/rsvpnano)** C++ project by [Ionut Decebal](https://github.com/ionutdecebal). The original project pioneered the hardware design, `.rsvp` file format, and core UX concepts that this port builds on. Huge thanks to all RSVP Nano contributors, this project would not exist without their work.

### How to contribute

1. Fork the repository and create a feature branch.
2. Run `just check` and `just test` before submitting a PR — both must pass without ESP toolchain.
3. Run `just fmt` and `just clippy` to keep code style consistent.
4. Open a pull request with a clear description of the change and any relevant context.

New features, bug fixes, hardware ports, and documentation improvements are all welcome.

---

## License

MIT — see [LICENSE](LICENSE) for details.

Original C++ project ([RSVP Nano](https://github.com/ionutdecebal/rsvpnano)) is also MIT licensed. Copyright © 2026 RSVP Nano contributors.
