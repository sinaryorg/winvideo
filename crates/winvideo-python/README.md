# WinVideo Python API (`sinary-winvideo`)

[![PyPI version](https://img.shields.io/pypi/v/sinary-winvideo.svg)](https://pypi.org/project/sinary-winvideo/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/sinaryorg/winvideo)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20(19041%2B)%20%7C%20Windows%2011-0078D6.svg)](https://github.com/sinaryorg/winvideo)

Modern, high-performance, driverless Windows screen and per-window video capture library for Python by **SINARY™**.

Powered by Windows Graphics Capture (WGC) and Direct3D 11, delivering zero-copy GPU frame extraction at 60–120+ FPS with ~1% CPU usage. Built-in hardware MP4 encoding using Windows Media Foundation (MFT).

> [!NOTE]
> WinVideo captures the visual video stream only (no audio track). To record application or system audio, pair with [**winaudio**](https://pypi.org/project/sinary-winaudio/).

---

## Features

- 🎯 **Target Any Window or Monitor**: Capture video by executable name (`name="chrome.exe"`), Process ID (`pid=1234`), window title substring (`window="YouTube"`), or monitor index (`monitor=1`).
- ⚡ **Direct3D 11 Hardware Acceleration**: Zero-copy GPU memory extraction delivers 60–120+ FPS with ~1% CPU usage.
- 🐍 **Zero-Copy NumPy Pipeline for AI**: Stream raw BGRA frames directly into NumPy arrays for OpenCV, PyTorch, or Vision LLM agents (Claude Computer-Use, OpenAI Operator).
- 🎬 **Hardware MP4 Recording**: Encode directly to `.mp4` using NVENC, Intel QuickSync, or AMD AMF hardware encoders with zero external `ffmpeg` dependencies.
- 🚫 **Yellow Border Suppression**: Programmatically disables the Windows capture border (`border=False`).
- 🖱️ **Cursor Control**: Optional capture of the mouse cursor (`cursor=True` or `cursor=False`).
- 📸 **Lossless Screenshots**: Save single frames directly to PNG, JPEG, or BMP.

---

## Installation

```bash
pip install sinary-winvideo
```

### System Requirements

- **Operating System**: Windows 10 Build 19041 (Version 2004) or later, or Windows 11
- **Python**: 3.8, 3.9, 3.10, 3.11, 3.12, or 3.13 (64-bit)
- **Direct3D 11 Capable GPU**: NVIDIA, Intel, or AMD graphics

---

## Quick Start

### 1. Enumerate Windows & Monitors

```python
import winvideo

# List all open application windows
for win in winvideo.list_windows():
    print(f"[{win.pid}] {win.process_name} ({win.width}x{win.height}) - '{win.title}'")

# List all display monitors
for mon in winvideo.list_monitors():
    primary = " (Primary)" if mon.is_primary else ""
    print(f"[{mon.index}] {mon.name}{primary} - {mon.width}x{mon.height} @ {mon.refresh_rate}Hz")
```

### 2. Lossless Screenshots

```python
import winvideo

# Instant screenshot of Chrome
winvideo.screenshot("chrome.png", name="chrome.exe")

# Instant screenshot of primary display
winvideo.screenshot("display.png", monitor=1)
```

### 3. Stream Video Frames into OpenCV / AI Models

```python
import winvideo
import cv2

# Stream 60 FPS directly into NumPy arrays
with winvideo.Capture(name="chrome.exe", cursor=False) as cap:
    for frame in cap.stream():
        # frame is a zero-copy NumPy ndarray of shape (Height, Width, 4) uint8
        cv2.imshow("DirectX Stream", frame)
        if cv2.waitKey(1) == 27:  # ESC to exit
            break

cv2.destroyAllWindows()
```

### 4. Record Application Window to MP4

```python
import winvideo

# Record 10 seconds of Chrome directly to hardware-encoded MP4
winvideo.record(name="chrome.exe", out="chrome.mp4", duration=10.0, fps=60)
```

### 5. Pair with WinAudio for Simultaneous Audio + Video

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

---

## License

Dual-licensed under the **MIT License** or **Apache License 2.0**. Copyright © 2026 **SINARY™**.
