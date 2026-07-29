//! Advert Composer — Composition Form
//!
//! Provides editable fields for all advert properties.
//! After AI generation, users can fine-tune every aspect.

use egui::*;
use crate::adverts::{
    Advert, AdvertStatus, AspectRatio, BrandIdentity, Campaign,
    CopyFramework, GenerationConfig, LayoutSpec, ProductPlacement,
    TextPosition, BackgroundStyle, ColorScheme,
};
use crate::app::DpfApp;

/// Show the advert composition form in a scrollable panel
pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("✏️ Advert Composer");
        ui.separator();

        let campaign = match &mut app.adverts_manager.campaign {
            Some(ref mut c) => c,
            None => {
                ui.label("No campaign loaded. Generate one from the Adverts tab first.");
                return;
            }
        };

        // Campaign-level info (editable)
        ui.group(|ui| {
            ui.label("Campaign Info");
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut campaign.name);
            });
            ui.horizontal(|ui| {
                ui.label("Target Audience:");
                ui.text_edit_singleline(&mut campaign.target_audience);
            });
            ui.horizontal(|ui| {
                ui.label("Platform:");
                ui.text_edit_singleline(&mut campaign.platform);
            });
        });

        ui.separator();

        // Select which advert to edit
        let advert_names: Vec<String> = campaign.adverts.iter()
            .map(|a| a.name.clone())
            .collect();
        let selected = app.adverts_manager.selected_advert;

        ui.horizontal(|ui| {
            ui.label("Edit Advert:");
            ComboBox::new("advert_selector", "")
                .selected_text(
                    selected.and_then(|i| advert_names.get(i)).map(|s| s.as_str()).unwrap_or("Select...")
                )
                .show_ui(ui, |ui| {
                    for (i, name) in advert_names.iter().enumerate() {
                        let selected = selected == Some(i);
                        if ui.selectable_label(selected, name).clicked() {
                            app.adverts_manager.selected_advert = Some(i);
                        }
                    }
                });
        });

        let selected = app.adverts_manager.selected_advert;
        if let Some(idx) = selected {
            if let Some(advert) = campaign.adverts.get_mut(idx) {
                show_advert_editor(ui, advert);
            }
        } else {
            ui.label("Select an advert above to edit its properties.");
        }
    });
}

