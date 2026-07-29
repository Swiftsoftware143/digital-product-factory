//! Logo Generator — UI tab for creating AI-generated SVG logos
//! Uses standard DPF egui pattern: show(app, ctx) with CentralPanel

use egui::{CentralPanel, Color32, Vec2, SidePanel};
use crate::app::DpfApp;
use crate::vector_generator;
use crate::vector_types::*;
use crate::ui::vector_preview;

pub fn show(app: &mut DpfApp, ctx: &egui::Context) {
    let state = &mut app.vector_state;

    CentralPanel::default().show(ctx, |ui| {
        // Left sidebar: saved logos
        SidePanel::left("logo_list")
            .resizable(true)
            .default_width(180.0)
            .show_inside(ui, |ui| {
                ui.heading("Logos");
                ui.separator();
                for logo in &state.saved_logos {
                    if ui.selectable_label(false, &logo.name).clicked() {
                        state.selected_logo_index = state.saved_logos.iter().position(|l| l.id == logo.id);
                    }
                }
                if state.saved_logos.is_empty() {
                    ui.label("No logos yet. Generate one!");
                }
            });

        // Right: params panel
        SidePanel::right("logo_params")
            .resizable(true)
            .default_width(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Parameters");
                ui.separator();
                ui.label("Brand Name:");
                ui.text_edit_singleline(&mut state.brand_name);
                ui.label("Tagline (optional):");
                ui.text_edit_singleline(&mut state.tagline);
                ui.label("Style:");
                ui.horizontal(|ui| {
                    for style in LogoStyle::all() {
                        let selected = state.selected_style == style;
                        if ui.selectable_label(selected, style.label()).clicked() {
                            state.selected_style = style.clone();
                        }
                    }
                });
                ui.label("Colors (comma-separated hex):");
                ui.text_edit_singleline(&mut state.palette_input);
                ui.label("Icon description (optional):");
                ui.text_edit_singleline(&mut state.icon_description);
                ui.add_space(8.0);

                if ui.button("✨ Generate Logo").clicked() {
                    let router = (!app.config.openai_key.is_empty()).then(|| {
                        crate::llm_router::LLMRouter::new(
                            app.config.openai_key.clone(),
                            app.config.anthropic_key.clone(),
                            app.config.google_key.clone(),
                            app.config.deepseek_key.clone(),
                            app.config.moonshot_key.clone(),
                        )
                    });
                    let colors: Vec<String> = state.palette_input.split(',')
                        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    if let Some(ref llm) = router {
                        let req = LogoGenerateRequest {
                            brand_name: state.brand_name.clone(),
                            tagline: if state.tagline.is_empty() { None } else { Some(state.tagline.clone()) },
                            style: state.selected_style.clone(),
                            palette: colors,
                            icon_description: if state.icon_description.is_empty() { None } else { Some(state.icon_description.clone()) },
                        };
                        match vector_generator::generate_logo(llm, &app.runtime, &req) {
                            Ok(gen) => {
                                state.current_logo = Some(Logo {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: state.brand_name.clone(),
                                    style: state.selected_style.clone(),
                                    brand_name: state.brand_name.clone(),
                                    tagline: state.tagline.clone(),
                                    icon_svg: gen.icon_svg,
                                    typography_svg: gen.typography_svg,
                                    full_svg: gen.full_svg,
                                    palette: gen.palette,
                                    favicon_enabled: state.favicon_enabled,
                                    favicon_package: None,
                                    status: "draft".to_string(),
                                    created_at: chrono::Utc::now().to_string(),
                                });
                            }
                            Err(e) => state.error = Some(e),
                        }
                    } else {
                        state.error = Some("LLM router not configured".to_string());
                    }
                }

                ui.checkbox(&mut state.favicon_enabled, "Auto-generate Favicon");

                if state.current_logo.is_some()
                    && ui.button("💾 Save Logo").clicked() {
                        if let Some(logo) = state.current_logo.clone() {
                            state.saved_logos.push(logo);
                        }
                    }

                if let Some(ref err) = state.error {
                    ui.colored_label(Color32::RED, err);
                }
            });

        // Center: preview area
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref logo) = state.current_logo {
                ui.heading("Preview");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    vector_preview::show_svg_preview(ui, &logo.full_svg, "Logo", Vec2::new(400.0, 500.0));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Logo Generator");
                    ui.label("Enter brand details and click Generate");
                });
            }
        });
    });
}
