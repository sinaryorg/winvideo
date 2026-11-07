# 🎥 WinVideo

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%20Windows%2011-0078D6.svg)](https://microsoft.com/windows)
[![Privacy: Local Only](https://img.shields.io/badge/privacy-local--only-success.svg)](PRIVACY.md)
[![Audio Companion: WinAudio](https://img.shields.io/badge/audio%20companion-WinAudio-ff69b4.svg)](https://github.com/sinaryorg/winaudio)

**WinVideo** is a modern, high-performance, driverless Windows screen and window video capture engine by **SINARY™**, written in pure Rust, with native Python bindings (`pyo3`) and a standalone command-line executable (`winvideo.exe`).

It captures digital video frames at **60 to 120+ FPS** from **specific applications** (e.g., Discord, Chrome, Spotify, games) or entire monitors directly from GPU memory using Windows Graphics Capture (WGC) and Direct3D 11 — **without OBS, virtual display drivers, or GDI lag**.

> [!NOTE]
> **Video-Only Engine (Visual Stream):**  
> WinVideo is dedicated exclusively to ultra-fast, zero-copy **visual capture** (GPU frame extraction, screen recording, and lossless screenshots). The resulting `.mp4` file contains a **video stream only (no audio track)**.  
> To record internal application or desktop audio, pair WinVideo with our companion library [**WinAudio**](https://github.com/sinaryorg/winaudio). See [Audio + Video Recording](#-recording-audio--video-together) below.

---

## ✨ Features

- 🎯 **Target Any Window or Application**: Capture video by Process ID (`--pid`), executable name (`--name chrome.exe`), window title substring (`--window "YouTube"`), or monitor index (`--monitor 0`).
- ⚡ **Direct3D 11 Hardware Acceleration**: Zero-copy GPU memory extraction delivers 60–120+ FPS with ~1% CPU usage.
- 🐍 **Zero-Copy NumPy Pipeline for AI**: Stream raw BGRA/RGBA frames directly into NumPy arrays for OpenCV, PyTorch, or Vision LLM agents (Claude Computer-Use, OpenAI Operator).
- 🎬 **Built-in Hardware MP4 Encoding**: Record directly to `.mp4` using Windows Media Foundation (MFT) hardware encoders (NVENC, QuickSync, AMF) with zero external `ffmpeg` dependencies.
- 🚫 **Yellow Border Suppression**: Programmatically disables the Windows capture border (`BorderRequired = false`).
- 🖱️ **Cursor Control**: Optional capture of the mouse cursor (`--cursor` or `--no-cursor`).
- 🔊 **Perfect Companion to WinAudio**: Pair with [WinAudio](https://github.com/sinaryorg/winaudio) for a 100% native, driverless desktop audio/video recording stack.

---

## 🛠 Repository Layout

```
win_video/
├── Cargo.toml                    # Root workspace
├── crates/
│   ├── winvideo-core/            # WGC & Direct3D 11 capture engine
│   ├── winvideo-cli/             # Standalone CLI binary (winvideo.exe)
│   └── winvideo-python/          # PyO3 Python bindings (sinary-winvideo)
├── python/                       # Python stubs, tests & examples
├── winvideo.cmd                  # Quick Windows CMD launcher
└── winvideo.ps1                  # Quick PowerShell launcher
```

---

## 🚀 Usage

### 1. Command-Line Interface (CLI)

```bash
# List all capturable application windows and monitors
winvideo list

# Record specific application by name (interactive Ctrl+C or fixed duration)
winvideo record --name chrome.exe --out chrome.mp4 --duration 10

# Record specific window title substring
winvideo record --window "Visual Studio Code" --out vscode.mp4

# Record entire primary monitor at 60 FPS
winvideo record --monitor 0 --fps 60 --out desktop.mp4

# Instant lossless PNG screenshot
winvideo screenshot --name spotify.exe --out spotify.png

# View runnable commands cheatsheet
winvideo commands
```

### 2. Python API (`sinary-winvideo`)

```python
import winvideo
import cv2

# 1. Enumerate capturable windows and monitors
for win in winvideo.list_windows():
    print(f"[{win.pid}] {win.process_name} - '{win.title}'")

# 2. Capture lossless screenshot
winvideo.screenshot(name="chrome.exe", out="chrome.png")

# 3. Stream frames directly into OpenCV / AI models (60 FPS, ~1% CPU)
with winvideo.Capture(name="game.exe", cursor=False) as cap:
    for frame in cap.stream():
        # frame is a zero-copy numpy array (Height, Width, 4) uint8
        cv2.imshow("Stream", frame)
        if cv2.waitKey(1) == 27:
            break

# 4. Record directly to MP4 (video stream only)
with winvideo.Capture(name="Discord.exe") as cap:
    cap.record_to_mp4("discord_call.mp4", duration_seconds=10.0)
```

---

## 🔊 Recording Audio + Video Together

WinVideo records video; [WinAudio](https://github.com/sinaryorg/winaudio) records internal application audio. Because both use Windows hardware high-resolution timers (`QueryPerformanceCounter`), they can run concurrently with zero conflict:

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
```

### Instant 1-Second Lossless Muxing:
Merge the MP4 video and WAV audio into a single finalized video without re-encoding:
```bash
ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac output_with_audio.mp4
```

---

## 🔒 Privacy & Local Processing

WinVideo adheres to a strict **local-only, zero-telemetry** policy. It does not access webcams, never transmits screen contents over any network, and operates 100% offline. See [PRIVACY.md](PRIVACY.md) for full details.

---

## 📄 License

Dual-licensed under the **MIT License** ([LICENSE-MIT](LICENSE-MIT)) or the **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE)) at your option.
