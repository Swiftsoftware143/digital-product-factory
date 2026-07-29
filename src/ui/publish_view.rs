//! Publish View — Marketplace Publishing UI

use egui::*;
use crate::app::DpfApp;
use crate::inline_help;
use crate::publishing::{PublishManager, PublishStatus, store_api_key, delete_api_key};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("📦 Marketplace Publishing");
            inline_help::help_button(ui, "publishing", &mut app.active_help_topic);
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Connected Platforms");
                Frame::group(ui.style()).show(ui, |ui| {
                    for (id, platform) in &[("etsy", "Etsy"), ("gumroad", "Gumroad"),
                                            ("shopify", "Shopify"), ("payhip", "Payhip")]
                    {
                        let has_creds = PublishManager::has_credentials(id);
                        let icon = if has_creds { "✅" } else { "❌" };
                        if ui.selectable_label(
                            app.selected_platform.as_deref() == Some(id),
                            format!("{} {}", icon, platform)
                        ).clicked() {
                            app.selected_platform = Some(id.to_string());
                        }
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                if let Some(plat) = &app.selected_platform.clone() {
                    let name = match plat.as_str() {
                        "etsy" => "Etsy",
                        "gumroad" => "Gumroad",
                        "shopify" => "Shopify",
                        "payhip" => "Payhip",
                        _ => "Platform",
                    };
                    ui.horizontal(|ui| {
                        ui.heading(format!("{} Settings", name));
                        inline_help::help_button(ui, &format!("publishing_{}", plat), &mut app.active_help_topic);
                    });

                    let has_creds = PublishManager::has_credentials(plat);

                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        if has_creds {
                            ui.label("••••••••••••••••");
                            if ui.button("Remove").clicked() {
                                let _ = delete_api_key(plat);
                            }
                        } else {
                            ui.text_edit_singleline(&mut app.new_api_key);
                            if ui.button("Save Key").clicked() && !app.new_api_key.is_empty() {
                                let _ = store_api_key(plat, &app.new_api_key);
                                app.new_api_key.clear();
                            }
                        }
                    });

                    if let Some(fmt) = app.publish_manager.platform_formats.get(plat.as_str()) {
                        ui.label(format!("Thumbnail: {}×{}", fmt.thumbnail_width, fmt.thumbnail_height));
                        ui.label(format!("Max file size: {} MB", fmt.max_file_size_mb));
                        ui.label(format!("Max title length: {} chars", fmt.max_title_length));
                        if fmt.max_tags > 0 {
                            ui.label(format!("Max tags: {}", fmt.max_tags));
                        }
                    }

                    ui.separator();

                    ui.heading("Publish Product");
                    let product_names: Vec<String> = app.pipeline.ideas
                        .iter()
                        .filter(|i| i.stage == crate::pipeline::PipelineStage::Listed
                                 || i.stage == crate::pipeline::PipelineStage::Review)
                        .map(|i| i.title.clone())
                        .collect();

                    if product_names.is_empty() {
                        ui.label("Move a product to 'Listed' stage to publish it.");
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Product:");
                            egui::ComboBox::from_id_source("publish_product")
                                .selected_text(&app.publish_target)
                                .show_ui(ui, |ui| {
                                    for name in &product_names {
                                        ui.selectable_value(&mut app.publish_target, name.clone(), name.as_str());
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Price ($):");
                            ui.add(DragValue::new(&mut app.publish_price).speed(0.5));
                        });

                        if ui.button("Publish").clicked() && !app.publish_target.is_empty() {
                            app.pending_publish = Some((
                                app.publish_target.clone(),
                                plat.clone(),
                                app.publish_price,
                            ));
                        }
                    }
                } else {
                    ui.label("Select a platform on the left to configure.");
                }
            });
        });

        ui.separator();
        ui.heading("Publish Log");

        ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            for log in &app.publish_manager.publish_logs {
                Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", log.published_at.format("%Y-%m-%d")));
                        ui.label(&log.product_name);
                        ui.label(format!("on {}", &log.platform));

                        let (status_text, color) = match log.status {
                            PublishStatus::Published => ("✓ Published", Color32::GREEN),
                            PublishStatus::Pending => ("⏳ Pending", Color32::YELLOW),
                            PublishStatus::Failed => ("✗ Failed", Color32::RED),
                            PublishStatus::Removed => ("Removed", Color32::GRAY),
                        };
                        ui.colored_label(color, status_text);

                        if let Some(url) = &log.listing_url {
                            ui.hyperlink_to("View", url.clone());
                        }
                    });
                });
            }
            if app.publish_manager.publish_logs.is_empty() {
                ui.label("  No publish logs yet.");
            }
        });
    });
}