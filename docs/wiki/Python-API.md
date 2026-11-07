# Python API Guide (`sinary-winvideo`)

The official Python package **`sinary-winvideo`** provides native CPython bindings (via PyO3) to the high-performance Rust Direct3D 11 & Windows Graphics Capture engine.

---

## 1. Installation

```bash
pip install sinary-winvideo
```

### Import Module
In your Python code, import `winvideo`:
```python
import winvideo
```

---

## 2. Window & Monitor Discovery

### Enumerate Application Windows
```python
import winvideo

for win in winvideo.list_windows():
    print(f"[{win.pid}] {win.process_name} ({win.width}x{win.height}) - '{win.title}'")
```

**`WindowInfo` Attributes:**
- `pid`: Integer Process ID.
- `hwnd`: Integer window handle (`isize`).
- `process_name`: String executable name (e.g. `chrome.exe`).
- `title`: String window title.
- `width` / `height`: Visible client pixel dimensions (excluding DWM shadow borders).
- `is_visible`: Boolean visibility flag.

### Enumerate Display Monitors
```python
import winvideo

for mon in winvideo.list_monitors():
    primary = " (Primary)" if mon.is_primary else ""
    print(f"[{mon.index}] {mon.name}{primary} - {mon.width}x{mon.height} @ {mon.refresh_rate}Hz")
```

---

## 3. Lossless Screenshots

Capture single-frame screenshots directly:

```python
import winvideo

# Instant screenshot of an application window
winvideo.screenshot("chrome.png", name="chrome.exe")

# Instant screenshot of a display monitor
winvideo.screenshot("desktop.png", monitor=1)

# Using keyword arguments
winvideo.screenshot(out="ide.png", window="Visual Studio Code")
```

---

## 4. Live Frame Streaming for AI & OpenCV (`cap.stream()`)

Stream raw DirectX video frames directly into NumPy arrays at 60–120 FPS with minimal CPU overhead:

```python
import winvideo
import cv2

with winvideo.Capture(name="chrome.exe", cursor=False) as cap:
    for frame in cap.stream():
        # frame is a zero-copy NumPy ndarray of shape (Height, Width, 4) uint8 (BGRA)
        cv2.imshow("DirectX Stream", frame)
        if cv2.waitKey(1) == 27:  # Press ESC to exit
            break

cv2.destroyAllWindows()
```

### `Capture` Options:
- `name`: Target executable name (e.g. `"chrome.exe"`).
- `pid`: Target process ID.
- `window`: Window title substring.
- `monitor`: Monitor index (1 for Primary).
- `fps`: Target frame rate (default: `60`).
- `cursor`: Include mouse cursor (default: `True`).
- `border`: Show Windows yellow capture border (default: `False`).

---

## 5. Direct MP4 Video Recording

Record directly to hardware-encoded MP4 files:

```python
import winvideo

# One-line recording
winvideo.record(name="Spotify.exe", out="spotify.mp4", duration=10.0, fps=60)

# Or via Capture context manager
with winvideo.Capture(name="Discord.exe") as cap:
    cap.record_to_mp4("meeting.mp4", duration_seconds=30.0)
```

---

## 6. Pairing with WinAudio (Simultaneous Audio + Video)

Run both captures concurrently in separate threads with zero conflict:

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

Merge the two files losslessly in 0.2 seconds using FFmpeg:
```bash
ffmpeg -i video.mp4 -i audio.wav -c:v copy -c:a aac final.mp4
```
