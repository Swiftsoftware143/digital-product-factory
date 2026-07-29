//! Admin Control Panel View â€” manage licenses, feature flags, pricing, platform formats

use egui::*;
use crate::app::DpfApp;
use crate::admin::AdminSection;

/// Show the admin control panel
pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("ðŸ›¡ï¸ Admin Control Panel");
            if ui.button("âŽ Exit Admin Mode").clicked() {
                app.admin.admin_mode = false;
                app.current_tab = crate::app::Tab::Dashboard;
            }
        });
        ui.separator();

        // Section tabs
        ui.horizontal(|ui| {
            let sections = [
                (AdminSection::Features, "ðŸ“‹ Feature Tiers"),
                (AdminSection::Pricing, "ðŸ’° Pricing"),
                (AdminSection::Formats, "ðŸ“ Platform Formats"),
                (AdminSection::Keys, "ðŸ”‘ License Keys"),
                (AdminSection::Revocations, "â›” Revocations"),
            ];
            for (section, label) in &sections {
                let selected = app.admin.active_section == *section;
                if ui.selectable_label(selected, *label).clicked() {
                    app.admin.active_section = *section;
                }
            }
        });
        ui.separator();

        match app.admin.active_section {
            AdminSection::Features => show_feature_tiers(app, ui),
            AdminSection::Pricing => show_pricing(app, ui),
            AdminSection::Formats => show_platform_formats(app, ui),
            AdminSection::Keys => show_license_keys(app, ui),
            AdminSection::Revocations => show_revocations(app, ui),
        }

        // Status message
        if !app.admin.status_message.is_empty() {
            ui.separator();
            ui.label(RichText::new(&app.admin.status_message).size(12.0));
        }
    });
}

/// Feature Tiers â€” JSON editor
fn show_feature_tiers(app: &mut DpfApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.strong("Feature Tiers Configuration");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("ðŸ’¾ Save").clicked() {
                app.admin.save_config("features");
            }
        });
    });
    ui.label("Edit the tier definitions as JSON. Each tier defines allowed features and device limits.");
    ui.separator();

    let mut json_str = serde_json::to_string_pretty(&app.admin.feature_tiers).unwrap_or_default();
    let response = egui::TextEdit::multiline(&mut json_str)
        .font(TextStyle::Monospace)
        .desired_rows(20)
        .code_editor()
        .ui(ui);
    if response.lost_focus() {
        if let Ok(v) = serde_json::from_str(&json_str) {
            app.admin.feature_tiers = v;
        }
    }
}

/// Pricing â€” JSON editor
fn show_pricing(app: &mut DpfApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.strong("Pricing Configuration");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("ðŸ’¾ Save").clicked() {
                app.admin.save_config("pricing");
            }
        });
    });
    ui.label("Edit pricing data as JSON. Format: { tier: { price: number, period: string } }");
    ui.separator();

    let mut json_str = serde_json::to_string_pretty(&app.admin.pricing_data).unwrap_or_default();
    let response = egui::TextEdit::multiline(&mut json_str)
        .font(TextStyle::Monospace)
        .desired_rows(16)
        .code_editor()
        .ui(ui);
    if response.lost_focus() {
        if let Ok(v) = serde_json::from_str(&json_str) {
            app.admin.pricing_data = v;
        }
    }
}

/// Platform Formats â€” JSON editor
fn show_platform_formats(app: &mut DpfApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.strong("Platform Formats Configuration");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("ðŸ’¾ Save").clicked() {
                app.admin.save_config("formats");
            }
        });
    });
    ui.label("Edit platform format constraints as JSON. Keyed by platform name.");
    ui.separator();

    let mut json_str = serde_json::to_string_pretty(&app.admin.platform_formats).unwrap_or_default();
    let response = egui::TextEdit::multiline(&mut json_str)
        .font(TextStyle::Monospace)
        .desired_rows(20)
        .code_editor()
        .ui(ui);
    if response.lost_focus() {
        if let Ok(v) = serde_json::from_str(&json_str) {
            app.admin.platform_formats = v;
        }
    }
}

/// License Keys â€” generate new keys
fn show_license_keys(app: &mut DpfApp, ui: &mut Ui) {
    ui.strong("Generate License Keys");
    ui.label("Select a tier, enter device count, and generate a new license key.");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Tier:");
        let tiers = ["personal", "team", "agency", "enterprise"];
        let current_idx = tiers.iter().position(|t| *t == app.admin.generate_key_input.as_str()).unwrap_or(0);
        egui::ComboBox::from_id_source("tier_combo")
            .selected_text(tiers[current_idx])
            .show_ui(ui, |ui| {
                for t in &tiers {
                    if ui.selectable_label(false, *t).clicked() {
                        app.admin.generate_key_input = t.to_string();
                    }
                }
            });
    });

    let mut device_count: u32 = 1;
    ui.horizontal(|ui| {
        ui.label("Devices:");
        ui.add(Slider::new(&mut device_count, 1..=100));
    });

    if ui.button("ðŸ”‘ Generate Key").clicked() {
        let tier_input = app.admin.generate_key_input.clone();
        let tier = if tier_input.is_empty() {
            "personal".to_string()
        } else {
            tier_input
        };
        app.admin.generate_key(&tier, device_count);
    }

    ui.separator();
    if let Some(key) = app.admin.generated_key.clone() {
        ui.horizontal(|ui| {
            ui.strong("Generated Key:");
            ui.label(RichText::new(&key).color(Color32::GREEN).size(16.0));
            if ui.button("ðŸ“‹ Copy").clicked() {
                ui.output_mut(|o| o.copied_text = key.clone());
            }
        });
    }
}

/// Revocations â€” view and manage revoked keys
fn show_revocations(app: &mut DpfApp, ui: &mut Ui) {
    ui.strong("Revoked License Keys");
    ui.label("Enter a license key and click Revoke to invalidate it.");
    ui.separator();

    let mut revoke_input = String::new();
    ui.horizontal(|ui| {
        ui.label("Key:");
        ui.text_edit_singleline(&mut revoke_input);
        if ui.button("â›” Revoke").clicked() {
            app.admin.revoke_key(&revoke_input);
        }
    });

    ui.separator();
    ui.strong(format!("Revoked Keys ({}):", app.admin.revoked_keys.len()));
    if app.admin.revoked_keys.is_empty() {
        ui.label("  No revoked keys.");
    } else {
        let revoked_keys = app.admin.revoked_keys.clone();
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for key in &revoked_keys {
                    ui.horizontal(|ui| {
                        ui.label("  ðŸ”´");
                        ui.label(key);
                    });
                }
            });
    }
}
