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

mod adverts;
mod advert_generator;
mod advert_export;
mod vector_types;
mod vector_generator;
mod vector_renderer;
mod vector_export;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Digital Product Factory"),
        // Intel UHD (and other integrated GPU) drivers on Windows frequently
        // render a black window with egui_glow's default multisampling/vsync
        // settings. Disabling both avoids the bad first-frame/driver path.
        multisampling: 0,
        vsync: false,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        // Switched from Glow (OpenGL) to wgpu, which uses DirectX12 on
        // Windows. Some Intel UHD Graphics driver builds render a black
        // window under OpenGL/glow no matter what GL settings are used;
        // their DirectX drivers are far more reliable, so wgpu avoids the
        // problem entirely instead of working around it.
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Digital Product Factory",
        options,
        Box::new(|cc| Box::new(app::DpfApp::new(cc))),
    )
}