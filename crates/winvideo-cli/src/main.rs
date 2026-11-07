#![allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::print_literal
)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use winvideo_core::{
    capture_screenshot, list_monitors, list_windows, record_to_mp4, CaptureConfig, ColorFormat,
    Target,
};

#[derive(Parser)]
#[command(
    name = "winvideo",
    about = "🎥 Modern hardware-accelerated Windows screen and window video capture by SINARY™",
    version = "0.1.0",
    after_help = "Run 'winvideo commands' to display all runnable command lines and cheatsheet."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all capturable application windows and connected monitors
    List {
        /// Show only monitors
        #[arg(short, long)]
        monitors: bool,
    },

    /// Record application window or monitor directly to MP4 (video stream only; no audio)
    Record {
        /// Target application executable name (e.g. chrome.exe, spotify.exe, Riff)
        #[arg(short, long)]
        name: Option<String>,

        /// Target process ID
        #[arg(short, long)]
        pid: Option<u32>,

        /// Target window title substring
        #[arg(short, long)]
        window: Option<String>,

        /// Target monitor index (1 for Primary, 2 for Secondary, etc.)
        #[arg(short, long)]
        monitor: Option<usize>,

        /// Output file path (default: recorded_video.mp4)
        #[arg(short, long, default_value = "recorded_video.mp4")]
        out: PathBuf,

        /// Recording duration in seconds (optional; press Ctrl+C anytime to stop and save)
        #[arg(short, long)]
        duration: Option<f64>,

        /// Target frame rate in FPS (default: 60)
        #[arg(long, default_value = "60")]
        fps: u32,

        /// Hide the mouse cursor in the video
        #[arg(long)]
        no_cursor: bool,

        /// Show the Windows yellow capture border (default: hidden)
        #[arg(long)]
        border: bool,
    },

    /// Capture an instant lossless screenshot to PNG, JPG, or BMP
    Screenshot {
        /// Target application executable name
        #[arg(short, long)]
        name: Option<String>,

        /// Target process ID
        #[arg(short, long)]
        pid: Option<u32>,

        /// Target window title substring
        #[arg(short, long)]
        window: Option<String>,

        /// Target monitor index
        #[arg(short, long)]
        monitor: Option<usize>,

        /// Output image file path (default: screenshot.png)
        #[arg(short, long, default_value = "screenshot.png")]
        out: PathBuf,
    },

    /// Display all runnable command lines, cheatsheet, and examples
    #[command(alias = "cheatsheet", alias = "examples", alias = "guide")]
    Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List { monitors }) => handle_list(monitors)?,
        Some(Commands::Record {
            name,
            pid,
            window,
            monitor,
            out,
            duration,
            fps,
            no_cursor,
            border,
        }) => handle_record(
            name, pid, window, monitor, out, duration, fps, no_cursor, border,
        )?,
        Some(Commands::Screenshot {
            name,
            pid,
            window,
            monitor,
            out,
        }) => handle_screenshot(name, pid, window, monitor, out)?,
        Some(Commands::Commands) => print_cheatsheet(),
        None => {
            println!(
                "{}",
                "🎥 WinVideo — Hardware-Accelerated Windows Video Capture"
                    .cyan()
                    .bold()
            );
            println!("Version: {} (by SINARY™)\n", winvideo_core::version());
            println!("Usage: winvideo <COMMAND>\n");
            println!("Quick start:");
            println!(
                "  {}     List capturable windows & monitors",
                "winvideo list".green()
            );
            println!(
                "  {}   Record app to MP4",
                "winvideo record --name chrome.exe".green()
            );
            println!(
                "  {}   Instant lossless screenshot",
                "winvideo screenshot --out screen.png".green()
            );
            println!(
                "  {} Check all runnable commands",
                "winvideo commands".yellow()
            );
        }
    }

    Ok(())
}

fn determine_target(
    name: Option<String>,
    pid: Option<u32>,
    window: Option<String>,
    monitor: Option<usize>,
) -> Target {
    if let Some(n) = name {
        Target::ByName(n)
    } else if let Some(p) = pid {
        Target::ByPid(p)
    } else if let Some(w) = window {
        Target::ByWindow(w)
    } else if let Some(m) = monitor {
        Target::ByMonitor(m)
    } else {
        Target::PrimaryMonitor
    }
}

