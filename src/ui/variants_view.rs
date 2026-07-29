//! Variants View — Manage product variants and version history
//!
//! Accessible from sidebar (Variants tab) and linked from Pipeline product cards.
//! Shows all variants grouped by product, with version history per variant.

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::product_variants::{Variant, VariantStatus, VariantVersion};
use crate::inline_help;

/// Format options for variant creation
const FORMAT_OPTIONS: &[&str] = &[
    "pdf", "docx", "xlsx", "zip", "png", "jpg",
    "txt", "html", "markdown", "json", "csv",
];


pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        // Toolbar
        ui.horizontal(|ui| {
            ui.heading("🏷️ Product Variants");
            inline_help::help_button(ui, "variants", &mut app.active_help_topic);

            ui.separator();

            // Product selector dropdown
            ui.label("Product:");
            let product_names: Vec<String> = app.pipeline.ideas
                .iter()
                .map(|i| format!("{} (ID:{})", i.title, i.id))
                .collect();
            let product_ids: Vec<usize> = app.pipeline.ideas.iter().map(|i| i.id).collect();
            let selected_idx = app.variant_manager.selected_variant
                .and_then(|vid| {
                    app.variant_manager.variants.iter()
                        .find(|v| v.id == vid)
                        .map(|v| v.product_id)
                })
                .and_then(|pid| product_ids.iter().position(|&id| id == pid));

            let dropdown_idx = selected_idx.map(|i| format!("{} (ID:{})", app.pipeline.ideas[i].title, app.pipeline.ideas[i].id))
                .unwrap_or_default();

            ComboBox::new("product_selector", "")
                .selected_text(if dropdown_idx.is_empty() { "Select product..." } else { &dropdown_idx })
                .width(200.0)
                .show_ui(ui, |ui| {
                    for (i, name) in product_names.iter().enumerate() {
                        let selected = selected_idx == Some(i);
                        if ui.selectable_label(selected, name).clicked() {
                            // Select first variant of this product, or none
                            let pid = product_ids[i];
                            let variant = app.variant_manager.get_variants_for_product(pid)
                                .first()
                                .map(|v| v.id);
                            app.variant_manager.selected_variant = variant;
                        }
                    }
                });

            ui.separator();

            if ui.button("➕ Add Variant").clicked() {
                app.variant_manager.new_variant_name.clear();
                app.variant_manager.new_variant_format = "pdf".to_string();
                app.variant_manager.new_variant_price = "9.99".to_string();
                app.variant_manager.show_add_variant_dialog = true;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("{} variants · {} versions",
                    app.variant_manager.variants.len(),
                    app.variant_manager.versions.len()));
            });
        });

        ui.separator();

        // Main content: split into product list and variant detail
        let product_ids_with_variants: Vec<usize> = {
            let mut ids: Vec<usize> = app.variant_manager.variants.iter()
                .map(|v| v.product_id)
                .collect();
            ids.sort();
            ids.dedup();
            ids
        };

        if product_ids_with_variants.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label("No variants yet. Select a product in the Pipeline, then add variants here.");
                ui.add_space(10.0);
                if ui.button("📋 Go to Pipeline").clicked() {
                    app.current_tab = Tab::Pipeline;
                }
            });
            return;
        }

        // ── Product-Variant Browser ────────────────────────────────
        ScrollArea::vertical().show(ui, |ui| {
            for pid in &product_ids_with_variants {
                // Find product name
                let product_name = app.pipeline.ideas.iter()
                    .find(|i| i.id == *pid)
                    .map(|i| i.title.as_str())
                    .unwrap_or("(deleted product)");

        let variants: Vec<_> = app.variant_manager.get_variants_for_product(*pid).into_iter().cloned().collect();
                let var_count = variants.len();

                // Product header with collapsible section
                let header_text = format!("📦 {} ({} variant{})", product_name, var_count,
                    if var_count == 1 { "" } else { "s" });

                CollapsingHeader::new(header_text)
                    .default_open(true)
                    .id_source(("prod_variants", *pid))
                    .show(ui, |ui| {
                        ui.indent("variant_list", |ui| {
                            if variants.is_empty() {
                                ui.label("  No variants. Add one using the button above.");
                            } else {
                                for variant in variants {
                                    show_variant_card(app, ui, &variant, *pid);
                                }
                            }
                        });
                    });

                ui.separator();
            }
        });
    });

    // ── Dialogs ───────────────────────────────────────────────────

    if app.variant_manager.show_add_variant_dialog {
        show_add_variant_dialog(app, ctx);
    }

    // Version history dialog
    if let Some(variant_id) = app.variant_manager.show_version_history {
        show_version_history_dialog(app, ctx, variant_id);
    }

    // View version content dialog
    if let Some((variant_id, version_number)) = app.variant_manager.show_view_version {
        show_view_version_dialog(app, ctx, variant_id, version_number);
    }
}

