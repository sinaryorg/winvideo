use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender};
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, ImageFormat, VideoEncoder,
    VideoSettingsBuilder, VideoSettingsSubType,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

use crate::errors::WinVideoError;
use crate::target::{ResolvedTarget, Target};

/// Configuration options for video capture.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Desired frame rate (default: 60).
    pub fps: u32,
    /// Whether to capture the mouse cursor (default: true).
    pub cursor: bool,
    /// Whether to draw the Windows yellow capture border (default: false).
    pub draw_border: bool,
    /// Pixel format (default: Bgra8 for universal compatibility with OpenCV/NumPy).
    pub color_format: ColorFormat,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            fps: 60,
            cursor: true,
            draw_border: false,
            color_format: ColorFormat::Bgra8,
        }
    }
}

/// A captured raw video frame.
#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp_ns: i64,
}

/// Statistics returned after recording video to disk.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecordStats {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub total_frames: u64,
    pub duration_seconds: f64,
    pub average_fps: f64,
}

// =========================================================================
// 1. Screenshot Handler
// =========================================================================

struct ScreenshotFlags {
    output_path: String,
    format: ImageFormat,
    success: Arc<AtomicBool>,
}

struct ScreenshotHandler {
    flags: ScreenshotFlags,
}

impl GraphicsCaptureApiHandler for ScreenshotHandler {
    type Flags = ScreenshotFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { flags: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        frame.save_as_image(&self.flags.output_path, self.flags.format)?;
        self.flags.success.store(true, Ordering::SeqCst);
        capture_control.stop();
        Ok(())
    }
}

// =========================================================================
// 2. MP4 Video Recording Handler
// =========================================================================

struct Mp4RecordFlags {
    output_path: String,
    fps: u32,
    bitrate: u32,
    total_frames: Arc<AtomicU64>,
    resolution: Arc<(AtomicU64, AtomicU64)>, // (width, height)
}

struct Mp4RecordHandler {
    pub encoder: Option<VideoEncoder>,
    flags: Mp4RecordFlags,
    initialized: bool,
}

impl GraphicsCaptureApiHandler for Mp4RecordHandler {
    type Flags = Mp4RecordFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            encoder: None,
            flags: ctx.flags,
            initialized: false,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let width = frame.width();
        let height = frame.height();

        if !self.initialized {
            self.flags
                .resolution
                .0
                .store(width as u64, Ordering::SeqCst);
            self.flags
                .resolution
                .1
                .store(height as u64, Ordering::SeqCst);

            let video_settings = VideoSettingsBuilder::new(width, height)
                .frame_rate(self.flags.fps)
                .bitrate(self.flags.bitrate)
                .sub_type(VideoSettingsSubType::H264);

            let audio_settings = AudioSettingsBuilder::new().disabled(true);
            let container_settings = ContainerSettingsBuilder::new();

            let encoder = VideoEncoder::new(
                video_settings,
                audio_settings,
                container_settings,
                &self.flags.output_path,
            )?;

            self.encoder = Some(encoder);
            self.initialized = true;
        }

        if let Some(enc) = self.encoder.as_mut() {
            enc.send_frame(frame)?;
            self.flags.total_frames.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        if let Some(enc) = self.encoder.take() {
            let _ = enc.finish();
        }
        Ok(())
    }
}

// =========================================================================
// 3. Live Streaming Frame Handler (for Python & Real-time AI)
// =========================================================================

struct StreamFlags {
    sender: Sender<VideoFrame>,
    stop_signal: Arc<AtomicBool>,
    temp_buf: Vec<u8>,
}

struct StreamHandler {
    flags: StreamFlags,
}