fn handle_list(only_monitors: bool) -> Result<()> {
    if !only_monitors {
        println!("\n{}", "🎥 CAPTURABLE APPLICATION WINDOWS".cyan().bold());
        println!("{}", "─".repeat(95).bright_black());
        println!(
            "{:<8} {:<24} {:<38} {:<12} {}",
            "PID".bold(),
            "APPLICATION".bold(),
            "WINDOW TITLE".bold(),
            "DIMENSIONS".bold(),
            "VISIBLE".bold()
        );
        println!("{}", "─".repeat(95).bright_black());

        let windows = list_windows().map_err(|e| anyhow::anyhow!("{}", e))?;
        for w in &windows {
            let vis_str = if w.is_visible {
                "YES".green()
            } else {
                "NO".bright_black()
            };
            let title_trunc = if w.title.chars().count() > 36 {
                format!("{}...", w.title.chars().take(33).collect::<String>())
            } else {
                w.title.clone()
            };
            let proc_trunc = if w.process_name.chars().count() > 22 {
                format!("{}...", w.process_name.chars().take(19).collect::<String>())
            } else {
                w.process_name.clone()
            };

            println!(
                "{:<8} {:<24} {:<38} {:<12} {}",
                w.pid.to_string().yellow(),
                proc_trunc.white(),
                title_trunc.bright_white(),
                format!("{}x{}", w.width, w.height).cyan(),
                vis_str
            );
        }
        println!("{}", "─".repeat(95).bright_black());
        println!(
            "Total capturable application windows: {}\n",
            windows.len().to_string().bold()
        );
    }

    println!("{}", "🖥️  CONNECTED DISPLAY MONITORS".cyan().bold());
    println!("{}", "─".repeat(85).bright_black());
    println!(
        "{:<8} {:<24} {:<20} {:<14} {}",
        "INDEX".bold(),
        "MONITOR NAME".bold(),
        "RESOLUTION".bold(),
        "REFRESH RATE".bold(),
        "PRIMARY".bold()
    );
    println!("{}", "─".repeat(85).bright_black());

    let monitors = list_monitors().map_err(|e| anyhow::anyhow!("{}", e))?;
    for m in &monitors {
        let prim_str = if m.is_primary {
            "YES (Default)".green().bold()
        } else {
            "NO".bright_black()
        };
        println!(
            "{:<8} {:<24} {:<20} {:<14} {}",
            m.index.to_string().yellow(),
            m.name.white(),
            format!("{}x{}", m.width, m.height).cyan(),
            format!("{} Hz", m.refresh_rate).bright_white(),
            prim_str
        );
    }
    println!("{}", "─".repeat(85).bright_black());
    println!(
        "Total connected monitors: {}\n",
        monitors.len().to_string().bold()
    );

    Ok(())
}

fn handle_screenshot(
    name: Option<String>,
    pid: Option<u32>,
    window: Option<String>,
    monitor: Option<usize>,
    out: PathBuf,
) -> Result<()> {
    let target = determine_target(name, pid, window, monitor);
    println!(
        "📸 Capturing lossless screenshot to '{}'...",
        out.display().to_string().cyan()
    );

    capture_screenshot(&target, &out)?;
    println!(
        "{} Screenshot saved successfully to '{}'",
        "✔".green().bold(),
        out.display().to_string().bold()
    );
    Ok(())
}

fn handle_record(
    name: Option<String>,
    pid: Option<u32>,
    window: Option<String>,
    monitor: Option<usize>,
    out: PathBuf,
    duration: Option<f64>,
    fps: u32,
    no_cursor: bool,
    border: bool,
) -> Result<()> {
    let target = determine_target(name, pid, window, monitor);
    let out_str = out.display().to_string();

    let target_desc = match &target {
        Target::ByName(n) => format!("Application: '{}'", n.yellow()),
        Target::ByPid(p) => format!("PID: {}", p.to_string().yellow()),
        Target::ByWindow(w) => format!("Window title: '{}'", w.yellow()),
        Target::ByMonitor(m) => format!("Monitor #{}", m.to_string().yellow()),
        Target::PrimaryMonitor => "Primary Display Monitor".yellow().to_string(),
        Target::Foreground => "Foreground Active Window".yellow().to_string(),
    };

    println!("\n{}", "🎥 STARTING HARDWARE VIDEO CAPTURE".cyan().bold());
    println!("  Target:     {}", target_desc);
    println!("  Output:     {}", out_str.bold());
    println!("  Framerate:  {} FPS", fps.to_string().cyan());
    println!(
        "  Stream:     {}",
        "Video Only (No audio track. Pair with WinAudio for sound)".bright_black()
    );
    println!(
        "  Cursor:     {}",
        if no_cursor {
            "Hidden".bright_black()
        } else {
            "Visible".green()
        }
    );
    println!(
        "  Border:     {}",
        if border {
            "Visible".yellow()
        } else {
            "Disabled (Clean)".green()
        }
    );

    if let Some(d) = duration {
        println!("  Duration:   {} seconds", d.to_string().bold());
    } else {
        println!(
            "  Duration:   Continuous (Press {} anytime to stop and save)",
            "Ctrl+C".red().bold()
        );
    }
    println!();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_clone = stop_signal.clone();

    // Set up Ctrl+C trap
    ctrlc::set_handler(move || {
        println!(
            "\n\n{} Ctrl+C received! Finalizing MP4 container cleanly...",
            "■".yellow().bold()
        );
        stop_clone.store(true, Ordering::SeqCst);
    })?;

    let config = CaptureConfig {
        fps,
        cursor: !no_cursor,
        draw_border: border,
        color_format: ColorFormat::Bgra8,
    };

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message("Capturing DirectX frames...");

    let max_dur = duration.map(Duration::from_secs_f64);

    let pb_clone = pb.clone();
    let stats = record_to_mp4(
        &target,
        &out,
        max_dur,
        &config,
        stop_signal,
        Some(move |frames: u64, elapsed: f64| {
            let fps_current = if elapsed > 0.0 {
                frames as f64 / elapsed
            } else {
                0.0
            };
            pb_clone.set_message(format!(
                "Recording: {} frames ({:.1}s) | Avg: {:.1} FPS",
                frames.to_string().cyan(),
                elapsed,
                fps_current
            ));
        }),
    )?;

    pb.finish_and_clear();

    println!("\n{}", "✔ VIDEO RECORDING COMPLETE".green().bold());
    println!("{}", "─".repeat(50).bright_black());
    println!("  File:        {}", stats.output_path.bold());
    println!(
        "  Resolution:  {}x{} pixels",
        stats.width.to_string().cyan(),
        stats.height.to_string().cyan()
    );
    println!(
        "  Frames:      {} frames",
        stats.total_frames.to_string().cyan()
    );
    println!("  Duration:    {:.2} seconds", stats.duration_seconds);
    println!(
        "  Average FPS: {} FPS",
        format!("{:.1}", stats.average_fps).green().bold()
    );
    println!("{}", "─".repeat(50).bright_black());
    println!("MP4 file saved and ready for playback.\n");

    Ok(())
}