/// Card for a single variant within a product group
fn show_variant_card(app: &mut DpfApp, ui: &mut Ui, variant: &Variant, product_id: usize) {
    Frame::group(ui.style())
        .fill(Color32::from_gray(35))
        .stroke(Stroke::new(1.0, Color32::from_gray(60)))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                // Status badge
                let status_color = match variant.status {
                    VariantStatus::Active => Color32::GREEN,
                    VariantStatus::Draft => Color32::YELLOW,
                    VariantStatus::Deprecated => Color32::GRAY,
                    VariantStatus::Archived => Color32::DARK_GRAY,
                };
                ui.colored_label(status_color, variant.status.name());
                ui.separator();

                // Variant name + format
                ui.strong(&variant.name);
                ui.label(format!("[{}]", variant.format.to_uppercase()));

                // Price
                if variant.price > 0.0 {
                    ui.colored_label(Color32::from_rgb(100, 200, 100), format!("${:.2}", variant.price));
                } else {
                    ui.colored_label(Color32::GRAY, "Free");
                }

                // Version count badge
                let versions = app.variant_manager.get_versions(variant.id);
                if let Some(latest) = versions.first() {
                    ui.colored_label(Color32::LIGHT_BLUE,
                        format!("v{}", latest.version_number));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Delete button
                    if ui.button("🗑").on_hover_text("Delete variant") .clicked() {
                        app.variant_manager.delete_variant(&app.db, variant.id);
                    }

                    // Add version button
                    if ui.button("📸").on_hover_text("Snapshot current content") .clicked() {
                        app.variant_manager.add_version(
                            &app.db,
                            variant.id,
                            format!("Version {} of {}", variant.current_version + 1, variant.name),
                            "text".to_string(),
                            "{}".to_string(),
                            0,
                        );
                    }

                    // View version history
                    if ui.button("📋").on_hover_text("Version history") .clicked() {
                        app.variant_manager.show_version_history = Some(variant.id);
                    }

                    // Status quick-switch dropdown
                    if ui.button("⚙").on_hover_text("Change status") .clicked() {
                        // Cycle through statuses
                        let next = match variant.status {
                            VariantStatus::Draft => VariantStatus::Active,
                            VariantStatus::Active => VariantStatus::Deprecated,
                            VariantStatus::Deprecated => VariantStatus::Archived,
                            VariantStatus::Archived => VariantStatus::Draft,
                        };
                        app.variant_manager.update_variant(
                            &app.db, variant.id, None, None, None, Some(next), None
                        );
                    }
                });
            });

            // Quick edit fields on hover
            ui.horizontal(|ui| {
                ui.label("Name:");
                let mut name = variant.name.clone();
                if ui.text_edit_singleline(&mut name).changed() {
                    app.variant_manager.update_variant(
                        &app.db, variant.id, Some(name), None, None, None, None
                    );
                }

                ui.label("Price:");
                let mut price_str = format!("{:.2}", variant.price);
                if ui.text_edit_singleline(&mut price_str).lost_focus() {
                    if let Ok(p) = price_str.parse::<f64>() {
                        app.variant_manager.update_variant(
                            &app.db, variant.id, None, None, Some(p), None, None
                        );
                    }
                }
            });
        });
}

