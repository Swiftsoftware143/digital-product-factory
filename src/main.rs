//! Digital Product Factory - Pure Rust Native Desktop App

// A GUI app must declare the Windows GUI subsystem, or Windows treats the binary as a CONSOLE
// program: double-clicking it opens a black console window, and the customer sees that instead of
// (or behind) the app. It was missing, so every Windows build shipped a console window.
//
// Kept ON in debug builds deliberately: a console is what makes a panic readable during
// development. Release is silent, which is why dpf-startup.log exists.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Dead code is WARNED, not silenced. A crate-level `allow(dead_code)` means the compiler never
// reports anything — which is how 39 unused items and 15 unused imports accumulated unnoticed, and
// why the cleanup mandate ("nothing left behind") was unenforceable. Warning keeps the build green
// while making every dead item visible in `cargo check`, so nothing can hide again.
#![warn(dead_code)]
#![allow(unused_variables, unused_imports)]

#![allow(clippy::too_many_arguments)]
mod app;
mod pipeline;
mod product_generator;
mod license_manager;
mod templates;
mod llm_router;
mod research;
mod scheduler;
mod bundler;
mod exporter;
mod contract_generator;
mod database;
mod config;
mod presets;
mod ui;
pub mod mockup_compositor;
pub mod analytics;
pub mod publishing;
pub mod db_ext;
pub mod inline_help;
pub mod product_variants;
pub mod admin;
pub mod qc;
pub mod webhook;
pub mod asset_library;
pub mod compliance;
pub mod client_manager;

mod adverts;
mod advert_generator;
mod advert_export;
mod strategy;
mod upgrade;
mod vector_types;
mod vector_generator;
mod vector_renderer;
mod vector_export;
mod theme;
use eframe::NativeOptions;
use std::io::Write;

/// Every place we will try to write the log.
///
/// A missing log is a disaster for diagnosis: it looks identical to "the app never started", and
/// the first report of this build was exactly that — no file anywhere. So the log is written to
/// ALL of these simultaneously rather than falling back down a list, and any one of them
/// surviving is enough. DPF_LOG forces a single explicit path.
///
/// A Windows GUI binary has NO console, so a panic or a renderer failure is otherwise completely
/// silent: the user sees a black window and there is nothing to send anybody.
fn log_paths() -> Vec<std::path::PathBuf> {
    let name = "dpf-startup.log";
    if let Ok(explicit) = std::env::var("DPF_LOG") {
        if !explicit.trim().is_empty() {
            return vec![std::path::PathBuf::from(explicit)];
        }
    }
    let mut out = Vec::new();
    // 1. beside the executable - the obvious one, and the one to send to support
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(name));
        }
    }
    // 2. %APPDATA%\DigitalProductFactory - always writable for a normal user, survives the
    //    executable being run from a temp dir (which is what happens when a zip is opened and the
    //    exe launched from inside it without extracting)
    if let Ok(appdata) = std::env::var("APPDATA") {
        out.push(std::path::PathBuf::from(appdata).join("DigitalProductFactory").join(name));
    }
    // 3. %TEMP% - last resort, and the one place guaranteed to exist
    out.push(std::env::temp_dir().join(name));
    // 4. the process working directory, which for a double-clicked exe is usually its own folder
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(name);
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

fn log(msg: &str) {
    let line = format!("[{}] {}\n", timestamp(), msg);
    for path in log_paths() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = f.write_all(line.as_bytes());
        }
    }
}

/// Public so modules that run BEFORE the first frame (notably `app::DpfApp::new`) can record how
/// far they got. A window that is already on screen and never paints has two completely
/// different causes — it blocked/panicked during setup, or it painted and the GPU never
/// presented — and where the log stops is what tells them apart.
pub fn log_line(msg: &str) {
    log(msg);
}

/// Minimal timestamp without pulling in a date crate for this one call.
fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("epoch+{}s", d.as_secs()),
        Err(_) => "epoch+?".to_string(),
    }
}

/// Turn any panic into a log line. Without this, a panic inside the UI thread kills the window
/// and leaves NOTHING behind — indistinguishable from a renderer problem.
fn install_panic_logger() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        log(&format!("PANIC at {loc}: {msg}"));
        log(&format!("{}", std::backtrace::Backtrace::force_capture()));
    }));
}

fn base_options() -> NativeOptions {
    let mut options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Digital Product Factory"),
        // Integrated-GPU drivers on Windows frequently render a black first frame with the
        // default multisampling/vsync settings, so both are off.
        multisampling: 0,
        vsync: false,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    // Brand icon in the title bar and the taskbar. Embedded in the binary, so the executable is
    // still a single self-contained file — there is no icon file for the user to keep next to it.
    if let Some(icon) = app_icon() {
        options.viewport = options.viewport.with_icon(std::sync::Arc::new(icon));
    }

    options
}

/// Decode the embedded brand mark into the pixel form egui wants for a window icon.
///
/// Failure is not fatal: a missing or unreadable icon should cost the icon, never the application.
fn app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/icon.png");
    let decoded = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = decoded.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    Some(egui::IconData {
        rgba: decoded.into_raw(),
        width,
        height,
    })
}