fn print_cheatsheet() {
    println!(
        "\n{}",
        "🎥 WinVideo — Runnable Commands Cheatsheet".cyan().bold()
    );
    println!("{}", "═".repeat(88).bright_black());

    println!(
        "\n{}",
        "1. DISCOVER CAPTURABLE WINDOWS & MONITORS".yellow().bold()
    );
    println!(
        "  {:<48} {}",
        "winvideo list".green(),
        "List all open application windows and monitors"
    );
    println!(
        "  {:<48} {}",
        "winvideo list --monitors".green(),
        "List only physical display monitors"
    );

    println!(
        "\n{}",
        "2. SINGLE APPLICATION WINDOW VIDEO RECORDING"
            .yellow()
            .bold()
    );
    println!(
        "  {:<48} {}",
        "winvideo record --name chrome.exe".green(),
        "Record Chrome window continuously (Ctrl+C to stop)"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --name spotify --duration 10".green(),
        "Record Spotify for 10 seconds"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --pid 15024 --out app.mp4".green(),
        "Record specific process by its PID"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --window \"YouTube\" --duration 15".green(),
        "Record window matching title substring"
    );

    println!(
        "\n{}",
        "3. FULL DESKTOP & DISPLAY MONITOR RECORDING"
            .yellow()
            .bold()
    );
    println!(
        "  {:<48} {}",
        "winvideo record --monitor 1 --out desktop.mp4".green(),
        "Record primary monitor at native 60 FPS"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --monitor 2 --fps 120".green(),
        "Record secondary monitor at 120 FPS"
    );

    println!("\n{}", "4. LOSSLESS SCREENSHOT CAPTURE".yellow().bold());
    println!(
        "  {:<48} {}",
        "winvideo screenshot --name chrome.exe".green(),
        "Instant lossless screenshot of Chrome to screenshot.png"
    );
    println!(
        "  {:<48} {}",
        "winvideo screenshot --window \"Code\" --out ide.png".green(),
        "Screenshot of VS Code window"
    );
    println!(
        "  {:<48} {}",
        "winvideo screenshot --monitor 1 --out display.png".green(),
        "Screenshot of full primary display"
    );

    println!("\n{}", "5. CAPTURE CUSTOMIZATION FLAGS".yellow().bold());
    println!(
        "  {:<48} {}",
        "winvideo record --name game.exe --no-cursor".green(),
        "Record game window without mouse cursor"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --name app.exe --fps 30".green(),
        "Record at 30 FPS to reduce video file size"
    );
    println!(
        "  {:<48} {}",
        "winvideo record --name app.exe --border".green(),
        "Show Windows yellow capture border (default: hidden)"
    );

    println!(
        "\n{}",
        "6. AUDIO RECORDING & WINAUDIO COMPANION".yellow().bold()
    );
    println!(
        "  {:<48} {}",
        "winvideo record ...".green(),
        "Records video only (visual stream without audio)"
    );
    println!(
        "  {:<48} {}",
        "winaudio record ...".green(),
        "Records pure digital audio from the same app (WASAPI)"
    );
    println!(
        "  {:<48} {}",
        "ffmpeg -i video.mp4 -i audio.wav -c copy final.mp4".bright_black(),
        "Lossless 1-second merge of video + audio"
    );

    println!("\n{}", "═".repeat(88).bright_black());
}
