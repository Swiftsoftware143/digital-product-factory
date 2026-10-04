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

            // ── The upgrade offer ────────────────────────────────────────────────────────────
            // Gating tells the user a feature is locked; this is the part that OFFERS the way
            // out. It only appears when they are actually missing something, so it never nags a
            // paying customer who already holds everything.
            let missing = app.license_manager.required_tier_for_missing();
            if let Some(target) = missing {
                ui.add_space(10.0);
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{} of the app is locked on your current plan.",
                        target.locked_summary()
                    ))
                    .strong(),
                );

                let url = crate::upgrade::upgrade_url();
                let placeholder = crate::upgrade::is_placeholder(url);

                ui.add_space(4.0);
                if ui.button(crate::upgrade::cta_label(Some(target.display_name()))).clicked() {
                    if placeholder {
                        // Do not pretend we opened a store that does not exist yet.
                        app.license_message = Some((
                            false,
                            "The upgrade page is not connected yet. It will open here as soon as \
                             the store is live."
                                .to_string(),
                        ));
                    } else {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                }
                ui.small(crate::upgrade::cta_note());
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
                        "• {} — {}  ({} modules)",
                        t.display_name(),
                        seats,
                        features_for_tier(&t).len()
                    ));
                }
                ui.add_space(4.0);
                // Says where the money question is answered, without answering it here.
                ui.small("What each plan costs is on the sales page — a licence can be a one-time payment or a subscription.");
            });

            ui.separator();
            if ui.button("Close").clicked() {
                app.show_license_dialog = false;
            }
        });
}