/// Dialog to add a new variant
fn show_add_variant_dialog(app: &mut DpfApp, ctx: &Context) {
    Window::new("Add Variant")
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.label("Create a new variant for the selected product.");

            ui.separator();

            // Product selector
            ui.horizontal(|ui| {
                ui.label("Product:");
                let product_names: Vec<String> = app.pipeline.ideas
                    .iter()
                    .map(|i| format!("{} (ID:{})", i.title, i.id))
                    .collect();
                let product_ids: Vec<usize> = app.pipeline.ideas.iter().map(|i| i.id).collect();

                let selected_idx = app.variant_manager.selected_variant
                    .and_then(|vid| {
                        app.variant_manager.variants.iter()
                            .find(|v| v.id == vid)
                            .map(|v| v.product_id)
                    })
                    .and_then(|pid| product_ids.iter().position(|&id| id == pid));

                let dropdown_text = selected_idx
                    .map(|i| format!("{} (ID:{})", app.pipeline.ideas[i].title, app.pipeline.ideas[i].id))
                    .unwrap_or_default();

                ComboBox::new("add_var_product", "")
                    .selected_text(if dropdown_text.is_empty() { "Select..." } else { &dropdown_text })
                    .width(250.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in product_names.iter().enumerate() {
                            if ui.selectable_label(selected_idx == Some(i), name).clicked() {
                                let pid = product_ids[i];
                                let existing = app.variant_manager.get_variants_for_product(pid);
                                let vid = existing.first().map(|v| v.id);
                                app.variant_manager.selected_variant = vid;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(egui::TextEdit::singleline(&mut app.variant_manager.new_variant_name).hint_text("e.g. Daily Planner - PDF"));
            });

            ui.horizontal(|ui| {
                ui.label("Format:");
                ComboBox::new("format_selector", "")
                    .selected_text(&app.variant_manager.new_variant_format)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for fmt in FORMAT_OPTIONS {
                            if ui.selectable_label(
                                app.variant_manager.new_variant_format == *fmt, *fmt
                            ).clicked() {
                                app.variant_manager.new_variant_format = fmt.to_string();
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Price:");
                ui.add(egui::TextEdit::singleline(&mut app.variant_manager.new_variant_price).hint_text("9.99"));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.variant_manager.show_add_variant_dialog = false;
                }

                let product_id_for_create = app.variant_manager.selected_variant
                    .and_then(|vid| {
                        app.variant_manager.variants.iter()
                            .find(|v| v.id == vid)
                            .map(|v| v.product_id)
                    })
                    .or_else(|| app.pipeline.ideas.first().map(|i| i.id))
                    .unwrap_or(0);

                if ui.button("Create Variant").clicked() {
                    let price = app.variant_manager.new_variant_price
                        .parse::<f64>().unwrap_or(0.0);
                    let name = if app.variant_manager.new_variant_name.is_empty() {
                        format!("Variant {}", app.variant_manager.variants.len() + 1)
                    } else {
                        app.variant_manager.new_variant_name.clone()
                    };

                    let vid = app.variant_manager.create_variant(
                        &app.db,
                        product_id_for_create,
                        name,
                        app.variant_manager.new_variant_format.clone(),
                        price,
                    );

                    // Create version 1 automatically
                    app.variant_manager.add_version(
                        &app.db,
                        vid,
                        format!("Initial version of {}", app.variant_manager.new_variant_name),
                        "text".to_string(),
                        "{\"initial\": true}".to_string(),
                        0,
                    );

                    app.variant_manager.selected_variant = Some(vid);
                    app.variant_manager.show_add_variant_dialog = false;
                }
            });
        });
}

/// Dialog showing version history for a variant
fn show_version_history_dialog(app: &mut DpfApp, ctx: &Context, variant_id: usize) {
    let variant = match app.variant_manager.variants.iter().find(|v| v.id == variant_id) {
        Some(v) => v.clone(),
        None => {
            app.variant_manager.show_version_history = None;
            return;
        }
    };

    let version_refs = app.variant_manager.get_versions(variant_id);
    let versions: Vec<_> = version_refs.into_iter().cloned().collect();

    let current_v = variant.current_version;

    Window::new(format!("Version History: {}", variant.name))
        .collapsible(false)
        .resizable(true)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.label(format!("Variant: {} [{}] — Current: v{}",
                variant.name, variant.format.to_uppercase(), current_v));
            ui.separator();

            if versions.is_empty() {
                ui.label("No versions yet.");
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for version in &versions {
                        let is_current = version.version_number == current_v;

                        Frame::group(ui.style())
                            .fill(if is_current {
                                Color32::from_rgb(20, 60, 20)
                            } else {
                                Color32::from_gray(30)
                            })
                            .show(ui, |ui| {
                                ui.set_min_width(450.0);
                                ui.horizontal(|ui| {
                                    // Version badge
                                    if is_current {
                                        ui.colored_label(Color32::GREEN, "▶");
                                    }
                                    ui.strong(format!("v{}", version.version_number));
                                    ui.separator();

                                    // Timestamp
                                    ui.label(version.created_at.format("%Y-%m-%d %H:%M").to_string());
                                    ui.separator();

                                    // Size
                                    if version.file_size_bytes > 0 {
                                        let size = if version.file_size_bytes > 1024 {
                                            format!("{:.1} KB", version.file_size_bytes as f64 / 1024.0)
                                        } else {
                                            format!("{} B", version.file_size_bytes)
                                        };
                                        ui.label(size);
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        // View content
                                        if ui.button("👁 View").on_hover_text("View this version's content").clicked() {
                                            app.variant_manager.show_view_version = Some((variant_id, version.version_number));
                                        }

                                        // Rollback (only if not current)
                                        if !is_current
                                            && ui.button("↩ Restore").on_hover_text("Rollback to this version").clicked() {
                                                app.variant_manager.rollback_to_version(
                                                    &app.db, variant_id, version.version_number
                                                );
                                            }
                                    });
                                });

                                if !version.metadata.is_empty() && version.metadata != "{}" {
                                    ui.label(RichText::new(format!("  Metadata: {}", version.metadata))
                                        .size(11.0).color(Color32::GRAY));
                                }
                            });
                        ui.add_space(4.0);
                    }
                });
            }

            ui.separator();
            if ui.button("Close").clicked() {
                app.variant_manager.show_version_history = None;
            }
        });
}

