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
                if ui.button("â“ Help").clicked() {
                    app.active_help_topic = Some("__index__".to_string());
                }

                // Admin toggle button
                let admin_active = app.admin.admin_mode;
                let admin_label = if admin_active { "ðŸ›¡ï¸ Admin" } else { "ðŸ›¡ï¸" };
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
                        "âœ“ Licensed"
                    } else {
                        "âš  Unlicensed"
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
