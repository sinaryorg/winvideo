# 🎥 WinVideo

[![Crates.io Core](https://img.shields.io/crates/v/winvideo-core.svg?label=crates.io%20(core))](https://crates.io/crates/winvideo-core)
[![Crates.io CLI](https://img.shields.io/crates/v/winvideo-cli.svg?label=crates.io%20(cli))](https://crates.io/crates/winvideo-cli)
[![PyPI version](https://img.shields.io/pypi/v/sinary-winvideo.svg?label=PyPI%20(python))](https://pypi.org/project/sinary-winvideo/)
[![GitHub Release](https://img.shields.io/github/v/release/sinaryorg/winvideo.svg?label=release)](https://github.com/sinaryorg/winvideo/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20(19041%2B)%20%7C%20Windows%2011-0078D6.svg)](https://microsoft.com/windows)
[![Privacy: Local Only](https://img.shields.io/badge/privacy-local--only-success.svg)](PRIVACY.md)
[![Audio Companion: WinAudio](https://img.shields.io/badge/audio%20companion-WinAudio-ff69b4.svg)](https://github.com/sinaryorg/winaudio)

**WinVideo** is a modern, high-performance Windows screen and window video capture engine by **SINARY™**, written in pure Rust, with native Python bindings (`pyo3`) and a standalone command-line executable.

It captures digital video frames at **60 to 120+ FPS** from **specific applications** (e.g., Discord, Spotify, Chrome, games) or entire monitors directly from GPU memory using Windows Graphics Capture (WGC) and Direct3D 11 — **without OBS, virtual display drivers, or GDI lag**.

> [!NOTE]
> **Video-Only Engine (Visual Stream):**  
> WinVideo is dedicated exclusively to ultra-fast, zero-copy **visual capture** (GPU frame extraction, screen recording, and lossless screenshots). The resulting `.mp4` file contains a **video stream only (no audio track)**.  
> To record internal application or desktop audio, pair WinVideo with our companion library [**WinAudio**](https://github.com/sinaryorg/winaudio). See [Audio + Video Recording](#-recording-audio--video-together) below.

---

## ✨ Features

- 🎯 **Target Any Window or Application**: Capture video by Process ID (`--pid`), executable name (`--name chrome.exe`), window title substring (`--window "YouTube"`), or monitor index (`--monitor 1`).
- ⚡ **Direct3D 11 Hardware Acceleration**: Zero-copy GPU memory extraction delivers 60–120+ FPS with ~1% CPU usage.
- 🌲 **Dynamic Process Tree Resolution**: Automatically discovers child rendering processes (such as `msedgewebview2.exe` in WebView2 apps or `chrome.exe` GPU workers) and maps them to their host window.
- 🐍 **Zero-Copy NumPy Pipeline for AI**: Stream raw BGRA/RGBA frames directly into NumPy arrays for OpenCV, PyTorch, or Vision LLM agents (Claude Computer-Use, OpenAI Operator).
- 🎬 **Built-in Hardware MP4 Encoding**: Record directly to `.mp4` using Windows Media Foundation (MFT) hardware encoders (NVENC, Intel QuickSync, AMD AMF) with zero external `ffmpeg` dependencies.
- 🚫 **Yellow Border Suppression**: Programmatically disables the Windows capture border (`BorderRequired = false`).
- 🖱️ **Cursor Control**: Optional capture of the mouse cursor (`--cursor` or `--no-cursor`).
- 📸 **Instant Lossless Screenshots**: Capture single-frame screenshots directly to PNG, JPEG, or BMP.
- 🔊 **Perfect Companion to WinAudio**: Pair with [WinAudio](https://github.com/sinaryorg/winaudio) for a 100% native, driverless desktop audio and video recording stack.
- 📋 **Built-in Runnable Cheatsheet**: Run `winvideo commands` at any time to see every runnable command with copy-pasteable examples.

---

## 📁 Repository Structure

```
win_video/
├── crates/
│   ├── winvideo-core/        # Core WGC & Direct3D 11 engine, MFT hardware MP4 encoder
│   ├── winvideo-cli/         # Standalone CLI binary (target/release/winvideo.exe)
│   └── winvideo-python/      # PyO3 C-extension module for Python (sinary-winvideo)
├── python/                   # Prebuilt Python extension (winvideo.pyd) & test scripts
├── winvideo.cmd              # Direct Windows CMD launcher wrapper
├── winvideo.ps1              # Direct PowerShell launcher wrapper
├── Cargo.toml                # Root workspace manifest
└── README.md
```

---

## 💻 OS & System Requirements

- **Windows 10** Build 19041 (Version 2004) or newer (required for Windows Graphics Capture).
- **Windows 11** (all builds supported).
- **Direct3D 11 Capable GPU**: NVIDIA, Intel, or AMD graphics (NVENC, QuickSync, or AMF for hardware MFT MP4 encoding).
- **Rust Toolchain**: `stable` (1.75+ recommended).
- **Python**: 3.8+ (for Python bindings).

---

## 📦 Installation

### 1. Python Package (`pip`)
Install the precompiled binary Python extension module (supports Python 3.8 to 3.13):
```bash
pip install sinary-winvideo
```

### 2. Cargo / Rust (`crates.io`)
Install the standalone CLI binary:
```bash
cargo install winvideo-cli
```
Or add the core Rust video capture engine to your project's `Cargo.toml`:
```toml
[dependencies]
winvideo-core = "0.1.0"
```

### 3. Prebuilt Binary
Download the standalone `winvideo.exe` from the [GitHub Releases](https://github.com/sinaryorg/winvideo/releases) page and place it in your `PATH`.

---

## 🛠 Building From Source

### 1. Build the Standalone CLI

```powershell
# Build optimized release binary
cargo build --release -p winvideo-cli

# The executable will be at:
# target\release\winvideo.exe
```

### 2. Build the Python Extension

```powershell
# Build the cdylib
cargo build --release -p winvideo-python

# Copy the DLL to python/winvideo.pyd
Copy-Item target\release\winvideo.dll -Destination python\winvideo.pyd -Force
```

### 3. Run Workspace Unit Tests

```powershell
cargo test --workspace
```

---

## 🚀 How to Use

### 1. Command-Line Interface (CLI)

You can run commands using `winvideo` (via the included `winvideo.cmd`) or directly using `.\target\release\winvideo.exe`.

#### View All Runnable Commands (Cheatsheet)
```cmd
winvideo commands
```

#### Discover Windows & Monitors
```cmd
# List all capturable application windows and connected monitors
winvideo list

# List only physical display monitors
winvideo list --monitors
```

#### Record Single Application Window
```cmd
# Record Chrome window live continuously (press Ctrl+C anytime to stop and save)
winvideo record --name chrome.exe

# Record for a specific duration (e.g. 10 seconds, 1 hour = 3600s)
winvideo record --name chrome.exe --duration 10 --out chrome.mp4

# Record specific application by Process ID
winvideo record --pid 15024 --out app.mp4

# Record window matching title substring
winvideo record --window "YouTube" --duration 15 --out video.mp4
```

> **No Time Limits**: WinVideo is not restricted by time. When `--duration` is omitted, it records live continuously for 1 hour, 2 hours, or longer. Press **`Ctrl + C`** at any time to gracefully finalize and save your complete MP4 recording.

#### Record Full Desktop / Monitor
```cmd
# Record primary display monitor at native 60 FPS
winvideo record --monitor 1 --out desktop.mp4

# Record secondary monitor at 120 FPS
winvideo record --monitor 2 --fps 120 --out gaming.mp4
```

#### Capture Instant Lossless Screenshot
```cmd
# Screenshot of Chrome window to screenshot.png
winvideo screenshot --name chrome.exe

# Screenshot of specific window title to custom file path
winvideo screenshot --window "Visual Studio Code" --out ide.png

# Screenshot of full primary display
winvideo screenshot --monitor 1 --out display.png
```

#### Capture Customization Flags
```cmd
# Hide mouse cursor in video
winvideo record --name game.exe --no-cursor

# Record at 30 FPS to reduce video file size
winvideo record --name app.exe --fps 30

# Show Windows yellow capture border (default: hidden)
winvideo record --name app.exe --border
```

---

### 2. Python API (`sinary-winvideo`)

Import `winvideo` directly into your Python scripts:

```python
import winvideo
import cv2

# 1. Enumerate capturable application windows and monitors
for win in winvideo.list_windows():
    print(f"[{win.pid}] {win.process_name} ({win.width}x{win.height}) - '{win.title}'")

for mon in winvideo.list_monitors():
    print(f"[{mon.index}] {mon.name} - {mon.width}x{mon.height} @ {mon.refresh_rate}Hz")

# 2. Capture lossless screenshot
winvideo.screenshot("chrome_screenshot.png", name="chrome.exe")

# 3. Stream frames directly into OpenCV / PyTorch / AI vision agents (60 FPS, ~1% CPU)
with winvideo.Capture(name="game.exe", cursor=False) as cap:
    for frame in cap.stream():
        # frame is a zero-copy numpy array (Height, Width, 4) uint8 (BGRA)
        cv2.imshow("DirectX Stream", frame)
        if cv2.waitKey(1) == 27:  # ESC to exit
            break

# 4. Record application directly to MP4 (video stream only)
with winvideo.Capture(name="Discord.exe") as cap:
    cap.record_to_mp4("meeting.mp4", duration_seconds=10.0)
```

---

### 3. Rust API (`winvideo-core`)

Add `winvideo-core` to your project's `Cargo.toml`:

```toml
[dependencies]
winvideo-core = "0.1.0"
```

```rust
use std::time::Duration;
use winvideo_core::{capture_screenshot, record_to_mp4, list_windows, CaptureConfig, Target};

fn main() -> anyhow::Result<()> {
    // 1. List capturable application windows
    let windows = list_windows()?;
    for w in windows {
        println!("[{}] {} - '{}'", w.pid, w.process_name, w.title);
    }

    // 2. Take an instant lossless screenshot
    capture_screenshot(&Target::ByName("chrome.exe".into()), "chrome.png")?;

    // 3. Record application directly to hardware-encoded MP4
    let config = CaptureConfig::default();
    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    record_to_mp4(
        &Target::ByName("Spotify.exe".into()),
        "spotify.mp4",
        Some(Duration::from_secs(10)),
        &config,
        stop_signal,
        None,
    )?;

    println!("MP4 recording complete!");
    Ok(())
}
```

---

## 🔊 Recording Audio + Video Together

WinVideo records video; [WinAudio](https://github.com/sinaryorg/winaudio) records internal application audio. Because both use Windows hardware high-resolution timers (`QueryPerformanceCounter`), they run concurrently with zero conflict:

### Python Simultaneous Capture:
```python
import threading
import winaudio
import winvideo

TARGET = "Spotify.exe"

# Record 10s of audio and video concurrently
t_audio = threading.Thread(target=lambda: winaudio.record(name=TARGET, out="audio.wav", duration=10))
t_video = threading.Thread(target=lambda: winvideo.record(name=TARGET, out="video.mp4", duration=10))

t_audio.start()
t_video.start()
t_audio.join()
t_video.join()

print("Recorded audio.wav and video.mp4 simultaneously!")
```

### Instant 1-Second Lossless Muxing:
Merge the MP4 video and WAV audio into a single finalized video without re-encoding:
```bash
ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac final_recording.mp4
```

---

## 🏗 Architecture

```
                    ┌──────────────────────────────────────────────┐
                    │      winvideo CLI / Python / Rust API        │
                    └──────────────────────┬───────────────────────┘
                                           │
                    ┌──────────────────────▼───────────────────────┐
                    │                winvideo-core                 │
                    ├──────────────────────┬───────────────────────┤
                    │   Target Resolver    │ Windows Graphics Cap  │
                    │ (Process Tree Walker)│(CreateForWindow/Item) │
                    ├──────────────────────┼───────────────────────┤
                    │  Direct3D 11 Engine  │ Hardware MFT Encoder  │
                    │ (Zero-Copy GPU VRAM) │ (NVENC / QuickSync)   │
                    ├──────────────────────┼───────────────────────┤
                    │  Border Suppression  │ Frame Dispatcher      │
                    │(BorderRequired=false)│(Crossbeam & NumPy DMA)│
                    └──────────────────────┴───────────┬───────────┘
                                                       │
                                     ┌─────────────────▼───────────┐
                                     │ Windows Graphics Capture    │
                                     │ + Direct3D 11 Device Engine │
                                     └─────────────────┬───────────┘
                                                       │
                                     ┌─────────────────▼───────────┐
                                     │ Desktop Window Manager(DWM) │
                                     └─────────────────────────────┘
```

---

## 🔒 Privacy & Local Processing

WinVideo adheres to a strict **local-only, zero-telemetry** policy:
- **100% Offline Execution:** WinVideo contains zero network calls, tracking beacons, or analytics.
- **Local Storage Only:** Video files and screenshots are saved strictly to the paths you specify on your local drive.
- **No Webcam Access:** WinVideo does not access, activate, or record webcams or physical cameras.
- **No Microphone or Audio Capture:** WinVideo exclusively captures visual frames; sound is never recorded. For audio, see [WinAudio](https://github.com/sinaryorg/winaudio).

See [PRIVACY.md](PRIVACY.md) for full details.

---

## 📄 License & Trademark

Copyright © 2026 **SINARY™**. All rights reserved.

**SINARY™** and the SINARY logo are trademarks of SINARY.

Dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
