//! Digital Product Factory - Pure Rust Native Desktop App
#![allow(dead_code)]
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
mod vector_types;
mod vector_generator;
mod vector_renderer;
mod vector_export;
use eframe::NativeOptions;
use std::io::Write;

/// Where the startup log goes.
///
/// A Windows GUI binary has NO console, so a panic or a renderer failure is otherwise completely
/// silent: the user sees a black window and there is nothing to send anybody. This log is the
/// only record of what actually happened, so it is written next to the executable first (easy to
/// find — it lands beside dpf.exe) and falls back to %APPDATA% and then the temp dir.
fn log_path() -> std::path::PathBuf {
    let name = "dpf-startup.log";
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join(name);
            // Probe writability once; a Program Files install is read-only.
            if std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&p)
                .is_ok()
            {
                return p;
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = std::path::PathBuf::from(appdata).join("DigitalProductFactory");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join(name);
    }
    std::env::temp_dir().join(name)
}

fn log(msg: &str) {
    let line = format!("[{}] {}\n", timestamp(), msg);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = f.write_all(line.as_bytes());
    }
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
    NativeOptions {
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
    }
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
    log(&format!("log file: {}", log_path().display()));
    log(&format!("renderer: {renderer:?} ({why})"));

    let mut options = base_options();
    options.renderer = renderer;

    log("creating window...");
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
            log("Try the other one: dpf-glow.bat  /  dpf-wgpu.bat");
        }
    }
    result
}