/// Editable fields for a single advert
fn show_advert_editor(ui: &mut Ui, advert: &mut Advert) {
    ScrollArea::vertical().show(ui, |ui| {
        // ── Aspect Ratio ────────────────────────────────────────────
        ui.group(|ui| {
            ui.label("Aspect Ratio");
            ui.horizontal(|ui| {
                for ar in AspectRatio::all() {
                    let selected = advert.aspect_ratio == ar;
                    if ui.selectable_label(selected, ar.label()).clicked() {
                        advert.aspect_ratio = ar;
                    }
                }
            });
        });

        // ── Headline & Copy ─────────────────────────────────────────
        ui.group(|ui| {
            ui.label("Copy");
            ui.horizontal(|ui| {
                ui.label("Headline:");
                ui.add(TextEdit::singleline(&mut advert.headline));
            });
            ui.horizontal(|ui| {
                ui.label("Subheadline:");
                ui.add(TextEdit::singleline(&mut advert.subheadline));
            });
            ui.label("Body Copy:");
            ui.add(TextEdit::multiline(&mut advert.body_copy).desired_rows(4));
            ui.horizontal(|ui| {
                ui.label("CTA:");
                ui.add(TextEdit::singleline(&mut advert.call_to_action));
            });

            // Copy framework selector
            ui.horizontal(|ui| {
                ui.label("Framework:");
                for fw in CopyFramework::all() {
                    let selected = advert.copy_framework == fw;
                    if ui.selectable_label(selected, fw.label()).clicked() {
                        advert.copy_framework = fw;
                    }
                }
            });
        });

        // ── Visual ──────────────────────────────────────────────────
        ui.group(|ui| {
            ui.label("Visual");
            ui.label("Visual Description:");
            ui.add(TextEdit::multiline(&mut advert.visual_description).desired_rows(3));
        });

        // ── Brand Identity ──────────────────────────────────────────
        ui.group(|ui| {
            ui.label("Brand Identity");
            ui.horizontal(|ui| {
                ui.label("Brand Name:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.brand_name));
            });
            ui.horizontal(|ui| {
                ui.label("Tagline:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.tagline));
            });
            ui.horizontal(|ui| {
                ui.label("Voice Tone:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.voice_tone));
            });
            ui.horizontal(|ui| {
                ui.label("Primary Color:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.primary_color).desired_width(80.0));
                ui.label("Secondary:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.secondary_color).desired_width(80.0));
                ui.label("Accent:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.accent_color).desired_width(80.0));
            });
            ui.horizontal(|ui| {
                ui.label("Font:");
                ui.add(TextEdit::singleline(&mut advert.brand_identity.font_family));
            });
        });

        // ── Product Placement ──────────────────────────────────────
        ui.group(|ui| {
            ui.label("Product Placement");
            ui.horizontal(|ui| {
                ui.label("Product:");
                ui.add(TextEdit::singleline(&mut advert.product_placement.product_name));
            });
            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.add(Slider::new(&mut advert.product_placement.scale_percent, 10.0..=100.0).text("%"));
            });
            ui.horizontal(|ui| {
                ui.label("X Position:");
                ui.add(Slider::new(&mut advert.product_placement.position_x, 0.0..=100.0).text("%"));
                ui.label("Y Position:");
                ui.add(Slider::new(&mut advert.product_placement.position_y, 0.0..=100.0).text("%"));
            });
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.add(Slider::new(&mut advert.product_placement.rotation_degrees, -180.0..=180.0).text("°"));
            });
            ui.checkbox(&mut advert.product_placement.shadow_enabled, "Enable Shadow");
        });

        // ── Layout ──────────────────────────────────────────────────
        ui.group(|ui| {
            ui.label("Layout");
            ui.horizontal(|ui| {
                ui.label("Headline Position:");
                ComboBox::new("headline_pos", "")
                    .selected_text(format!("{:?}", advert.layout_spec.headline_position))
                    .show_ui(ui, |ui| {
                        for &p in &[TextPosition::Top, TextPosition::Middle, TextPosition::Bottom,
                                    TextPosition::Left, TextPosition::Right, TextPosition::Center] {
                            if ui.selectable_label(advert.layout_spec.headline_position == p, format!("{:?}", p)).clicked() {
                                advert.layout_spec.headline_position = p;
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Background Style:");
                ComboBox::new("bg_style", "")
                    .selected_text(format!("{:?}", advert.layout_spec.background_style))
                    .show_ui(ui, |ui| {
                        for &s in &[BackgroundStyle::Solid, BackgroundStyle::Gradient,
                                    BackgroundStyle::Image, BackgroundStyle::Pattern] {
                            if ui.selectable_label(advert.layout_spec.background_style == s, format!("{:?}", s)).clicked() {
                                advert.layout_spec.background_style = s;
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Color Scheme:");
                ComboBox::new("color_scheme", "")
                    .selected_text(format!("{:?}", advert.layout_spec.color_scheme))
                    .show_ui(ui, |ui| {
                        for &c in &[ColorScheme::Default, ColorScheme::Light, ColorScheme::Dark,
                                    ColorScheme::Vibrant, ColorScheme::Monochrome] {
                            if ui.selectable_label(advert.layout_spec.color_scheme == c, format!("{:?}", c)).clicked() {
                                advert.layout_spec.color_scheme = c;
                            }
                        }
                    });
            });
        });

        // ── Status & Score ──────────────────────────────────────────
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Conversion Score: {}/100", advert.conversion_score));
            if ui.button("🔄 Re-score").clicked() {
                // Placeholder — real scoring would use AI
                use rand::Rng;
                let mut rng = rand::thread_rng();
                advert.conversion_score = rng.gen_range(55..98) as u8;
            }
            if ui.button("✅ Mark Approved").clicked() {
                advert.status = AdvertStatus::Approved;
            }
        });
    });
}
