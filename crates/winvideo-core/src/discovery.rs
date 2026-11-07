use std::path::Path;
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, MAX_PATH, RECT, TRUE};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible,
};
use windows_capture::monitor::Monitor;

/// Information about an open application window.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub pid: u32,
    pub process_name: String,
    pub title: String,
    pub is_visible: bool,
    pub width: i32,
    pub height: i32,
}

/// Information about a connected display monitor.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub device_name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub refresh_rate: u32,
}

/// Enumerate all capturable top-level application windows.
pub fn list_windows() -> Result<Vec<WindowInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let mut results: Vec<WindowInfo> = Vec::new();

    unsafe {
        windows::Win32::Foundation::SetLastError(windows::Win32::Foundation::WIN32_ERROR(0));
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut results as *mut Vec<WindowInfo> as isize),
        );
    }

    // Sort by process name, then title
    results.sort_by(|a, b| {
        a.process_name
            .to_lowercase()
            .cmp(&b.process_name.to_lowercase())
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    Ok(results)
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let results = unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };

    // Must be visible
    if !unsafe { IsWindowVisible(hwnd).as_bool() } {
        return TRUE;
    }

    // Must have a title
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return TRUE;
    }

    let mut buf = vec![0u16; (len + 1) as usize];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if copied == 0 {
        return TRUE;
    }

    let title = String::from_utf16_lossy(&buf[..copied as usize])
        .trim()
        .to_string();
    if title.is_empty() {
        return TRUE;
    }

    let mut rect = RECT::default();
    let rect_ok = unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok();
    if !rect_ok {
        return TRUE;
    }

    // Try to refine with DWM extended frame bounds if available
    let mut dwm_rect = RECT::default();
    let dwm_ok = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut dwm_rect as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
    }
    .is_ok();

    if dwm_ok && (dwm_rect.right - dwm_rect.left) > 10 && (dwm_rect.bottom - dwm_rect.top) > 10 {
        rect = dwm_rect;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 10 || height <= 10 {
        return TRUE;
    }

    // Get PID and process name
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };

    let process_name = get_process_name(pid).unwrap_or_else(|| String::from("unknown.exe"));

    results.push(WindowInfo {
        hwnd: hwnd.0 as isize,
        pid,
        process_name,
        title,
        is_visible: true,
        width,
        height,
    });

    TRUE
}

/// Query process executable name cleanly via limited query info
pub fn get_process_name(pid: u32) -> Option<String> {
    if pid == 0 {
        return Some("System".into());
    }

    let handle: HANDLE =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; MAX_PATH as usize];
    let mut size = buffer.len() as u32;

    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };

    let _ = unsafe { CloseHandle(handle) };

    if ok.is_ok() {
        let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
        let file_name = Path::new(&full_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&full_path)
            .to_string();
        Some(file_name)
    } else {
        None
    }
}

/// Enumerate all active monitors.
pub fn list_monitors() -> Result<Vec<MonitorInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let raw_monitors = Monitor::enumerate()?;
    let primary_hmon = Monitor::primary().map(|p| p.as_raw_hmonitor()).ok();
    let mut results = Vec::new();

    for (i, m) in raw_monitors.into_iter().enumerate() {
        let raw_name = m.name().unwrap_or_default();
        let device_name = m.device_name().unwrap_or_default();
        let name = if !raw_name.trim().is_empty() {
            raw_name
        } else if !device_name.trim().is_empty() {
            device_name.clone()
        } else {
            format!("Display {}", i + 1)
        };
        let width = m.width().unwrap_or(0);
        let height = m.height().unwrap_or(0);
        let is_primary = primary_hmon == Some(m.as_raw_hmonitor());
        let refresh_rate = m.refresh_rate().unwrap_or(60);

        results.push(MonitorInfo {
            index: i + 1,
            name,
            device_name,
            width,
            height,
            is_primary,
            refresh_rate,
        });
    }

    Ok(results)
}
