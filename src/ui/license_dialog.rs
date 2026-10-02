//! License activation dialog
//!
//! NOTE: the previous version created `let mut key_input = String::new();` **inside** the frame
//! closure, so anything typed was wiped on the next repaint and "Activate" was a `TODO`. The input
//! buffer and the result message now live on `DpfApp`.

use egui::*;
use crate::app::DpfApp;
use crate::license_manager::{features_for_tier, LicenseTier};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    Window::new("Licence")
        .collapsible(false)
        .resizable(false)
        .default_size([480.0, 400.0])
        .show(ctx, |ui| {
            ui.heading("Licence Activation");
            ui.separator();

            if app.license_manager.is_licensed() {
                let tier = app.license_manager.tier();
                ui.label(
                    RichText::new(format!("Active: {}", app.license_manager.tier_name()))
                        .strong()
                        .color(Color32::from_rgb(120, 220, 140)),
                );
                if let Some(license) = app.license_manager.current_license() {
                    ui.label(format!("Key: {}", license.key));
                    ui.label(
                        if license.max_devices < 0 {
                            "Seats: unlimited".to_string()
                        } else {
                            format!("Seats: up to {}", license.max_devices)
                        },
                    );
                }
                ui.add_space(4.0);
                ui.label(format!(
                    "{} modules unlocked on this tier.",
                    features_for_tier(&tier).len()
                ));

                ui.add_space(8.0);
                if ui.button("Deactivate this licence").clicked() {
                    app.license_manager.deactivate();
                    app.license_message = Some((
                        false,
                        "Licence removed — back on the free Personal tier.".to_string(),
                    ));
                }
            } else {
                ui.label(format!("Your tier: {}", app.license_manager.tier_name()));
                ui.add_space(4.0);
                ui.label("Enter your licence key to unlock the modules for your tier:");
                ui.add_space(6.0);

                ui.add(
                    egui::TextEdit::singleline(&mut app.license_key_input)
                        .hint_text("DPF-TEAM-XXXXXXXX-CCCC")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(6.0);

                if ui.button("Activate").clicked() {
                    let key = app.license_key_input.clone();
                    match app.license_manager.activate(&key) {
                        Ok(()) => {
                            app.license_message = Some((
                                false,
                                format!("Activated — {}", app.license_manager.tier_name()),
                            ));
                            app.license_key_input.clear();
                        }
                        Err(e) => app.license_message = Some((true, e)),
                    }
                }
            }

            if let Some((is_err, msg)) = app.license_message.clone() {
                ui.add_space(8.0);
                let color = if is_err {
                    Color32::from_rgb(240, 120, 120)
                } else {
                    Color32::from_rgb(120, 220, 140)
                };
                ui.colored_label(color, msg);
            }

            ui.separator();
            ui.collapsing("Plans", |ui| {
                for t in LicenseTier::all() {
                    let seats = if t.max_devices() < 0 {
                        "unlimited seats".to_string()
                    } else {
                        format!("{} seat{}", t.max_devices(), if t.max_devices() == 1 { "" } else { "s" })
                    };
                    ui.label(format!(
                        "• {} — {}  ({}, {} modules)",
                        t.display_name(),
                        t.price_label(),
                        seats,
                        features_for_tier(&t).len()
                    ));
                }
            });

            ui.separator();
            if ui.button("Close").clicked() {
                app.show_license_dialog = false;
            }
        });
}
