# Rust API Guide (`winvideo-core`)

The official Rust crate **`winvideo-core`** provides direct access to the Windows Graphics Capture (WGC) and Direct3D 11 engine.

---

## 1. Installation

Add `winvideo-core` to your `Cargo.toml`:

```toml
[dependencies]
winvideo-core = "0.1.0"
```

---

## 2. Target Selection (`Target`)

The `Target` enum defines what screen or window to capture:

```rust
use winvideo_core::Target;

// Target application executable name
let t1 = Target::ByName("chrome.exe".into());

// Target specific Process ID
let t2 = Target::ByPid(15024);

// Target window title substring
let t3 = Target::ByWindow("YouTube".into());

// Target physical display monitor
let t4 = Target::ByMonitor(1); // Primary display

// Target currently focused foreground window
let t5 = Target::Foreground;

// Target primary monitor
let t6 = Target::PrimaryMonitor;
```

---

## 3. Lossless Screenshot Capture

```rust
use winvideo_core::{capture_screenshot, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = Target::ByName("chrome.exe".into());
    capture_screenshot(&target, "chrome_screenshot.png")?;
    println!("Screenshot saved successfully!");
    Ok(())
}
```

---

## 4. Hardware MP4 Video Recording

### Simple Recording (`record_mp4`):
```rust
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use winvideo_core::{record_mp4, CaptureConfig, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = Target::ByName("chrome.exe".into());
    let config = CaptureConfig::default();
    let stop_signal = Arc::new(AtomicBool::new(false));

    let stats = record_mp4(
        &target,
        "recorded_output.mp4",
        Some(Duration::from_secs(10)), // Duration or None for continuous
        &config,
        stop_signal,
    )?;

    println!("Recorded {} frames in {:.2}s (Avg FPS: {:.1})",
        stats.total_frames, stats.duration_seconds, stats.average_fps);

    Ok(())
}
```

### Recording with Progress Callback (`record_to_mp4`):
```rust
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use winvideo_core::{record_to_mp4, CaptureConfig, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = Target::PrimaryMonitor;
    let config = CaptureConfig::default();
    let stop_signal = Arc::new(AtomicBool::new(false));

    record_to_mp4(
        &target,
        "desktop.mp4",
        Some(Duration::from_secs(5)),
        &config,
        stop_signal,
        Some(|frames: u64, elapsed: f64| {
            println!("Live frames: {}, elapsed: {:.1}s", frames, elapsed);
        }),
    )?;

    Ok(())
}
```

---

## 5. Live Frame Streaming (`LiveCaptureStream`)

Stream raw BGRA frames directly through bounded crossbeam channels:

```rust
use std::time::Duration;
use winvideo_core::{CaptureConfig, LiveCaptureStream, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = Target::ByName("game.exe".into());
    let config = CaptureConfig::default();

    let mut stream = LiveCaptureStream::start(&target, &config)?;

    for _ in 0..100 {
        if let Some(frame) = stream.read_frame(Duration::from_millis(100)) {
            println!("Captured frame: {}x{} (bytes: {})", frame.width, frame.height, frame.data.len());
        }
    }

    stream.stop();
    Ok(())
}
```