/// Dialog to view an old version's content in read-only mode
fn show_view_version_dialog(app: &mut DpfApp, ctx: &Context, variant_id: usize, version_number: u32) {
    let version: Option<VariantVersion> = app.variant_manager.versions.iter()
        .find(|v| v.variant_id == variant_id && v.version_number == version_number)
        .cloned();

    let version = match version {
        Some(v) => v,
        None => {
            app.variant_manager.show_view_version = None;
            return;
        }
    };

    let variant_name = app.variant_manager.variants.iter()
        .find(|v| v.id == variant_id)
        .map(|v| v.name.as_str())
        .unwrap_or("unknown");

    Window::new(format!("View v{} — {}", version_number, variant_name))
        .collapsible(false)
        .resizable(true)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Version: v{}", version_number));
                ui.separator();
                ui.label(format!("Type: {}", version.content_type));
                ui.separator();
                ui.label(format!("Created: {}", version.created_at.format("%Y-%m-%d %H:%M")));
            });

            ui.separator();

            // Content display (read-only)
            ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    Frame::group(ui.style())
                        .fill(Color32::from_gray(25))
                        .show(ui, |ui| {
                            ui.set_min_height(100.0);
                            if version.content.is_empty() {
                                ui.colored_label(Color32::GRAY, "(no content)");
                            } else if version.content.len() > 5000 {
                                // Truncate large content for display
                                let truncated = &version.content[..5000];
                                ui.label(RichText::new(truncated).monospace().size(11.0));
                                ui.colored_label(Color32::GRAY,
                                    format!("... ({} more bytes)", version.content.len() - 5000));
                            } else {
                                ui.label(RichText::new(&version.content).monospace().size(11.0));
                            }
                        });
                });

            if !version.metadata.is_empty() && version.metadata != "{}" {
                ui.separator();
                ui.label(RichText::new(format!("Metadata: {}", version.metadata))
                    .size(11.0).color(Color32::GRAY));
            }

            ui.separator();

            ui.horizontal(|ui| {
                // Can rollback from here too
                let current_v = app.variant_manager.variants.iter()
                    .find(|v| v.id == variant_id)
                    .map(|v| v.current_version)
                    .unwrap_or(0);

                if version_number != current_v
                    && ui.button("↩ Restore This Version").clicked() {
                        app.variant_manager.rollback_to_version(&app.db, variant_id, version_number);
                        app.variant_manager.show_view_version = None;
                    }

                if ui.button("Close").clicked() {
                    app.variant_manager.show_view_version = None;
                }
            });
        });
}
