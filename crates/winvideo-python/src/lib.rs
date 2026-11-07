#![allow(clippy::useless_conversion, clippy::too_many_arguments)]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use winvideo_core::{
    capture_screenshot, record_to_mp4, CaptureConfig, ColorFormat, LiveCaptureStream,
    MonitorInfo as CoreMonitor, Target, WindowInfo as CoreWindow,
};

#[pyclass(name = "WindowInfo")]
#[derive(Clone)]
pub struct PyWindowInfo {
    #[pyo3(get)]
    pub hwnd: isize,
    #[pyo3(get)]
    pub pid: u32,
    #[pyo3(get)]
    pub process_name: String,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub is_visible: bool,
    #[pyo3(get)]
    pub width: i32,
    #[pyo3(get)]
    pub height: i32,
}

#[pymethods]
impl PyWindowInfo {
    fn __repr__(&self) -> String {
        format!(
            "<WindowInfo pid={} app='{}' title='{}' size={}x{} visible={}>",
            self.pid, self.process_name, self.title, self.width, self.height, self.is_visible
        )
    }
}

impl From<CoreWindow> for PyWindowInfo {
    fn from(w: CoreWindow) -> Self {
        Self {
            hwnd: w.hwnd,
            pid: w.pid,
            process_name: w.process_name,
            title: w.title,
            is_visible: w.is_visible,
            width: w.width,
            height: w.height,
        }
    }
}

#[pyclass(name = "MonitorInfo")]
#[derive(Clone)]
pub struct PyMonitorInfo {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub device_name: String,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub is_primary: bool,
    #[pyo3(get)]
    pub refresh_rate: u32,
}

#[pymethods]
impl PyMonitorInfo {
    fn __repr__(&self) -> String {
        format!(
            "<MonitorInfo index={} name='{}' res={}x{} primary={} fps={}Hz>",
            self.index, self.name, self.width, self.height, self.is_primary, self.refresh_rate
        )
    }
}

impl From<CoreMonitor> for PyMonitorInfo {
    fn from(m: CoreMonitor) -> Self {
        Self {
            index: m.index,
            name: m.name,
            device_name: m.device_name,
            width: m.width,
            height: m.height,
            is_primary: m.is_primary,
            refresh_rate: m.refresh_rate,
        }
    }
}

#[pyclass(name = "VideoFrame")]
#[derive(Clone)]
pub struct PyVideoFrame {
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub timestamp_ns: i64,
    data: Vec<u8>,
}

#[pymethods]
impl PyVideoFrame {
    /// Return the raw frame pixel bytes (BGRA/RGBA).
    pub fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new_bound(py, &self.data)
    }

    /// Converts the frame into a NumPy uint8 array of shape (height, width, 4).
    pub fn to_numpy<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let np = py.import_bound("numpy")?;
        let bytes_obj = PyBytes::new_bound(py, &self.data);
        let frombuffer = np.getattr("frombuffer")?;
        let uint8 = np.getattr("uint8")?;
        let flat_arr = frombuffer.call1((bytes_obj, uint8))?;
        let shape = (self.height, self.width, 4);
        let reshape = flat_arr.getattr("reshape")?;
        reshape.call1((shape,))
    }

    fn __repr__(&self) -> String {
        format!(
            "<VideoFrame {}x{} bytes={}>",
            self.width,
            self.height,
            self.data.len()
        )
    }
}

#[pyclass(name = "Capture")]
pub struct PyCapture {
    target: Target,
    config: CaptureConfig,
    stream: Option<LiveCaptureStream>,
}

#[pymethods]
impl PyCapture {
    #[new]
    #[pyo3(signature = (name=None, pid=None, window=None, monitor=None, fps=60, cursor=true, border=false))]
    pub fn new(
        name: Option<String>,
        pid: Option<u32>,
        window: Option<String>,
        monitor: Option<usize>,
        fps: u32,
        cursor: bool,
        border: bool,
    ) -> PyResult<Self> {
        let target = if let Some(n) = name {
            Target::ByName(n)
        } else if let Some(p) = pid {
            Target::ByPid(p)
        } else if let Some(w) = window {
            Target::ByWindow(w)
        } else if let Some(m) = monitor {
            Target::ByMonitor(m)
        } else {
            Target::PrimaryMonitor
        };

        let config = CaptureConfig {
            fps,
            cursor,
            draw_border: border,
            color_format: ColorFormat::Bgra8,
        };

        Ok(Self {
            target,
            config,
            stream: None,
        })
    }

    /// Starts live streaming frames.
    pub fn start(&mut self) -> PyResult<()> {
        if self.stream.is_none() {
            let stream = LiveCaptureStream::start(&self.target, &self.config)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            self.stream = Some(stream);
        }
        Ok(())
    }

    /// Reads the next frame with a timeout in milliseconds.
    #[pyo3(signature = (timeout_ms=100))]
    pub fn read_frame(&mut self, timeout_ms: u64) -> PyResult<Option<PyVideoFrame>> {
        if self.stream.is_none() {
            self.start()?;
        }

        if let Some(s) = &self.stream {
            let f = s.read_frame(Duration::from_millis(timeout_ms));
            Ok(f.map(|frame| PyVideoFrame {
                width: frame.width,
                height: frame.height,
                timestamp_ns: frame.timestamp_ns,
                data: frame.data,
            }))
        } else {
            Ok(None)
        }
    }