impl GraphicsCaptureApiHandler for StreamHandler {
    type Flags = StreamFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { flags: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.flags.stop_signal.load(Ordering::SeqCst) {
            capture_control.stop();
            return Ok(());
        }

        let width = frame.width();
        let height = frame.height();
        let ts_ns = frame.timestamp().map(|t| t.Duration).unwrap_or(0);

        let fb = frame.buffer()?;
        let data = fb.as_nopadding_buffer(&mut self.flags.temp_buf).to_vec();

        let vframe = VideoFrame {
            width,
            height,
            data,
            timestamp_ns: ts_ns,
        };

        if self.flags.sender.send(vframe).is_err() {
            // Receiver dropped, stop capture
            capture_control.stop();
        }

        Ok(())
    }
}

// =========================================================================
// 4. High-Level Public API
// =========================================================================

/// Captures a single lossless screenshot of the target window or monitor.
pub fn capture_screenshot<P: AsRef<Path>>(
    target: &Target,
    output_path: P,
) -> Result<(), WinVideoError> {
    let resolved = target.resolve()?;
    let path_str = output_path.as_ref().to_string_lossy().to_string();

    let ext = output_path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let format = match ext.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "bmp" => ImageFormat::Bmp,
        _ => ImageFormat::Png,
    };

    let success = Arc::new(AtomicBool::new(false));
    let flags = ScreenshotFlags {
        output_path: path_str,
        format,
        success: success.clone(),
    };

    match resolved {
        ResolvedTarget::Window(w) => {
            let settings = create_settings(w, &CaptureConfig::default(), flags);
            ScreenshotHandler::start(settings)
                .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?;
        }
        ResolvedTarget::Monitor(m) => {
            let settings = create_settings(m, &CaptureConfig::default(), flags);
            ScreenshotHandler::start(settings)
                .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?;
        }
    }

    if success.load(Ordering::SeqCst) {
        Ok(())
    } else {
        Err(WinVideoError::FrameError(
            "Failed to capture screenshot frame".into(),
        ))
    }
}

