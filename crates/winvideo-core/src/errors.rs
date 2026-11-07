use thiserror::Error;

/// WinVideo core error types.
#[derive(Error, Debug)]
pub enum WinVideoError {
    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("Capture initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Frame processing error: {0}")]
    FrameError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Windows error: {0}")]
    WindowsError(String),
}
