# 🔒 Privacy Policy

**Effective Date:** November 7, 2026  
**Product:** WinVideo  
**Trademark:** SINARY™  

---

## 1. Overview & Commitment

**WinVideo** is an open-source, local-first per-window and screen video capture utility provided by **SINARY™**.

Your privacy is paramount. WinVideo is built with a strict **zero-telemetry, zero-cloud, local-only** design philosophy. We do not collect, store, transmit, or monetize any of your video data, process information, screen contents, or personal identifiers.

---

## 2. Local-Only Processing (No Cloud / No Network Transmission)

- **100% Offline Execution:** WinVideo operates entirely offline on your local Windows system.
- **No Network Requests:** WinVideo contains zero network calls, telemetry libraries, tracking beacons, analytics scripts, or remote logging services.
- **Video Stored Locally Only:** Video recordings created using `winvideo record` or the Python API are saved strictly to the file path you specify on your local drive (e.g. `recorded_video.mp4`). At no point is video data uploaded or streamed over the internet.

---

## 3. Scope of Video Capture

- **Desktop & Window Graphics Capture Only:** WinVideo exclusively captures digital frames rendered to Windows displays or DirectX surfaces by user applications (via native Windows Graphics Capture).
- **No Webcam Access:** WinVideo does **not** access, activate, or record webcams, front/back cameras, or physical video input devices.
- **No Microphone or Audio Capture:** WinVideo does **not** access, activate, or record microphones, audio endpoints, or internal sound streams. For audio recording, use [WinAudio](https://github.com/sinaryorg/winaudio).
- **Target Isolation:** WinVideo isolates only the application window(s) or monitor you explicitly target. Unrelated applications and background windows can be ignored based on your parameters.

---

## 4. System Inspection & Memory

To allow you to target applications by name, PID, or window title, WinVideo queries Windows system APIs (`CreateToolhelp32Snapshot`, `EnumWindows`, and `GetWindowThreadProcessId`).
- This information is processed **strictly in volatile system memory (RAM)**.
- It is used solely to match your specified target application and resolve its window handles (`HWND`).
- None of this process information is logged to persistent disk files or transmitted externally.

---

## 5. Security & Transparency

- **User-Mode Only:** WinVideo runs entirely in standard Windows user-mode without requiring kernel drivers or elevated administrator privileges (unless targeting an elevated process).
- **Open Source:** The complete source code of WinVideo is publicly auditable and inspectable in this repository.

---

## 6. Trademark & Copyright

**SINARY™** and the SINARY logo are trademarks of SINARY. All rights reserved.
