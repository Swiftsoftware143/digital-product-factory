//! Settings dialog

use egui::*;
use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("General").clicked() {
                    // Show general settings
                }
                if ui.button("API Keys").clicked() {
                    // Show API settings
                }
                if ui.button("Safety").clicked() {
                    // Show safety limits
                }
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.heading("API Configuration");

                    ui.horizontal(|ui| {
                        ui.label("OpenAI:");
                        ui.add(egui::TextEdit::singleline(&mut app.config.openai_key).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Anthropic:");
                        ui.add(egui::TextEdit::singleline(&mut app.config.anthropic_key).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Google:");
                        ui.add(egui::TextEdit::singleline(&mut app.config.google_key).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.label("DeepSeek:");
                        ui.add(egui::TextEdit::singleline(&mut app.config.deepseek_key).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Moonshot:");
                        ui.add(egui::TextEdit::singleline(&mut app.config.moonshot_key).password(true));
                    });
                });

                ui.group(|ui| {
                    ui.heading("Safety Limits");

                    ui.horizontal(|ui| {
                        ui.label("Max searches/hour:");
                        ui.add(DragValue::new(&mut app.config.max_searches_per_hour));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max products/day:");
                        ui.add(DragValue::new(&mut app.config.max_products_per_day));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max publish/hour:");
                        ui.add(DragValue::new(&mut app.config.max_publish_per_hour));
                    });
                });

                ui.group(|ui| {
                    ui.heading("Performance");

                    ui.checkbox(&mut app.config.auto_save, "Auto-save");
                    ui.checkbox(&mut app.config.dark_mode, "Dark mode");

                    ui.horizontal(|ui| {
                        ui.label("Max concurrent tasks:");
                        ui.add(DragValue::new(&mut app.config.max_concurrent_tasks));
                    });
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.show_settings = false;
                }
                if ui.button("Save").clicked() {
                    // Save config
                    app.show_settings = false;
                }
            });
        });
}
