# winvideo-core

[![Crates.io](https://img.shields.io/crates/v/winvideo-core.svg)](https://crates.io/crates/winvideo-core)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/sinaryorg/winvideo)
[![Documentation](https://docs.rs/winvideo-core/badge.svg)](https://docs.rs/winvideo-core)

Modern, high-performance Windows screen and per-window video capture engine in pure Rust by **SINARY™**.

Powered by Windows Graphics Capture (WGC) and Direct3D 11, delivering zero-copy GPU frame extraction at 60–120+ FPS with ~1% CPU usage. Built-in hardware MP4 encoding using Windows Media Foundation (MFT).

> [!NOTE]
> WinVideo captures the visual video stream only (no audio track). To capture application audio, pair with [**WinAudio**](https://crates.io/crates/winaudio-core).

---

## Features

- 🎯 **Target Any Application or Monitor**: Target windows by executable name, PID, window title substring, or monitor index.
- ⚡ **Direct3D 11 Zero-Copy**: GPU VRAM frame extraction without GDI lag or virtual display drivers.
- 🎬 **Hardware MP4 Recording**: Encode directly to `.mp4` using NVENC, Intel QuickSync, or AMD AMF.
- 🚫 **Yellow Border Suppression**: Programmatically disable the Windows capture border (`BorderRequired = false`).
- 🖱️ **Cursor Capture Control**: Configurable cursor visibility.
- 📸 **Lossless Screenshots**: Save single frames directly to PNG, JPEG, or BMP.

---

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
winvideo-core = "0.1.0"
```

---

## Quick Example

```rust
use std::time::Duration;
use winvideo_core::{capture_screenshot, record_mp4, list_windows, CaptureConfig, Target};

fn main() -> anyhow::Result<()> {
    // 1. Enumerate capturable windows
    for w in list_windows()? {
        println!("[{}] {} - '{}'", w.pid, w.process_name, w.title);
    }

    // 2. Instant lossless screenshot
    capture_screenshot(&Target::ByName("chrome.exe".into()), "chrome.png")?;

    // 3. Record application directly to MP4
    let config = CaptureConfig::default();
    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    record_mp4(
        &Target::ByName("Spotify.exe".into()),
        "spotify.mp4",
        Some(Duration::from_secs(10)),
        &config,
        stop_signal,
    )?;

    Ok(())
}
```

---

## License

Dual-licensed under MIT or Apache-2.0. Copyright © 2026 SINARY™.
