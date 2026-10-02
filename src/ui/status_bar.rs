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

                // Admin toggle button
                let admin_active = app.admin.admin_mode;
                let admin_label = if admin_active { "🛡️ Admin" } else { "🛡️" };
                let admin_btn = if admin_active {
                    ui.selectable_label(true, admin_label)
                } else {
                    ui.selectable_label(false, admin_label)
                };
                if admin_btn.clicked() {
                    app.admin.admin_mode = !app.admin.admin_mode;
                    if app.admin.admin_mode {
                        app.current_tab = crate::app::Tab::Admin;
                    } else if app.current_tab == crate::app::Tab::Admin {
                        app.current_tab = crate::app::Tab::Dashboard;
                    }
                }

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

    // Render help overlay
    if let Some(topic_id) = &app.active_help_topic.clone() {
        if topic_id == "__index__" {
            inline_help::show_help_index(ctx, &mut app.active_help_topic);
        } else {
            inline_help::show_help_popup(ctx, topic_id, &mut app.active_help_topic);
        }
    }
}