/// Which back end to use, and why there is no automatic retry.
///
/// Two things are true at once and they pull in opposite directions:
///   * some integrated GPUs render a BLACK window under Glow/OpenGL, so wgpu (DirectX 12) was
///     made the default;
///   * the window was then reported black under wgpu as well, on another machine.
///
/// The obvious fix — try one, fall back to the other — CANNOT BE DONE IN-PROCESS: winit allows
/// exactly one EventLoop per process, so the second `run_native` dies with "EventLoop can't be
/// recreated" (measured, and it is why an earlier version of this file's fallback never worked).
/// And it cannot be done automatically from the outside either, because a black window is not an
/// exit: the app is alive and simply never presenting pixels.
///
/// So the back end is SELECTABLE without a rebuild, and the choice is recorded in the log:
///     dpf.exe --renderer glow          (or  set DPF_RENDERER=glow)
///     dpf.exe --renderer wgpu
/// The release ships a dpf-glow.bat and dpf-wgpu.bat beside the executable so switching is a
/// double-click rather than a command line.
fn chosen_renderer() -> (eframe::Renderer, &'static str) {
    let arg = std::env::args().position(|a| a == "--renderer");
    let from_arg = arg.and_then(|i| std::env::args().nth(i + 1));
    let want = std::env::var("DPF_RENDERER").ok().or(from_arg);
    match want.as_deref().map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("glow") => (eframe::Renderer::Glow, "explicit: --renderer glow / DPF_RENDERER"),
        Some("wgpu") => (eframe::Renderer::Wgpu, "explicit: --renderer wgpu / DPF_RENDERER"),
        _ => (
            eframe::Renderer::Wgpu,
            "default (no --renderer given; use --renderer glow if the window is black)",
        ),
    }
}

/// The other back end. Kept in one place so the two call sites cannot disagree.
fn other_renderer(r: eframe::Renderer) -> eframe::Renderer {
    match r {
        eframe::Renderer::Glow => eframe::Renderer::Wgpu,
        eframe::Renderer::Wgpu => eframe::Renderer::Glow,
    }
}

fn renderer_name(r: eframe::Renderer) -> &'static str {
    match r {
        eframe::Renderer::Glow => "glow",
        eframe::Renderer::Wgpu => "wgpu",
    }
}

/// Relaunch this executable with the other back end. Returns true if a new process was started.
///
/// Sets DPF_RENDERER_FALLBACK=0 in the child so a machine where NEITHER back end works cannot
/// bounce between two processes forever.
fn relaunch_with_other_renderer(current: eframe::Renderer) -> bool {
    if std::env::var("DPF_RENDERER_FALLBACK").as_deref() == Ok("0") {
        log("fallback already attempted once — not relaunching again (set DPF_RENDERER to choose)");
        return false;
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { log(&format!("cannot locate own executable to relaunch: {e}")); return false; }
    };
    let next = renderer_name(other_renderer(current));
    log(&format!("relaunching with --renderer {next} (fresh process = fresh event loop)"));

    let mut args: Vec<String> = std::env::args().skip(1)
        .filter(|a| a != "--renderer")
        .filter(|a| a != renderer_name(current))
        .filter(|a| a != "glow" && a != "wgpu")
        .collect();
    args.push("--renderer".into());
    args.push(next.into());

    match std::process::Command::new(exe).args(&args)
        .env("DPF_RENDERER_FALLBACK", "0").spawn()
    {
        Ok(_) => true,
        Err(e) => { log(&format!("relaunch failed: {e}")); false }
    }
}

/// Mode 2 watchdog: a back end can initialise and then never present a single pixel, leaving a
/// black window and a perfectly healthy-looking process. Nothing inside the event loop can decide
/// to fall back, so this watches the app's own "frame N rendered" marker and relaunches if the
/// first frame never lands.
fn spawn_black_window_watchdog(current: eframe::Renderer) {
    if std::env::var("DPF_RENDERER_FALLBACK").as_deref() == Ok("0") {
        return;
    }
    // Only the DEFAULT back end auto-switches: if the user named one, respect it.
    if std::env::var("DPF_RENDERER").is_ok() || std::env::args().any(|a| a == "--renderer") {
        return;
    }
    std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            std::thread::sleep(std::time::Duration::from_millis(750));
            if first_frame_rendered() {
                return; // healthy — the app is drawing
            }
            if std::time::Instant::now() >= deadline {
                log("NO FRAME RENDERED within 20s — the back end is not presenting (black window).");
                if relaunch_with_other_renderer(current) {
                    std::process::exit(0); // the relaunched process takes over
                }
                return;
            }
        }
    });
}

/// True once the startup log shows the app reached its first frame.
fn first_frame_rendered() -> bool {
    log_paths().iter().any(|p| {
        std::fs::read_to_string(p)
            .map(|t| t.contains("frame 1 rendered"))
            .unwrap_or(false)
    })
}

fn main() -> eframe::Result<()> {
    install_panic_logger();
    let (renderer, why) = chosen_renderer();

    log("================================================================");
    log(&format!(
        "DPF {} starting on {}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    log("log file (written to all of these):");
    for p in log_paths() {
        log(&format!("    {}", p.display()));
    }
    log(&format!("exe: {:?}", std::env::current_exe()));
    log(&format!("cwd: {:?}", std::env::current_dir()));
    log(&format!("renderer: {renderer:?} ({why})"));

    let mut options = base_options();
    options.renderer = renderer;

    log("creating window...");
    spawn_black_window_watchdog(renderer);
    let result = eframe::run_native(
        "Digital Product Factory",
        options,
        Box::new(|cc| {
            log("window + egui context created OK — if the window is black from here, the GPU is not presenting");
            Box::new(app::DpfApp::new(cc))
        }),
    );

    match &result {
        Ok(()) => log(&format!("{renderer:?}: exited normally")),
        Err(e) => {
            log(&format!("{renderer:?} FAILED to start: {e}"));
            log("If this mentions the event loop or a display, the back end could not initialise.");
            // Mode 1: the back end could not initialise. A fresh process gets a fresh EventLoop,
            // so relaunching with the other back end is safe here (in-process retry is not).
            if relaunch_with_other_renderer(renderer) {
                return Ok(());
            }
            log("Try the other one: dpf-glow.bat  /  dpf-wgpu.bat");
        }
    }
    result
}
