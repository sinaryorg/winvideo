//! # WinVideo Core
//!
//! Modern hardware-accelerated Windows screen and window video capture engine by **SINARY™**.
//!
//! WinVideo focuses exclusively on ultra-fast, zero-copy **visual capture** (GPU frame extraction,
//! screen recording, and lossless screenshots). The recorded MP4 files contain a video stream only
//! without an audio track. For audio recording, pair with [`winaudio`](https://github.com/sinaryorg/winaudio).
//!
//! Features:
//! - Per-window and per-process video capture via Windows Graphics Capture (WGC).
//! - Direct3D 11 hardware-accelerated zero-copy frame pipelines (60–120+ FPS).
//! - Direct hardware MP4 video recording using Windows Media Foundation (MFT).
//! - Border suppression (hiding the Windows 10/11 yellow capture border).
//! - Dynamic process-tree matching (targeting by PID, window title, or executable name).
//! - High-speed lossless screenshot capture (PNG, JPEG, BMP).

pub mod capture;
pub mod discovery;
pub mod errors;
pub mod target;

// Re-exports for convenience
pub use capture::{
    capture_screenshot, record_mp4, record_to_mp4, CaptureConfig, LiveCaptureStream, RecordStats,
    VideoFrame,
};
pub use discovery::{list_monitors, list_windows, MonitorInfo, WindowInfo};
pub use errors::WinVideoError;
pub use target::{ResolvedTarget, Target};
pub use windows_capture::encoder::ImageFormat;
pub use windows_capture::settings::ColorFormat;

/// Current WinVideo version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.fps, 60);
        assert!(config.cursor);
        assert!(!config.draw_border);
        assert_eq!(config.color_format, ColorFormat::Bgra8);
    }

    #[test]
    fn test_target_variants() {
        let t1 = Target::ByName("chrome.exe".into());
        let t2 = Target::ByPid(1234);
        let t3 = Target::ByWindow("Notepad".into());
        let t4 = Target::ByMonitor(1);
        let t5 = Target::PrimaryMonitor;
        let t6 = Target::Foreground;

        assert_eq!(t1, Target::ByName("chrome.exe".into()));
        assert_ne!(t1, t2);
        assert_eq!(t3, Target::ByWindow("Notepad".into()));
        assert_eq!(t4, Target::ByMonitor(1));
        assert_eq!(t5, Target::PrimaryMonitor);
        assert_eq!(t6, Target::Foreground);
    }

    #[test]
    fn test_list_monitors() {
        let monitors = list_monitors().expect("Should enumerate monitors on Windows");
        assert!(
            !monitors.is_empty(),
            "Should have at least 1 connected monitor"
        );
        let primary = monitors.iter().find(|m| m.is_primary);
        assert!(primary.is_some(), "Should find a primary monitor");
    }

    #[test]
    fn test_list_windows() {
        let windows = list_windows().expect("Should enumerate windows");
        assert!(windows.iter().all(|w| !w.title.is_empty()));
    }
}