    /// Record directly to an MP4 video file (video frames only; no audio track).
    /// To record audio, pair with `winaudio`.
    #[pyo3(signature = (path, duration_seconds=None))]
    pub fn record_to_mp4(&self, path: &str, duration_seconds: Option<f64>) -> PyResult<()> {
        let max_dur = duration_seconds.map(Duration::from_secs_f64);
        let stop_signal = Arc::new(AtomicBool::new(false));

        record_to_mp4::<_, fn(u64, f64)>(
            &self.target,
            path,
            max_dur,
            &self.config,
            stop_signal,
            None,
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(())
    }

    /// Stream frames continuously as zero-copy NumPy arrays.
    #[pyo3(signature = (timeout_ms=100))]
    pub fn stream(slf: Py<Self>, py: Python<'_>, timeout_ms: u64) -> PyResult<PyFrameStream> {
        {
            let mut cap = slf.borrow_mut(py);
            if cap.stream.is_none() {
                cap.start()?;
            }
        }
        Ok(PyFrameStream {
            capture: slf,
            timeout_ms,
        })
    }

    fn __iter__(slf: Py<Self>, py: Python<'_>) -> PyResult<PyFrameStream> {
        Self::stream(slf, py, 100)
    }

    /// Stops the capture stream.
    pub fn stop(&mut self) {
        if let Some(mut s) = self.stream.take() {
            s.stop();
        }
    }

    fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        slf.start()?;
        Ok(slf)
    }

    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<Bound<'_, PyAny>>,
        _exc_val: Option<Bound<'_, PyAny>>,
        _exc_tb: Option<Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.stop();
        Ok(false)
    }
}

/// An iterator yielding live video frames as NumPy arrays.
#[pyclass(name = "FrameStream")]
pub struct PyFrameStream {
    capture: Py<PyCapture>,
    timeout_ms: u64,
}

#[pymethods]
impl PyFrameStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let mut cap = self.capture.borrow_mut(py);
        match cap.read_frame(self.timeout_ms)? {
            Some(frame) => {
                let np_arr = frame.to_numpy(py)?;
                Ok(Some(np_arr))
            }
            None => Ok(None),
        }
    }
}

/// Enumerate all capturable application windows.
#[pyfunction]
pub fn list_windows() -> PyResult<Vec<PyWindowInfo>> {
    let windows =
        winvideo_core::list_windows().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(windows.into_iter().map(PyWindowInfo::from).collect())
}

/// Enumerate all connected display monitors.
#[pyfunction]
pub fn list_monitors() -> PyResult<Vec<PyMonitorInfo>> {
    let monitors =
        winvideo_core::list_monitors().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(monitors.into_iter().map(PyMonitorInfo::from).collect())
}

/// Capture an instant lossless screenshot to PNG, JPG, or BMP.
#[pyfunction]
#[pyo3(signature = (path="screenshot.png", out=None, name=None, pid=None, window=None, monitor=None))]
pub fn screenshot(
    path: &str,
    out: Option<String>,
    name: Option<String>,
    pid: Option<u32>,
    window: Option<String>,
    monitor: Option<usize>,
) -> PyResult<()> {
    let target = if let Some(n) = name {
        Target::ByName(n)
    } else if let Some(p) = pid {
        Target::ByPid(p)
    } else if let Some(w) = window {
        Target::ByWindow(w)
    } else if let Some(m) = monitor {
        Target::ByMonitor(m)
    } else {
        Target::PrimaryMonitor
    };

    let target_path = out.as_deref().unwrap_or(path);
    capture_screenshot(&target, target_path).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Record application window or display monitor directly to an MP4 video file.
#[pyfunction]
#[pyo3(signature = (out="recorded_video.mp4", name=None, pid=None, window=None, monitor=None, duration=None, fps=60, cursor=true, border=false))]
pub fn record(
    out: &str,
    name: Option<String>,
    pid: Option<u32>,
    window: Option<String>,
    monitor: Option<usize>,
    duration: Option<f64>,
    fps: u32,
    cursor: bool,
    border: bool,
) -> PyResult<()> {
    let target = if let Some(n) = name {
        Target::ByName(n)
    } else if let Some(p) = pid {
        Target::ByPid(p)
    } else if let Some(w) = window {
        Target::ByWindow(w)
    } else if let Some(m) = monitor {
        Target::ByMonitor(m)
    } else {
        Target::PrimaryMonitor
    };

    let config = CaptureConfig {
        fps,
        cursor,
        draw_border: border,
        color_format: ColorFormat::Bgra8,
    };

    let max_dur = duration.map(Duration::from_secs_f64);
    let stop_signal = Arc::new(AtomicBool::new(false));

    record_to_mp4::<_, fn(u64, f64)>(&target, out, max_dur, &config, stop_signal, None)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Ok(())
}

/// Version of WinVideo.
#[pyfunction]
pub fn version() -> &'static str {
    winvideo_core::version()
}

/// The WinVideo native Python extension module by SINARY™.
#[pymodule]
fn winvideo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(list_windows, m)?)?;
    m.add_function(wrap_pyfunction!(list_monitors, m)?)?;
    m.add_function(wrap_pyfunction!(screenshot, m)?)?;
    m.add_function(wrap_pyfunction!(record, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_class::<PyWindowInfo>()?;
    m.add_class::<PyMonitorInfo>()?;
    m.add_class::<PyVideoFrame>()?;
    m.add_class::<PyCapture>()?;
    m.add_class::<PyFrameStream>()?;
    Ok(())
}
