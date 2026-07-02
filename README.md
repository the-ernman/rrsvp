# Rust Rapid Serial Visual Presentation (RSVP) Reader

A Rust implementation of a Rapid Serial Visual Presentation (RSVP) reader. Designed to help users read text more efficiently by displaying words one at a time in a fixed position on the screen.

## Background

Rapid Serial Visual Presentation (RSVP) is a speed-reading technique that presents words in a sequence at a fixed location on the screen, allowing readers to focus on one word at a time. This method can significantly increase reading speed and comprehension by minimizing eye movement and distractions.

This project aims to implement an RSVP reader using the Rust programming language, leveraging the performance and safety features of Rust to create a responsive and efficient reading application.

## Goals

- Implement a responsive RSVP reader in Rust.
- Optimize for performance and safety using Rust's features.
- Provide a user-friendly interface for efficient reading.
- Provide customization options for reading speed, font size, display settings and other user preferences.
- Create a custom file format for storing, loading and reading resources within the application.
- File translator to convert existing text, epub, etc formats into the custom file format for use within the application.
- Design to be compatible with the ESP32-S3 microcontroller and its touch screen display for a portable reading experience.
- Design to be compatible with native desktop platforms (Windows, macOS, Linux) for a versatile reading experience across devices.

## Hardware Specifications

- Built for the ESP32-S3 microcontroller with a 3.49inch Touch Screen (172X640) with ESP32-S3R8 Dual-core processor (held horizontally).
    - [Kit Link](https://www.waveshare.com/esp32-s3-touch-lcd-3.49.htm?srsltid=AfmBOor3K_nKkWZ9civEcIQXUEXl0pAd3hivjLtvLBPcmP9xhz9urYIK)
    - [Wiki](https://docs.waveshare.com/ESP32-S3-Touch-LCD-3.49?variant=ESP32-S3-Touch-LCD-3.49B-EN)