/// Records the target window or monitor directly to an MP4 video file.
pub fn record_to_mp4<P: AsRef<Path>, F>(
    target: &Target,
    output_path: P,
    duration: Option<Duration>,
    config: &CaptureConfig,
    stop_signal: Arc<AtomicBool>,
    progress_callback: Option<F>,
) -> Result<RecordStats, WinVideoError>
where
    F: Fn(u64, f64) + Send + 'static,
{
    let resolved = target.resolve()?;
    let path_str = output_path.as_ref().to_string_lossy().to_string();

    let total_frames = Arc::new(AtomicU64::new(0));
    let resolution = Arc::new((AtomicU64::new(0), AtomicU64::new(0)));
    let start_time = Instant::now();

    // Default bitrate 8 Mbps for 1080p60
    let bitrate = 8_000_000;

    let flags = Mp4RecordFlags {
        output_path: path_str.clone(),
        fps: config.fps,
        bitrate,
        total_frames: total_frames.clone(),
        resolution: resolution.clone(),
    };

    let ctrl = match resolved {
        ResolvedTarget::Window(w) => {
            let settings = create_settings(w, config, flags);
            Mp4RecordHandler::start_free_threaded(settings)
                .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?
        }
        ResolvedTarget::Monitor(m) => {
            let settings = create_settings(m, config, flags);
            Mp4RecordHandler::start_free_threaded(settings)
                .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?
        }
    };

    // Main loop: wait for duration or external stop_signal (e.g. Ctrl+C)
    while !stop_signal.load(Ordering::SeqCst) {
        if let Some(max_dur) = duration {
            if start_time.elapsed() >= max_dur {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(50));

        if let Some(cb) = &progress_callback {
            let frames = total_frames.load(Ordering::SeqCst);
            let elapsed = start_time.elapsed().as_secs_f64();
            cb(frames, elapsed);
        }
    }

    // Finalize the encoder cleanly to write MP4 trailer and header
    let callback = ctrl.callback();
    if let Some(enc) = callback.lock().encoder.take() {
        let _ = enc.finish();
    }

    // Stop the capture background thread and wait for it
    let _ = ctrl.stop();

    let elapsed = start_time.elapsed().as_secs_f64();
    let frames = total_frames.load(Ordering::SeqCst);
    let avg_fps = if elapsed > 0.0 {
        frames as f64 / elapsed
    } else {
        0.0
    };

    Ok(RecordStats {
        output_path: path_str,
        width: resolution.0.load(Ordering::SeqCst) as u32,
        height: resolution.1.load(Ordering::SeqCst) as u32,
        total_frames: frames,
        duration_seconds: elapsed,
        average_fps: avg_fps,
    })
}

/// Convenience function to record to an MP4 video file without a progress callback.
pub fn record_mp4<P: AsRef<Path>>(
    target: &Target,
    output_path: P,
    duration: Option<Duration>,
    config: &CaptureConfig,
    stop_signal: Arc<AtomicBool>,
) -> Result<RecordStats, WinVideoError> {
    record_to_mp4::<P, fn(u64, f64)>(target, output_path, duration, config, stop_signal, None)
}

/// A live streaming capture handle that yields raw frames.
pub struct LiveCaptureStream {
    receiver: Receiver<VideoFrame>,
    stop_signal: Arc<AtomicBool>,
    control: Option<CaptureControl<StreamHandler, Box<dyn std::error::Error + Send + Sync>>>,
}

impl LiveCaptureStream {
    /// Starts a live capture stream on a background thread.
    pub fn start(target: &Target, config: &CaptureConfig) -> Result<Self, WinVideoError> {
        let resolved = target.resolve()?;
        let (sender, receiver) = bounded(16);
        let stop_signal = Arc::new(AtomicBool::new(false));

        let flags = StreamFlags {
            sender,
            stop_signal: stop_signal.clone(),
            temp_buf: Vec::new(),
        };

        let control = match resolved {
            ResolvedTarget::Window(w) => {
                let settings = create_settings(w, config, flags);
                StreamHandler::start_free_threaded(settings)
                    .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?
            }
            ResolvedTarget::Monitor(m) => {
                let settings = create_settings(m, config, flags);
                StreamHandler::start_free_threaded(settings)
                    .map_err(|e| WinVideoError::InitializationFailed(e.to_string()))?
            }
        };

        Ok(Self {
            receiver,
            stop_signal,
            control: Some(control),
        })
    }

    /// Reads the next available video frame with a timeout.
    pub fn read_frame(&self, timeout: Duration) -> Option<VideoFrame> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// Iterator over incoming frames.
    pub fn frames(&self) -> impl Iterator<Item = VideoFrame> + '_ {
        self.receiver.iter()
    }

    /// Stops the capture session.
    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(ctrl) = self.control.take() {
            let _ = ctrl.stop();
        }
    }
}

impl Drop for LiveCaptureStream {
    fn drop(&mut self) {
        self.stop();
    }
}

// =========================================================================
// Helpers
// =========================================================================

fn create_settings<T, Flags>(item: T, config: &CaptureConfig, flags: Flags) -> Settings<Flags, T>
where
    T: TryInto<windows_capture::settings::GraphicsCaptureItemType>,
{
    let cursor_settings = if config.cursor {
        CursorCaptureSettings::WithCursor
    } else {
        CursorCaptureSettings::WithoutCursor
    };

    let border_settings = if config.draw_border {
        DrawBorderSettings::WithBorder
    } else {
        DrawBorderSettings::WithoutBorder
    };

    let interval = match 1000u64.checked_div(config.fps as u64) {
        Some(ms) => MinimumUpdateIntervalSettings::Custom(Duration::from_millis(ms)),
        None => MinimumUpdateIntervalSettings::Default,
    };

    Settings::new(
        item,
        cursor_settings,
        border_settings,
        SecondaryWindowSettings::Default,
        interval,
        DirtyRegionSettings::Default,
        config.color_format,
        flags,
    )
}
