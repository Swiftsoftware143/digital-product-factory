//! Product creation view
//!
//! Wired to the real backend:
//!   * category filter  -> filters `get_template_registry().list()`
//!   * template `Select` -> stores the id and loads that template's parameters
//!   * `Preview Prompt`  -> `ProductGenerator::preview_template`
//!   * `⚡ Generate`     -> `ProductGenerator::generate_blocking`
//!   * parameter form    -> `#[2. Configure Parameters]` edits the per-template
//!     value map that is serialised into the `params` object handed to the backend.

use egui::*;
use crate::app::DpfApp;
use crate::templates::{ParameterType, Template, TemplateCategory};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Create Product");
        ui.separator();

        // Check if API keys are configured
        if app.config.openai_key.is_empty() && app.config.anthropic_key.is_empty() {
            ui.group(|ui| {
                ui.label(RichText::new("⚠️ API Keys Required").color(Color32::YELLOW));
                ui.label("Please configure your API keys in Settings to generate products.");
                if ui.button("Open Settings").clicked() {
                    app.show_settings = true;
                }
            });
            return;
        }

        // Snapshot the registry (owned) so we can freely mutate `app.create_state`
        // below without fighting the borrow checker over `app.generator`.
        let mut all_templates: Vec<Template> = app
            .generator
            .get_template_registry()
            .list()
            .into_iter()
            .cloned()
            .collect();
        // Deterministic ordering: most trending first, then name.
        all_templates.sort_by(|a, b| {
            b.trending_score
                .cmp(&a.trending_score)
                .then_with(|| a.name.cmp(&b.name))
        });

        // ------------------------------------------------------------------
        // 1. Select Template
        // ------------------------------------------------------------------
        ui.group(|ui| {
            ui.heading("1. Select Template");

            // ---- Category filter (real filtering) ----
            ui.horizontal(|ui| {
                ui.label("Category:");
                let showing_all = app.create_state.category_filter.is_none();
                if ui.selectable_label(showing_all, "All").clicked() {
                    app.create_state.category_filter = None;
                }
                for category in TemplateCategory::all() {
                    let name = category.name();
                    let active = app.create_state.category_filter.as_deref() == Some(name);
                    if ui.selectable_label(active, name).clicked() {
                        // Click the active category again to clear the filter.
                        app.create_state.category_filter =
                            if active { None } else { Some(name.to_string()) };
                    }
                }
            });

            ui.separator();

            // ---- Template grid (filtered) ----
            let mut pending_selection: Option<String> = None;
            ScrollArea::vertical()
                .id_source("create_template_list")
                .max_height(300.0)
                .show(ui, |ui| {
                    let filter = app.create_state.category_filter.clone();
                    let mut shown = 0usize;

                    for template in &all_templates {
                        if let Some(cat) = &filter {
                            if template.category.name() != cat {
                                continue;
                            }
                        }
                        shown += 1;

                        let is_selected = app.create_state.selected_template.as_deref()
                            == Some(template.id.as_str());

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    let title = if is_selected {
                                        RichText::new(&template.name)
                                            .strong()
                                            .color(Color32::LIGHT_GREEN)
                                    } else {
                                        RichText::new(&template.name).strong()
                                    };
                                    ui.label(title);
                                    ui.label(
                                        RichText::new(&template.description)
                                            .size(12.0)
                                            .color(Color32::GRAY),
                                    );
                                    ui.horizontal(|ui| {
                                        for tag in &template.tags {
                                            ui.label(
                                                RichText::new(format!("# {}", tag))
                                                    .size(10.0)
                                                    .color(Color32::LIGHT_BLUE),
                                            );
                                        }
                                    });
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let label =
                                        if is_selected { "Selected" } else { "Select" };
                                    if ui.button(label).clicked() {
                                        pending_selection = Some(template.id.clone());
                                    }
                                    ui.label(format!("🔥 {}", template.trending_score));
                                });
                            });
                        });
                        ui.add_space(8.0);
                    }

                    if shown == 0 {
                        ui.label(
                            RichText::new("No templates in this category.")
                                .color(Color32::GRAY),
                        );
                    }
                });

            // Apply the selection after the immutable borrow of `all_templates`
            // inside the ScrollArea closure has ended.
            if let Some(id) = pending_selection {
                select_template(app, &all_templates, &id);
            }
        });

        ui.separator();

        // ------------------------------------------------------------------
        // 2. Configure Parameters
        // ------------------------------------------------------------------
        ui.group(|ui| {
            ui.heading("2. Configure Parameters");
            match app.create_state.selected_template.clone() {
                None => {
                    ui.label("Select a template above to configure parameters");
                }
                Some(id) => {
                    match all_templates.iter().find(|t| t.id == id) {
                        None => {
                            ui.label(
                                RichText::new("Selected template no longer exists.")
                                    .color(Color32::RED),
                            );
                        }
                        Some(template) => {
                            ui.label(format!("Template: {}", template.name));
                            ui.label(
                                RichText::new(&template.description)
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );
                            ui.separator();

                            let params = template.parameters.clone();
                            if params.is_empty() {
                                ui.label("This template has no parameters.");
                            }

                            for param in &params {
                                let mut value = app
                                    .create_state
                                    .param_values
                                    .get(&param.name)
                                    .cloned()
                                    .unwrap_or_default();

                                ui.horizontal(|ui| {
                                    let label = if param.required {
                                        format!("{} *", param.name)
                                    } else {
                                        param.name.clone()
                                    };
                                    ui.label(label).on_hover_text(&param.description);

                                    match &param.param_type {
                                        ParameterType::Text
                                        | ParameterType::Number
                                        | ParameterType::Date => {
                                            ui.add(
                                                TextEdit::singleline(&mut value)
                                                    .desired_width(280.0),
                                            );
                                        }
                                        ParameterType::Color => {
                                            ui.add(
                                                TextEdit::singleline(&mut value)
                                                    .desired_width(120.0),
                                            );
                                        }
                                        ParameterType::Boolean => {
                                            let mut checked =
                                                value.eq_ignore_ascii_case("true");
                                            if ui.checkbox(&mut checked, "").changed() {
                                                value = if checked {
                                                    "true".to_string()
                                                } else {
                                                    "false".to_string()
                                                };
                                            }
                                        }
                                        ParameterType::Select(options) => {
                                            let text = if value.is_empty() {
                                                "(select)".to_string()
                                            } else {
                                                value.clone()
                                            };
                                            ComboBox::from_id_source(&param.name)
                                                .selected_text(text)
                                                .width(280.0)
                                                .show_ui(ui, |ui| {
                                                    for opt in options {
                                                        ui.selectable_value(
                                                            &mut value,
                                                            opt.clone(),
                                                            opt.as_str(),
                                                        );
                                                    }
                                                });
                                        }
                                    }

                                    if param.required && value.trim().is_empty() {
                                        ui.label(
                                            RichText::new("required")
                                                .size(10.0)
                                                .color(Color32::RED),
                                        );
                                    }
                                });

                                app.create_state
                                    .param_values
                                    .insert(param.name.clone(), value);
                            }
                        }
                    }
                }
            }
        });

        ui.separator();

        // ------------------------------------------------------------------
        // 3. Generate
        // ------------------------------------------------------------------
        ui.group(|ui| {
            ui.heading("3. Generate");

            // Status line from the previous action this frame / last frame.
            if let Some(status) = app.create_state.status.clone() {
                let color = if status.starts_with('❌') {
                    Color32::RED
                } else if status.starts_with('✅') {
                    Color32::LIGHT_GREEN
                } else {
                    Color32::LIGHT_BLUE
                };
                ui.label(RichText::new(status).color(color));
            }

            let selected = app.create_state.selected_template.clone();
            let has_selection = selected.is_some();

            ui.horizontal(|ui| {
                let preview_clicked = ui
                    .add_enabled(has_selection, Button::new("Preview Prompt"))
                    .clicked();

                let generate_label = if app.create_state.is_generating {
                    "⚡ Generating…"
                } else {
                    "⚡ Generate Product"
                };
                let generate_clicked = ui
                    .add_enabled(
                        has_selection && !app.create_state.is_generating,
                        Button::new(RichText::new(generate_label).strong()),
                    )
                    .clicked();

                if preview_clicked {
                    if let Some(id) = &selected {
                        let params = collect_params(app);
                        match app.generator.preview_template(id, &params) {
                            Ok(prompt) => {
                                app.create_state.prompt_preview = Some(prompt);
                                app.create_state.status =
                                    Some("Prompt preview generated.".to_string());
                            }
                            Err(e) => {
                                app.create_state.prompt_preview = None;
                                app.create_state.status =
                                    Some(format!("❌ Preview failed: {}", e));
                            }
                        }
                    }
                }

                if generate_clicked {
                    if let Some(id) = &selected {
                        // Validate required parameters locally before hitting the backend.
                        match validate_required(app, &all_templates, id) {
                            Err(msg) => {
                                app.create_state.status = Some(format!("❌ {}", msg));
                            }
                            Ok(()) => {
                                let params = collect_params(app);
                                app.create_state.is_generating = true;
                                app.create_state.status =
                                    Some("Generating…".to_string());
                                // Synchronous backend call; `is_generating` brackets it.
                                let result = app.generator.generate_blocking(id, params);
                                app.create_state.is_generating = false;
                                match result {
                                    Ok(product) => {
                                        app.create_state.status = Some(format!(
                                            "✅ Generated '{}' using {} ({} tokens, {} ms)",
                                            product.name,
                                            product.metadata.model_used,
                                            product.metadata.tokens_used,
                                            product.metadata.generation_time_ms,
                                        ));
                                        app.create_state.last_product = Some(product);
                                    }
                                    Err(e) => {
                                        app.create_state.status =
                                            Some(format!("❌ Generation failed: {}", e));
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // ---- Prompt preview (read-only) ----
            if let Some(preview) = app.create_state.prompt_preview.clone() {
                ui.separator();
                ui.label(RichText::new("Prompt Preview").strong());
                ScrollArea::vertical()
                    .id_source("create_prompt_preview")
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let mut text = preview;
                        ui.add(
                            TextEdit::multiline(&mut text)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
            }

            // ---- Generated product result (read-only) ----
            if let Some(product) = app.create_state.last_product.clone() {
                ui.separator();
                ui.label(
                    RichText::new(format!("Generated Product: {}", product.name)).strong(),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Model: {}", product.metadata.model_used));
                    ui.label(format!("Tokens: {}", product.metadata.tokens_used));
                    ui.label(format!("Time: {} ms", product.metadata.generation_time_ms));
                    ui.label(format!("Format: {:?}", product.format));
                });
                ScrollArea::vertical()
                    .id_source("create_result_content")
                    .max_height(320.0)
                    .show(ui, |ui| {
                        let mut content = product.content.clone();
                        ui.add(
                            TextEdit::multiline(&mut content)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
            }
        });
    });
}

/// Store the chosen template id and seed the parameter values from the
/// template's declared defaults.
fn select_template(app: &mut DpfApp, templates: &[Template], id: &str) {
    if let Some(t) = templates.iter().find(|t| t.id == id) {
        app.create_state.selected_template = Some(t.id.clone());
        let mut values = std::collections::HashMap::new();
        for p in &t.parameters {
            values.insert(p.name.clone(), p.default.clone().unwrap_or_default());
        }
        app.create_state.param_values = values;
        app.create_state.prompt_preview = None;
        app.create_state.last_product = None;
        app.create_state.status = Some(format!("Selected template: {}", t.name));
    }
}

/// Assemble the edited parameter values into the `serde_json::Value::Object`
/// the generator's `build_prompt` expects (keyed by parameter name).
fn collect_params(app: &DpfApp) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &app.create_state.param_values {
        map.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    serde_json::Value::Object(map)
}

/// Reject generation when a `required` parameter is empty so the user gets a
/// clear message instead of the backend's `Err`.
fn validate_required(app: &DpfApp, templates: &[Template], id: &str) -> Result<(), String> {
    let template = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| "Selected template no longer exists".to_string())?;
    for p in &template.parameters {
        if p.required {
            let val = app
                .create_state
                .param_values
                .get(&p.name)
                .map(|s| s.trim())
                .unwrap_or("");
            if val.is_empty() {
                return Err(format!("Required parameter '{}' is empty", p.name));
            }
        }
    }
    Ok(())
}
