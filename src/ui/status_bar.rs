//! Status bar at bottom of window

use egui::*;
use crate::app::DpfApp;
use crate::inline_help;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left side - help and admin buttons
                if ui.button("❓ Help").clicked() {
                    app.active_help_topic = Some("__index__".to_string());
                }

                // Admin is reached from the sidebar, and only with an OWNER licence. There used to
                // be a 🛡️ toggle here that flipped `admin_mode` with no licence check at all, which
                // meant ANY customer — including the free tier — could open the key-generation
                // screen. Removed: the sidebar entry is owner-gated, and nothing here can bypass it.

                // Theme toggle — one click between the night and daylight themes.
                // The label shows the CURRENT theme; clicking switches to the other one.
                let theme_label = if app.config.dark_mode {
                    "🌙 Night"
                } else {
                    "☀️ Daylight"
                };
                if ui
                    .selectable_label(app.config.dark_mode, theme_label)
                    .on_hover_text("Switch between the daylight and night themes")
                    .clicked()
                {
                    app.config.dark_mode = !app.config.dark_mode;
                }

                ui.separator();

                // Center - status messages
                if app.admin.admin_mode {
                    ui.label(RichText::new("Admin Mode Active").color(Color32::YELLOW).strong());
                } else {
                    ui.label("Ready");
                }

                // Right side
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(format!("{:.0} FPS", app.fps));
                    ui.separator();

                    // License status
                    let license_status = if app.license_manager.is_licensed() {
                        "✓ Licensed"
                    } else {
                        "⚠ Unlicensed"
                    };
                    if ui.selectable_label(false, license_status).clicked() {
                        app.show_license_dialog = true;
                    }
                });
            });
        });

    // The help overlay is rendered by `main_content::show`, which runs earlier in the same frame.
    // It used to be drawn HERE as well, so every help window (and every widget inside it) was
    // built twice per frame - egui then reports "Second use of widget ID" for each one and paints
    // its debug boxes over the panel. One owner, one draw.
}
