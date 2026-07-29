//! Vector Generator — UI tab for AI-generated SVG icons, illustrations, etc.
//! Uses standard DPF egui pattern: show(app, ctx) with CentralPanel

use egui::{CentralPanel, Color32, Vec2, SidePanel};
use crate::app::DpfApp;
use crate::vector_generator;
use crate::vector_types::*;
use crate::ui::vector_preview;

pub fn show(app: &mut DpfApp, ctx: &egui::Context) {
    let state = &mut app.vector_state;

    CentralPanel::default().show(ctx, |ui| {
        // Left sidebar
        SidePanel::left("vector_list")
            .resizable(true)
            .default_width(180.0)
            .show_inside(ui, |ui| {
                ui.heading("Vectors");
                ui.separator();
                for asset in &state.saved_vectors {
                    if ui.selectable_label(false, &asset.name).clicked() {
                        state.selected_vector_index = state.saved_vectors.iter().position(|a| a.id == asset.id);
                    }
                }
                if state.saved_vectors.is_empty() {
                    ui.label("No vectors yet.");
                }
            });

        // Right: params
        SidePanel::right("vector_params")
            .resizable(true)
            .default_width(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Parameters");
                ui.separator();
                ui.label("Name:");
                ui.text_edit_singleline(&mut state.vector_name);
                ui.label("Category:");
                ui.horizontal(|ui| {
                    for cat in VectorCategory::all() {
                        let selected = state.selected_category == cat;
                        if ui.selectable_label(selected, cat.label()).clicked() {
                            state.selected_category = cat.clone();
                        }
                    }
                });
                ui.label("Prompt:");
                ui.text_edit_multiline(&mut state.prompt);
                ui.label("Style (optional):");
                ui.text_edit_singleline(&mut state.style_input);
                ui.label("Colors (comma-separated hex):");
                ui.text_edit_singleline(&mut state.palette_input);
                ui.add_space(8.0);

                if ui.button("🎨 Generate Vector").clicked() {
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
                        let req = VectorGenerateRequest {
                            category: state.selected_category.clone(),
                            prompt: state.prompt.clone(),
                            style: if state.style_input.is_empty() { None } else { Some(state.style_input.clone()) },
                            palette: colors,
                        };
                        match vector_generator::generate_vector(llm, &app.runtime, &req) {
                            Ok(gen) => {
                                state.current_vector = Some(VectorAsset {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: state.vector_name.clone(),
                                    category: state.selected_category.clone(),
                                    prompt: state.prompt.clone(),
                                    svg_content: gen.svg_content,
                                    palette: gen.palette,
                                    view_box: gen.view_box,
                                    export_formats: vec!["svg".to_string(), "png".to_string()],
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

                if state.current_vector.is_some()
                    && ui.button("💾 Save").clicked() {
                        if let Some(asset) = state.current_vector.clone() {
                            state.saved_vectors.push(asset);
                        }
                    }

                if let Some(ref err) = state.error {
                    ui.colored_label(Color32::RED, err);
                }
            });

        // Center: preview
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref asset) = state.current_vector {
                ui.heading(&asset.name);
                ui.separator();
                ui.label(format!("Category: {}", asset.category.label()));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    vector_preview::show_svg_preview(ui, &asset.svg_content, "Vector", Vec2::new(400.0, 400.0));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Vector Generator");
                    ui.label("Enter a prompt and click Generate");
                });
            }
        });
    });
}
