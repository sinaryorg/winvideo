# winvideo-cli

[![Crates.io](https://img.shields.io/crates/v/winvideo-cli.svg)](https://crates.io/crates/winvideo-cli)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/sinaryorg/winvideo)

Standalone command-line tool for high-performance Windows screen and window video capture by **SINARY™**.

---

## Installation

```bash
cargo install winvideo-cli
```

---

## Usage

```bash
# List capturable application windows and monitors
winvideo list

# Record specific application to MP4
winvideo record --name chrome.exe --out chrome.mp4 --duration 10

# Record entire display monitor at 60 FPS
winvideo record --monitor 1 --fps 60 --out desktop.mp4

# Lossless screenshot
winvideo screenshot --name spotify.exe --out spotify.png

# View runnable commands cheatsheet
winvideo commands
```

---

## License

Dual-licensed under MIT or Apache-2.0. Copyright © 2026 SINARY™.
