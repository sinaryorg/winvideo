use std::collections::HashMap;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_capture::monitor::Monitor;
use windows_capture::window::Window;

use crate::discovery::list_windows;
use crate::errors::WinVideoError;

/// Defines what application or screen to capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Target by executable name (e.g. "chrome.exe", "spotify", "Riff").
    ByName(String),
    /// Target by Process ID.
    ByPid(u32),
    /// Target by substring of window title (e.g. "YouTube", "Discord").
    ByWindow(String),
    /// Target by monitor index (1-based or 0-based).
    ByMonitor(usize),
    /// Target primary display monitor.
    PrimaryMonitor,
    /// Target the currently focused / active foreground window.
    Foreground,
}

/// A target resolved to a concrete capturable item.
pub enum ResolvedTarget {
    Window(Window),
    Monitor(Monitor),
}

impl Target {
    /// Resolves this target specification into a concrete Windows capture item.
    pub fn resolve(&self) -> Result<ResolvedTarget, WinVideoError> {
        match self {
            Target::Foreground => {
                let win = Window::foreground()
                    .map_err(|e| WinVideoError::TargetNotFound(e.to_string()))?;
                Ok(ResolvedTarget::Window(win))
            }
            Target::PrimaryMonitor => {
                let mon =
                    Monitor::primary().map_err(|e| WinVideoError::TargetNotFound(e.to_string()))?;
                Ok(ResolvedTarget::Monitor(mon))
            }
            Target::ByMonitor(index) => {
                let idx = if *index == 0 { 1 } else { *index };
                let mon = Monitor::from_index(idx).map_err(|e| {
                    WinVideoError::TargetNotFound(format!("Monitor {}: {}", index, e))
                })?;
                Ok(ResolvedTarget::Monitor(mon))
            }
            Target::ByWindow(title_query) => {
                let query = title_query.to_lowercase();
                let windows =
                    list_windows().map_err(|e| WinVideoError::TargetNotFound(e.to_string()))?;

                for w in windows {
                    if w.title.to_lowercase().contains(&query) {
                        let win = Window::from_raw_hwnd(w.hwnd as *mut _);
                        return Ok(ResolvedTarget::Window(win));
                    }
                }
                Err(WinVideoError::TargetNotFound(format!(
                    "No visible window found matching title query: '{}'",
                    title_query
                )))
            }
            Target::ByPid(target_pid) => {
                let pids = get_process_tree_pids(*target_pid);
                let windows =
                    list_windows().map_err(|e| WinVideoError::TargetNotFound(e.to_string()))?;

                for w in windows {
                    if pids.contains(&w.pid) {
                        let win = Window::from_raw_hwnd(w.hwnd as *mut _);
                        return Ok(ResolvedTarget::Window(win));
                    }
                }
                Err(WinVideoError::TargetNotFound(format!(
                    "No visible window found for PID {} (or its child processes)",
                    target_pid
                )))
            }
            Target::ByName(name_query) => {
                let clean_query = name_query.trim().to_lowercase();
                let strip_exe = clean_query.strip_suffix(".exe").unwrap_or(&clean_query);

                let windows =
                    list_windows().map_err(|e| WinVideoError::TargetNotFound(e.to_string()))?;

                // Priority 1: Exact process name match (e.g. "chrome.exe")
                for w in &windows {
                    let clean_pname = w.process_name.to_lowercase();
                    let p_strip = clean_pname.strip_suffix(".exe").unwrap_or(&clean_pname);
                    if clean_pname == clean_query || p_strip == strip_exe {
                        let win = Window::from_raw_hwnd(w.hwnd as *mut _);
                        return Ok(ResolvedTarget::Window(win));
                    }
                }

                // Priority 2: Process name contains query or window title contains query
                for w in &windows {
                    let matches_proc = w.process_name.to_lowercase().contains(strip_exe);
                    let matches_title = w.title.to_lowercase().contains(strip_exe);

                    if matches_proc || matches_title {
                        let win = Window::from_raw_hwnd(w.hwnd as *mut _);
                        return Ok(ResolvedTarget::Window(win));
                    }
                }

                Err(WinVideoError::TargetNotFound(format!(
                    "No application window found matching name '{}'",
                    name_query
                )))
            }
        }
    }
}

/// Discovers parent-child process relationships and returns the target PID plus all descendants.
fn get_process_tree_pids(root_pid: u32) -> Vec<u32> {
    let mut tree_pids = vec![root_pid];
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if let Ok(handle) = snapshot {
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut parent_to_children: HashMap<u32, Vec<u32>> = HashMap::new();

        unsafe {
            if Process32FirstW(handle, &mut entry).is_ok() {
                loop {
                    let pid = entry.th32ProcessID;
                    let parent_pid = entry.th32ParentProcessID;
                    parent_to_children.entry(parent_pid).or_default().push(pid);

                    if Process32NextW(handle, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = windows::Win32::Foundation::CloseHandle(handle);
        }

        // BFS to collect all descendants
        let mut queue = vec![root_pid];
        while let Some(current) = queue.pop() {
            if let Some(children) = parent_to_children.get(&current) {
                for &child in children {
                    if !tree_pids.contains(&child) {
                        tree_pids.push(child);
                        queue.push(child);
                    }
                }
            }
        }
    }
    tree_pids
}
