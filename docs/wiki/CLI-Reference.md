# CLI Reference Guide (`winvideo.exe`)

The standalone command-line executable **`winvideo.exe`** provides direct hardware-accelerated video capture, recording, and screenshots without needing Python or external dependencies.

---

## 1. Discover Windows & Monitors (`winvideo list`)

List all capturable application windows and connected physical monitors.

```cmd
winvideo list
```

### Options:
- `--monitors` (or `-m`): Show only physical display monitors.
  ```cmd
  winvideo list --monitors
  ```

---

## 2. Video Recording (`winvideo record`)

Record an application window or physical monitor directly into an H.264 hardware-encoded MP4 file using Windows Media Foundation (MFT).

### Basic Syntax:
```cmd
winvideo record [TARGET_OPTIONS] [FLAGS] -o <OUTPUT.MP4>
```

### Target Selection Options:
- `--name <NAME>`: Target by executable name (e.g. `--name chrome.exe`, `--name spotify`, `--name Riff`).
- `--pid <PID>`: Target by Process ID (e.g. `--pid 15024`).
- `--window <TITLE>`: Target by window title substring (e.g. `--window "YouTube"`).
- `--monitor <INDEX>`: Target physical monitor index (e.g. `--monitor 1` for Primary display).

### Duration & Continuous Recording:
- `--duration <SECONDS>`: Record for a fixed duration in seconds:
  ```cmd
  winvideo record --name chrome.exe --duration 10 --out chrome_10s.mp4
  ```
- **Continuous Mode (Omit `--duration`)**:
  When `--duration` is omitted, `winvideo` records live continuously for 1 hour, 2 hours, or indefinitely. Press **`Ctrl + C`** at any time to gracefully finalize and save your complete MP4 file.

### Customization Flags:
- `--fps <FPS>`: Set target frame rate (default: `60`). Example for high-refresh recording:
  ```cmd
  winvideo record --monitor 1 --fps 120 --out gaming.mp4
  ```
- `--no-cursor`: Hide the mouse cursor in the recorded video:
  ```cmd
  winvideo record --name game.exe --no-cursor
  ```
- `--border`: Display the Windows yellow capture border (hidden by default).

---

## 3. Lossless Screenshot Capture (`winvideo screenshot`)

Capture an instant single-frame lossless screenshot of any window or display.

```cmd
# Screenshot of Chrome to screenshot.png
winvideo screenshot --name chrome.exe

# Screenshot of specific window title to custom file path
winvideo screenshot --window "Visual Studio Code" --out ide.png

# Screenshot of full primary display
winvideo screenshot --monitor 1 --out desktop.png
```

Supports `.png`, `.jpg` / `.jpeg`, and `.bmp` extensions automatically based on file name.

---

## 4. Runnable Commands Cheatsheet (`winvideo commands`)

Print the complete list of copy-pasteable runnable commands at any time:

```cmd
winvideo commands
```
