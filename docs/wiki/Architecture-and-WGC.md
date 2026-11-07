# Architecture & WGC Deep-Dive

This document details the internal architecture, operating system hooks, and GPU rendering pipelines that power **WinVideo**.

---

## 1. System Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│           User API / CLI / Python / Rust API           │
└───────────────────────────┬────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────┐
│                    winvideo-core                       │
├───────────────────────────┬────────────────────────────┤
│      Target Resolver      │   Windows Graphics Capture │
│ (Process Tree Walker BFS) │ (IGraphicsCaptureItemInterop│
├───────────────────────────┼────────────────────────────┤
│    Direct3D 11 Engine     │  Hardware MFT MP4 Encoder  │
│ (Zero-Copy GPU Swapchain) │   (NVENC / QuickSync / AMF)│
├───────────────────────────┼────────────────────────────┤
│    Border Suppression     │   High-Resolution Timer    │
│  (BorderRequired = false) │  (QueryPerformanceCounter) │
└───────────────────────────┴─────────────┬──────────────┘
                                          │
                        ┌─────────────────▼──────────────┐
                        │ Windows Graphics Capture (WGC) │
                        │  + Direct3D 11 Device Pool     │
                        └─────────────────┬──────────────┘
                                          │
                        ┌─────────────────▼──────────────┐
                        │  Desktop Window Manager (DWM)  │
                        │      Compositor Surface        │
                        └────────────────────────────────┘
```

---

## 2. Windows Graphics Capture (WGC)

Unlike legacy Windows capture techniques (such as GDI `BitBlt` or DirectX hook DLL injection), WinVideo uses native **Windows Graphics Capture (WGC)**:

### How it Works:
1. **Target Activation:**
   WinVideo creates a capture item from a native window handle (`HWND`) or monitor handle (`HMONITOR`) via `IGraphicsCaptureItemInterop::CreateForWindow` or `CreateForMonitor`.
2. **Direct3D 11 Device Context:**
   A hardware Direct3D 11 device is initialized using `D3D11CreateDevice`.
3. **Frame Pool Allocation:**
   A `Direct3D11CaptureFramePool` is created matching the target surface dimensions and color format (`DXGI_FORMAT_B8G8R8A8_UNORM`).
4. **Zero-Copy Swapchain:**
   Frames arrive directly in GPU VRAM. WinVideo extracts raw surfaces without GDI blit overhead or software rasterization.

---

## 3. Hardware MFT H.264 Video Encoding

WinVideo incorporates hardware video encoding via **Windows Media Foundation (MFT)**:
- Automatically detects and utilizes GPU video encoders:
  - **NVIDIA:** NVENC
  - **Intel:** Intel QuickSync Video
  - **AMD:** AMD Advanced Media Framework (AMF)
- Encodes uncompressed BGRA Direct3D textures directly into H.264 video streams packaged inside standard MP4 containers.
- Does not require external binaries, DLLs, or `ffmpeg.exe`.

---

## 4. Event-Driven Capture & Dirty Rectangles

Windows Graphics Capture is **event-driven**:
- Callbacks only dispatch when the target application repaints pixels on screen (dirty region updates).
- If a web browser or document is static, WGC pauses frame dispatching, saving significant GPU memory, CPU cycles, and disk space.
- WinVideo uses `start_free_threaded` to decouple frame ingestion from duration checks, ensuring smooth finalization on `Ctrl+C` even if the target window is idle.

---

## 5. Border Suppression Mechanics

By default, Windows Graphics Capture draws a bright yellow border around captured windows:
- On Windows 10 (Build 20348+) and Windows 11, WinVideo sets `GraphicsCaptureSession.IsBorderRequired = false`.
- This programmatically suppresses the border, providing a completely clean recording surface without visual artifacts.

---

## 6. Shared Master Clock with WinAudio

Both WinVideo and [WinAudio](https://github.com/sinaryorg/winaudio) utilize Windows **`QueryPerformanceCounter` (QPC)** as their internal reference clock:
- QPC operates at **100-nanosecond precision**.
- Video frame presentation timestamps and WASAPI audio buffer timestamps correlate directly with zero clock drift.
- This allows independent recordings of video and audio to be remuxed losslessly into a single synchronized container.
