# WinVideo Documentation

Welcome to the official **WinVideo** documentation by **SINARY™**.

WinVideo is a modern, high-performance Windows screen and window video capture engine written in Rust, featuring a standalone CLI (`winvideo.exe`), official Python bindings (`sinary-winvideo`), and core Rust engine crates (`winvideo-core`).

It captures digital video frames at **60 to 120+ FPS** from **specific applications** (e.g. Discord, Spotify, Chrome, games) or entire monitors directly from GPU memory using Windows Graphics Capture (WGC) and Direct3D 11 — **without OBS, virtual display drivers, or GDI lag**.

> [!NOTE]
> **Video-Only Engine (Visual Stream):**  
> WinVideo is dedicated exclusively to ultra-fast, zero-copy **visual capture** (GPU frame extraction, screen recording, and lossless screenshots). The resulting `.mp4` file contains a **video stream only (no audio track)**.  
> To record internal application or desktop audio, pair WinVideo with our companion library [**WinAudio**](https://github.com/sinaryorg/winaudio).

---

## 📚 Table of Contents

- [CLI Reference Guide](CLI-Reference)
  - Discovering application windows and monitors (`winvideo list`)
  - Target by name, PID, window title substring, or monitor index
  - Continuous and timed recording (`Ctrl+C` safe finalization)
  - Direct hardware MP4 recording (NVENC, QuickSync, AMF)
  - Instant lossless screenshot capture (PNG, JPEG, BMP)
  - Yellow border suppression & cursor control
  - Runnable command cheatsheet (`winvideo commands`)
- [Python API Guide (`sinary-winvideo`)](Python-API)
  - Installation via `pip`
  - Enumerating capturable windows and connected displays
  - Instant lossless screenshots
  - Streaming frames into NumPy, OpenCV, PyTorch, and Vision LLM agents
  - One-line MP4 recording
  - Concurrent Audio + Video recording with WinAudio
- [Rust API Guide (`winvideo-core`)](Rust-API)
  - Adding the crate dependency
  - Target selection variants (`Target`)
  - Taking screenshots via `capture_screenshot`
  - Direct hardware MP4 recording via `record_mp4` and `record_to_mp4`
  - Live frame streaming via `LiveCaptureStream`
- [Architecture & WGC Deep-Dive](Architecture-and-WGC)
  - Windows Graphics Capture (WGC) APIs (`IGraphicsCaptureItemInterop`)
  - Direct3D 11 zero-copy texture swapchains
  - Hardware MFT H.264 video encoding (Windows Media Foundation)
  - Dynamic process tree walking for child rendering workers
  - Border suppression mechanics (`BorderRequired = false`)
  - Synchronized hardware clock (`QueryPerformanceCounter`) with WinAudio

---

## 💻 Operating System Requirements

- **Windows 10**: Build 19041 (Version 2004) or newer (required for Windows Graphics Capture).
- **Windows 11**: All builds supported.
- **Architecture**: 64-bit (`x86_64` / `x64`).
- **GPU**: Direct3D 11 capable graphics card (NVIDIA, Intel, or AMD).
- **Permissions**: Standard user permissions (Administrator not required unless capturing elevated windows).

---

## 📦 Official Distribution Channels

- **Python (`pip`)**: `pip install sinary-winvideo` ([PyPI Project](https://pypi.org/project/sinary-winvideo/))
- **Rust Core (`crates.io`)**: `winvideo-core = "0.1.0"` ([crates.io/crates/winvideo-core](https://crates.io/crates/winvideo-core))
- **CLI via Cargo**: `cargo install winvideo-cli` ([crates.io/crates/winvideo-cli](https://crates.io/crates/winvideo-cli))
- **GitHub Releases**: Pre-built stand-alone binaries and wheels at [sinaryorg/winvideo/releases](https://github.com/sinaryorg/winvideo/releases)

---

## 📄 License & Trademark

Copyright © 2026 **SINARY™**. All rights reserved.

**SINARY™** and the SINARY logo are trademarks of SINARY.

Dual-licensed under either:
- **MIT License**
- **Apache License, Version 2.0**
