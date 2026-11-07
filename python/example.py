"""
WinVideo Python Example
======================
Demonstrates discovering open application windows, monitors, capturing lossless screenshots,
and streaming DirectX video frames in pure Python.

Ensure winvideo.pyd is built and located in this directory:
    cargo build --release -p winvideo-python
    Copy-Item target/release/winvideo.dll -Destination python/winvideo.pyd
"""

import winvideo

def main():
    print(f"=== 🎥 WinVideo v{winvideo.version()} (by SINARY™) ===")

    print("\n--- 🖥️  Connected Display Monitors ---")
    monitors = winvideo.list_monitors()
    for m in monitors:
        primary = " (Primary)" if m.is_primary else ""
        print(f"[{m.index}] {m.name}{primary} - {m.width}x{m.height} @ {m.refresh_rate}Hz")

    print("\n--- 🪟 Capturable Application Windows ---")
    windows = winvideo.list_windows()
    print(f"Total open windows: {len(windows)}\n")
    for w in windows[:15]:
        print(f"[{w.pid:<6}] {w.process_name:<20} | {w.width}x{w.height} | '{w.title}'")

    # Example 1: Instant lossless screenshot
    # print("\n--- Taking Screenshot ---")
    # winvideo.screenshot("screenshot.png", name="chrome.exe")
    # print("Saved to screenshot.png")

    # Example 2: Record specific application to MP4 for 5 seconds (video stream only)
    # print("\n--- Recording Application to MP4 (Video Only) ---")
    # with winvideo.Capture(name="chrome.exe", fps=60) as cap:
    #     cap.record_to_mp4("chrome_record.mp4", duration_seconds=5.0)
    # print("Saved to chrome_record.mp4 (Note: for audio, record concurrently with winaudio)")

    # Example 3: Stream frames into OpenCV / NumPy
    # print("\n--- Streaming Frames ---")
    # with winvideo.Capture(name="chrome.exe", fps=60) as cap:
    #     for i in range(10):
    #         frame = cap.read_frame(timeout_ms=100)
    #         if frame:
    #             print(f"Captured frame #{i+1}: {frame.width}x{frame.height} ({len(frame.to_bytes())} bytes)")

    # Example 4: Concurrent Audio + Video Recording with WinAudio
    # import threading
    # import winaudio
    # target = "Spotify.exe"
    # t_audio = threading.Thread(target=lambda: winaudio.record(name=target, out="audio.wav", duration=5))
    # t_video = threading.Thread(target=lambda: winvideo.record(name=target, out="video.mp4", duration=5))
    # t_audio.start(); t_video.start()
    # t_audio.join(); t_video.join()
    # print("Recorded audio.wav and video.mp4 simultaneously!")

if __name__ == "__main__":
    main()
