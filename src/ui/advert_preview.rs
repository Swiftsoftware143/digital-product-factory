//! Advert Preview — Preview Rendering
//!
//! Renders an in-app preview of the advert with aspect ratio guides,
//! copy positioning, and visual concept layout. Uses egui painting
//! primitives for fast display.

use egui::*;
use crate::adverts::{Advert, AspectRatio, TextPosition};
use crate::app::DpfApp;

/// Show the advert preview panel
pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("👁️ Advert Preview");
        ui.separator();

        let campaign = match &app.adverts_manager.campaign {
            Some(ref c) => c,
            None => {
                ui.label("No campaign loaded. Generate one from the Adverts tab.");
                return;
            }
        };

        // Select advert to preview
        let advert_names: Vec<String> = campaign.adverts.iter()
            .map(|a| a.name.clone())
            .collect();
        let selected = app.adverts_manager.selected_preview;

        ui.horizontal(|ui| {
            ui.label("Preview:");
            ComboBox::new("preview_selector", "")
                .selected_text(
                    selected.and_then(|i| advert_names.get(i)).map(|s| s.as_str()).unwrap_or("Select...")
                )
                .show_ui(ui, |ui| {
                    for (i, name) in advert_names.iter().enumerate() {
                        let selected = selected == Some(i);
                        if ui.selectable_label(selected, name).clicked() {
                            app.adverts_manager.selected_preview = Some(i);
                        }
                    }
                });
        });

        if let Some(idx) = selected {
            if let Some(advert) = campaign.adverts.get(idx) {
                render_preview(ui, advert);
            }
        } else {
            ui.label("Select an advert to preview.");
        }
    });
}

/// Render the advert preview using egui painting
fn render_preview(ui: &mut Ui, advert: &Advert) {
    let (width, height) = advert.aspect_ratio.dimensions();

    // Scale preview to fit UI (max ~400px wide)
    let max_preview_width = 400.0;
    let scale = max_preview_width / width as f32;
    let preview_width = width as f32 * scale;
    let preview_height = height as f32 * scale;

    Frame::dark_canvas(ui.style()).show(ui, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            vec2(preview_width, preview_height),
            Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            // Background fill (color scheme approximation)
            let bg_color = match advert.layout_spec.color_scheme {
                crate::adverts::ColorScheme::Light => Color32::from_rgb(248, 248, 248),
                crate::adverts::ColorScheme::Dark => Color32::from_rgb(30, 30, 30),
                crate::adverts::ColorScheme::Vibrant => Color32::from_rgb(255, 235, 59),
                crate::adverts::ColorScheme::Monochrome => Color32::from_gray(200),
                crate::adverts::ColorScheme::Default => Color32::from_rgb(240, 240, 245),
            };
            painter.rect_filled(rect, 0.0, bg_color);

            // Aspect ratio guide border
            let border_color = Color32::from_rgb(100, 100, 100);
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, border_color));

            // Brand name — top area
            if !advert.brand_identity.brand_name.is_empty() {
                let brand_rect = Rect::from_min_size(
                    rect.min + vec2(8.0, 8.0),
                    vec2(rect.width() - 16.0, 24.0),
                );
                painter.text(
                    brand_rect.center_top(),
                    Align2::LEFT_TOP,
                    &advert.brand_identity.brand_name,
                    FontId::proportional(14.0),
                    Color32::from_rgb(80, 80, 80),
                );
            }

            // Headline — positioned according to layout spec
            let headline_y = match advert.layout_spec.headline_position {
                TextPosition::Top => rect.top() + rect.height() * 0.08,
                TextPosition::Middle => rect.top() + rect.height() * 0.35,
                TextPosition::Bottom => rect.top() + rect.height() * 0.70,
                TextPosition::Center => rect.top() + rect.height() * 0.40,
                TextPosition::Left | TextPosition::Right => rect.top() + rect.height() * 0.08,
            };

            if !advert.headline.is_empty() {
                let headline_rect = Rect::from_min_size(
                    pos2(rect.left() + 12.0, headline_y),
                    vec2(rect.width() - 24.0, 36.0),
                );
                painter.text(
                    headline_rect.min,
                    Align2::LEFT_TOP,
                    &advert.headline,
                    FontId::proportional(22.0),
                    Color32::from_rgb(20, 20, 20),
                );
            }

            // Subheadline
            if !advert.subheadline.is_empty() {
                let sub_y = headline_y + 40.0;
                let sub_rect = Rect::from_min_size(
                    pos2(rect.left() + 12.0, sub_y),
                    vec2(rect.width() - 24.0, 24.0),
                );
                painter.text(
                    sub_rect.min,
                    Align2::LEFT_TOP,
                    &advert.subheadline,
                    FontId::proportional(16.0),
                    Color32::from_rgb(100, 100, 100),
                );
            }

            // Product placement (visual indicator)
            let (px, py) = (
                rect.left() + rect.width() * advert.product_placement.position_x / 100.0,
                rect.top() + rect.height() * advert.product_placement.position_y / 100.0,
            );
            let product_size = 60.0 * advert.product_placement.scale_percent / 100.0;
            let product_rect = Rect::from_center_size(
                pos2(px, py),
                vec2(product_size, product_size * 0.75),
            );

            // Draw product placeholder with shadow
            if advert.product_placement.shadow_enabled {
                let shadow_rect = product_rect.translate(vec2(3.0, 3.0));
                painter.rect_filled(shadow_rect, 4.0, Color32::from_black_alpha(40));
            }
            painter.rect_filled(product_rect, 4.0, Color32::from_rgb(66, 133, 244));
            painter.text(
                product_rect.center(),
                Align2::CENTER_CENTER,
                "📦",
                FontId::proportional(18.0),
                Color32::WHITE,
            );

            // CTA button area — bottom
            let cta_y = match advert.layout_spec.headline_position {
                TextPosition::Bottom => rect.bottom() - rect.height() * 0.15,
                _ => rect.bottom() - 50.0,
            };

            if !advert.call_to_action.is_empty() {
                let cta_rect = Rect::from_min_size(
                    pos2(rect.center().x - 80.0, cta_y),
                    vec2(160.0, 36.0),
                );
                painter.rect_filled(cta_rect, 18.0, Color32::from_rgb(233, 69, 96));
                painter.text(
                    cta_rect.center(),
                    Align2::CENTER_CENTER,
                    &advert.call_to_action,
                    FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }

            // Conversion score badge
            let score_color = if advert.conversion_score >= 80 {
                Color32::GREEN
            } else if advert.conversion_score >= 60 {
                Color32::YELLOW
            } else {
                Color32::RED
            };
            let score_rect = Rect::from_min_size(
                rect.right_top() - vec2(65.0, 0.0),
                vec2(65.0, 22.0),
            );
            painter.rect_filled(score_rect, 4.0, Color32::from_black_alpha(60));
            painter.text(
                score_rect.center(),
                Align2::CENTER_CENTER,
                format!("Score: {}/100", advert.conversion_score),
                FontId::proportional(10.0),
                score_color,
            );

            // Aspect ratio label bottom-left
            painter.text(
                rect.left_bottom() + vec2(4.0, -4.0),
                Align2::LEFT_BOTTOM,
                format!("{} | {}x{}", advert.aspect_ratio.label(), width, height),
                FontId::proportional(9.0),
                Color32::from_gray(120),
            );
        }

        // Click handler (no-op for now)
        let _ = response;
    });
}
