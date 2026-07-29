//! Mockup View â€” DropMock-style mockup compositor UI
//!
//! Provides the egui interface for:
//! - Scene template selection
//! - Product image loading
//! - Guide region selection and placement
//! - Scale/offset controls
//! - Export as PNG/JPG
//!
//! Tier: Agency+ (gated)

use image::GenericImageView;
use egui::*;
use crate::app::DpfApp;
use crate::inline_help;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("ðŸŽ¨ Mockup Compositor");
            inline_help::help_button(ui, "mockup_compositor", &mut app.active_help_topic);
        });
        ui.separator();

        if !app.mockup_compositor.can_use_compositor() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new("ðŸ”’ Agency+ Feature").size(24.0).strong().color(Color32::GOLD));
                ui.add_space(8.0);
                ui.label("Upgrade your license to unlock the Mockup Compositor.");
                ui.label("Create professional product mockups with scene templates,");
                ui.label("image overlays, and PNG/JPG export.");
                ui.add_space(8.0);
                if ui.button("View Licenses").clicked() {
                    app.show_license_dialog = true;
                }
            });
            return;
        }

        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_min_width(280.0);

                    ui.group(|ui| {
                        ui.label(RichText::new("ðŸ“¦ Product Image").strong());
                        if ui.button("Load Product Image").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                let path_str = path.to_string_lossy().to_string();
                                match app.mockup_compositor.load_product(&path_str) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("Failed to load product: {}", e);
                                    }
                                }
                            }
                        }
                        if let Some(ref product) = app.mockup_compositor.product {
                            ui.label(format!("Loaded: {}", product.name));
                            let (w, h) = product.image.dimensions();
                            ui.label(format!("{} x {} px", w, h));
                        } else {
                            ui.label(RichText::new("No product loaded").weak());
                        }
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(RichText::new("ðŸ–¼ï¸ Scene Template").strong());
                        let names = app.mockup_compositor.scene_names();
                        if names.len() == 1 && names[0].starts_with("No templates") {
                            ui.label(RichText::new(&names[0]).weak());
                        } else {
                            let current_name = app.mockup_compositor.current_template
                                .and_then(|i| app.mockup_compositor.templates.get(i))
                                .map(|t| t.name.clone())
                                .unwrap_or_default();
                            ComboBox::from_id_source("template_select")
                                .selected_text(&current_name)
                                .width(240.0)
                                .show_ui(ui, |ui| {
                                    for (i, name) in names.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut app.mockup_compositor.current_template,
                                            Some(i),
                                            name,
                                        );
                                    }
                                });
                        }
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(RichText::new("ðŸ“ Guide Region").strong());
                        if let Some(t_idx) = app.mockup_compositor.current_template {
                            if let Some(template) = app.mockup_compositor.templates.get(t_idx) {
                                if template.guides.is_empty() {
                                    ui.label(RichText::new("No guide regions defined").weak());
                                } else {
                                    let current_guide = app.mockup_compositor.selected_guide
                                        .map(|i| template.guides[i].label.clone())
                                        .unwrap_or_default();
                                    ComboBox::from_id_source("guide_select")
                                        .selected_text(&current_guide)
                                        .width(240.0)
                                        .show_ui(ui, |ui| {
                                            for (i, guide) in template.guides.iter().enumerate() {
                                                let label = format!(
                                                    "{} ({}x{} @ {}, {})",
                                                    guide.label, guide.width, guide.height,
                                                    guide.x, guide.y
                                                );
                                                ui.selectable_value(
                                                    &mut app.mockup_compositor.selected_guide,
                                                    Some(i),
                                                    label,
                                                );
                                            }
                                        });
                                }
                            }
                        } else {
                            ui.label(RichText::new("Select a template first").weak());
                        }
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label(RichText::new("ðŸŽ¯ Placement").strong());
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(Slider::new(&mut app.mockup_compositor.scale, 0.1..=3.0)
                                .step_by(0.05)
                                .text("x"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("X Offset:");
                            ui.add(Slider::new(&mut app.mockup_compositor.offset_x, -500..=500)
                                .text("px"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y Offset:");
                            ui.add(Slider::new(&mut app.mockup_compositor.offset_y, -500..=500)
                                .text("px"));
                        });
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let can_compose = app.mockup_compositor.product.is_some()
                            && app.mockup_compositor.current_template.is_some()
                            && app.mockup_compositor.selected_guide.is_some();
                        if ui.add_enabled(can_compose, Button::new("ðŸ’¾ Export PNG")).clicked() {
                            export_composite(app, "png");
                        }
                        if ui.add_enabled(can_compose, Button::new("ðŸ’¾ Export JPG")).clicked() {
                            export_composite(app, "jpg");
                        }
                    });
                });
            });

            columns[1].vertical(|ui| {
                ui.label(RichText::new("ðŸ‘ï¸ Preview").strong());
                ui.separator();
                ui.add_space(4.0);

                if let Some(t_idx) = app.mockup_compositor.current_template {
                    if let Some(template) = app.mockup_compositor.templates.get(t_idx) {
                        if let Some(ref preview) = template.preview {
                            let (pw, ph) = preview.dimensions();
                            Frame::group(ui.style()).show(ui, |ui| {
                                ui.monospace(format!("Template: {} ({}x{})", template.name, pw, ph));
                            });
                            for (i, guide) in template.guides.iter().enumerate() {
                                let highlight = app.mockup_compositor.selected_guide == Some(i);
                                let label = if highlight {
                                    RichText::new(format!(
                                        "  {}: {}x{} @ ({},{}) - SELECTED",
                                        guide.label, guide.width, guide.height, guide.x, guide.y
                                    ))
                                    .color(Color32::YELLOW)
                                } else {
                                    RichText::new(format!(
                                        "  {}: {}x{} @ ({},{})",
                                        guide.label, guide.width, guide.height, guide.x, guide.y
                                    ))
                                    .color(Color32::GREEN)
                                };
                                ui.label(label);
                            }
                            let max_w = ui.available_width().min(600.0);
                            let s = max_w / pw as f32;
                            let display_h = (ph as f32 * s).max(200.0);
                            Frame::dark_canvas(ui.style()).show(ui, |ui| {
                                ui.allocate_space(vec2(max_w, display_h));
                            });
                        } else {
                            ui.label("Template preview not available");
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label("Select a scene template to begin");
                        ui.label("Load a product image to overlay");
                    });
                }

                ui.add_space(12.0);

                if let Some(ref product) = app.mockup_compositor.product {
                    ui.group(|ui| {
                        ui.label(format!("Product: {}", product.name));
                        let (w, h) = product.image.dimensions();
                        let thumb_w = ui.available_width().min(200.0);
                        let s = thumb_w / w as f32;
                        let thumb_h = (h as f32 * s).max(40.0);
                        ui.allocate_space(vec2(thumb_w, thumb_h));
                    });
                }
            });
        });
    });
}

fn export_composite(app: &mut DpfApp, format: &str) {
    if let (Some(t_idx), Some(g_idx)) = (app.mockup_compositor.current_template, app.mockup_compositor.selected_guide) {
        match app.mockup_compositor.compose(t_idx, g_idx) {
            Ok(result) => {
                let ext = if format == "png" { "png" } else { "jpg" };
                let default_name = format!("mockup_output.{}", ext);
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(format!("Image (*.{})", ext), &[ext])
                    .set_file_name(&default_name)
                    .save_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    let _ = app.mockup_compositor.export(&result, &path_str, format);
                }
            }
            Err(e) => {
                tracing::error!("Composition failed: {}", e);
            }
        }
    }
}
