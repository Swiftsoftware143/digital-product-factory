//! Product creation view
//!
//! Layout rationale (2026-10-24, after David's feedback): the screen used to be one long
//! vertical stack — title, template list, a parameter form of arbitrary length, then the Generate
//! button, and *below that* the prompt and the result. On a normal window the action was pushed off
//! the bottom and the prompt could never be seen at the same time as the thing that produced it.
//!
//! It is now:
//!   * a STICKY action bar pinned to the top of the tab (mode, Generate, status) — always reachable
//!     no matter how long the parameter form is;
//!   * a scrollable body, so the page can never "run out of room";
//!   * two columns: what you configure on the left, what you are about to send (the prompt) and
//!     what came back on the right.
//!
//! Bulk mode lives here too. It is a variation of creating rather than a separate destination, so
//! it is a toggle on the same screen and reuses the same template selection and parameters.
//!
//! Wired to the real backend:
//!   * category filter -> filters `get_template_registry().list()`
//!   * template Select -> stores the id and loads that template's parameters
//!   * Prompt editor   -> `ProductGenerator::preview_template`, editable, sent verbatim
//!   * Generate        -> `generate_blocking_with_prompt` (single) / same per line (bulk)

use egui::*;

use crate::app::DpfApp;
use crate::templates::{ParameterType, Template, TemplateCategory};

/// Upper bound for one bulk run. Each item is a full model call, so an accidental paste of a
/// thousand lines would be an expensive mistake rather than a feature.
const BULK_MAX: usize = 20;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        // Must match EVERY provider we support. This previously tested only the OpenAI and
        // Anthropic keys, so a user whose only key was Google, DeepSeek or Moonshot was told
        // "API Keys Required" and blocked from generating — with no way to understand why.
        if app.has_no_api_key() {
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.label(RichText::new("⚠️ API Key Required").color(Color32::YELLOW));
                ui.label(
                    "Open ⚙ Settings and paste a key from any one provider — OpenAI, Anthropic, \
                     Google, DeepSeek or Moonshot — to start generating products.",
                );
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
        all_templates.sort_by(|a, b| {
            b.trending_score
                .cmp(&a.trending_score)
                .then_with(|| a.name.cmp(&b.name))
        });

        let ready = app.create_state.selected_template.is_some() && !app.create_state.is_generating;

        // ------------------------------------------------------------------
        // STICKY ACTION BAR — pinned above the scroll area, so the primary
        // action is never scrolled away from the user.
        // ------------------------------------------------------------------
        TopBottomPanel::top("create_action_bar").show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Create Product");
                ui.add_space(12.0);

                // Mode: one at a time, or a batch from a list.
                if ui
                    .selectable_label(!app.create_state.bulk_mode, "Single")
                    .clicked()
                {
                    app.create_state.bulk_mode = false;
                }
                if ui
                    .selectable_label(app.create_state.bulk_mode, "Bulk")
                    .clicked()
                {
                    app.create_state.bulk_mode = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let label = if app.create_state.is_generating {
                        "⚡ Working…"
                    } else if app.create_state.bulk_mode {
                        "⚡ Generate Batch"
                    } else {
                        "⚡ Generate Product"
                    };
                    let clicked = ui
                        .add_enabled(ready, Button::new(RichText::new(label).strong()))
                        .clicked();

                    if clicked {
                        if app.create_state.bulk_mode {
                            run_bulk(app, &all_templates);
                        } else {
                            run_single(app, &all_templates);
                        }
                    }

                    if ui.button("↺ Reset").clicked() {
                        app.create_state.prompt_preview = None;
                        app.create_state.status =
                            Some("Prompt reset to the template wording.".to_string());
                    }
                });
            });

            // Status sits in the bar too: the outcome of the last action is the one thing that
            // must not scroll out of view either.
            if let Some(status) = app.create_state.status.clone() {
                let color = if status.starts_with('❌') {
                    Color32::RED
                } else if status.starts_with('✅') {
                    Color32::LIGHT_GREEN
                } else {
                    Color32::LIGHT_BLUE
                };
                ui.label(RichText::new(status).size(11.5).color(color));
            }
            ui.add_space(4.0);
        });

        // ------------------------------------------------------------------
        // BODY — scrollable, two columns on anything wider than a phone.
        // ------------------------------------------------------------------
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let wide = ui.available_width() >= 820.0;
                if wide {
                    ui.columns(2, |cols| {
                        let (left, right) = cols.split_at_mut(1);
                        left[0].push_id("create_left", |ui| {
                            template_panel(app, ui, &all_templates);
                            ui.add_space(8.0);
                            params_panel(app, ui, &all_templates);
                        });
                        right[0].push_id("create_right", |ui| {
                            if app.create_state.bulk_mode {
                                bulk_panel(app, ui);
                            } else {
                                prompt_panel(app, ui);
                            }
                            ui.add_space(8.0);
                            result_panel(app, ui);
                        });
                    });
                } else {
                    // Narrow window: one column, but the action bar above is still pinned.
                    template_panel(app, ui, &all_templates);
                    ui.add_space(8.0);
                    params_panel(app, ui, &all_templates);
                    ui.add_space(8.0);
                    if app.create_state.bulk_mode {
                        bulk_panel(app, ui);
                    } else {
                        prompt_panel(app, ui);
                    }
                    ui.add_space(8.0);
                    result_panel(app, ui);
                }
                ui.add_space(12.0);
            });
    });
}

/// Template picker: category filter plus a scrollable list.
fn template_panel(app: &mut DpfApp, ui: &mut Ui, all_templates: &[Template]) {
    ui.group(|ui| {
        ui.label(RichText::new("1 · Choose a template").strong());

        ui.horizontal_wrapped(|ui| {
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

        let mut pending_selection: Option<String> = None;
        ScrollArea::vertical()
            .id_source("create_template_list")
            .max_height(240.0)
            .show(ui, |ui| {
                let filter = app.create_state.category_filter.clone();
                let mut shown = 0usize;

                for template in all_templates {
                    if let Some(cat) = &filter {
                        if template.category.name() != cat {
                            continue;
                        }
                    }
                    shown += 1;

                    let is_selected =
                        app.create_state.selected_template.as_deref() == Some(template.id.as_str());

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
                                        .size(11.5)
                                        .color(Color32::GRAY),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let label = if is_selected { "Selected" } else { "Select" };
                                if ui.button(label).clicked() {
                                    pending_selection = Some(template.id.clone());
                                }
                                ui.label(
                                    RichText::new(format!("🔥 {}", template.trending_score))
                                        .size(10.0)
                                        .color(Color32::GRAY),
                                );
                            });
                        });
                    });
                    ui.add_space(4.0);
                }

                if shown == 0 {
                    ui.label(RichText::new("No templates in this category.").color(Color32::GRAY));
                }
            });

        // Apply the selection after the immutable borrow of `all_templates` has ended.
        if let Some(id) = pending_selection {
            select_template(app, all_templates, &id);
        }
    });
}

/// Parameter form for the selected template.
fn params_panel(app: &mut DpfApp, ui: &mut Ui, all_templates: &[Template]) {
    ui.group(|ui| {
        ui.label(RichText::new("2 · Fill in the details").strong());
        match app.create_state.selected_template.clone() {
            None => {
                ui.label(
                    RichText::new("Pick a template first — its options appear here.")
                        .size(11.5)
                        .color(Color32::GRAY),
                );
            }
            Some(id) => match all_templates.iter().find(|t| t.id == id) {
                None => {
                    ui.label(
                        RichText::new("Selected template no longer exists.").color(Color32::RED),
                    );
                }
                Some(template) => {
                    let params = template.parameters.clone();
                    if params.is_empty() {
                        ui.label(
                            RichText::new("This template has no options — just generate.")
                                .size(11.5)
                                .color(Color32::GRAY),
                        );
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
                                        TextEdit::singleline(&mut value).desired_width(200.0),
                                    );
                                }
                                ParameterType::Color => {
                                    ui.add(
                                        TextEdit::singleline(&mut value).desired_width(100.0),
                                    );
                                }
                                ParameterType::Boolean => {
                                    let mut checked = value.eq_ignore_ascii_case("true");
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
                                        .width(200.0)
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
                                    RichText::new("required").size(10.0).color(Color32::RED),
                                );
                            }
                        });

                        app.create_state
                            .param_values
                            .insert(param.name.clone(), value);
                    }
                }
            },
        }
    });
}

/// The prompt: always shown once a template is chosen, always editable, sent verbatim.
fn prompt_panel(app: &mut DpfApp, ui: &mut Ui) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("3 · The prompt").strong());
            ui.label(
                RichText::new("edit freely — this exact text is what gets sent")
                    .size(10.0)
                    .color(Color32::GRAY),
            );
        });

        // Seed from the template the first time, then leave the user's edits alone.
        let selected = app.create_state.selected_template.clone();
        if let Some(id) = selected {
            if app.create_state.prompt_preview.is_none() {
                let params = collect_params(app);
                if let Ok(seed) = app.generator.preview_template(&id, &params) {
                    app.create_state.prompt_preview = Some(seed);
                }
            }
        }

        match app.create_state.prompt_preview.clone() {
            None => {
                ui.label(
                    RichText::new("Choose a template to see its prompt here.")
                        .size(11.5)
                        .color(Color32::GRAY),
                );
            }
            Some(preview) => {
                let mut text = preview;
                ScrollArea::vertical()
                    .id_source("create_prompt_edit")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let resp = ui.add(
                            TextEdit::multiline(&mut text)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(12),
                        );
                        if resp.changed() {
                            app.create_state.prompt_preview = Some(text.clone());
                        }
                    });

                if ui.button("📋 Copy prompt").clicked() {
                    ui.output_mut(|o| o.copied_text = text.clone());
                    app.create_state.status = Some("Prompt copied.".to_string());
                }
            }
        }
    });
}

/// Bulk mode: one product per line.
fn bulk_panel(app: &mut DpfApp, ui: &mut Ui) {
    ui.group(|ui| {
        ui.label(RichText::new("3 · Bulk — one product per line").strong());
        ui.label(
            RichText::new(
                "Each line becomes its own product. Name the thing that changes (theme, niche, \
                 audience…) and pick which option above it fills.",
            )
            .size(11.0)
            .color(Color32::GRAY),
        );

        // Which parameter each line feeds. Empty means "append the line to the prompt", which is
        // the useful default for a template that has no obvious variation field.
        let param_names: Vec<String> = app
            .create_state
            .selected_template
            .as_ref()
            .and_then(|id| {
                app.generator
                    .get_template_registry()
                    .list()
                    .into_iter()
                    .find(|t| &t.id == id)
                    .map(|t| t.parameters.iter().map(|p| p.name.clone()).collect())
            })
            .unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label("Vary:");
            let current = if app.create_state.bulk_param.is_empty() {
                "(append to prompt)".to_string()
            } else {
                app.create_state.bulk_param.clone()
            };
            ComboBox::from_id_source("bulk_param")
                .selected_text(current)
                .width(180.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.create_state.bulk_param.is_empty(), "(append to prompt)")
                        .clicked()
                    {
                        app.create_state.bulk_param.clear();
                    }
                    for name in &param_names {
                        let sel = &app.create_state.bulk_param == name;
                        if ui.selectable_label(sel, name).clicked() {
                            app.create_state.bulk_param = name.clone();
                        }
                    }
                });

            let count = app
                .create_state
                .bulk_lines
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count();
            ui.label(
                RichText::new(format!("{count} item(s) — max {BULK_MAX} per run"))
                    .size(10.5)
                    .color(if count > BULK_MAX {
                        Color32::RED
                    } else {
                        Color32::GRAY
                    }),
            );
        });

        ScrollArea::vertical()
            .id_source("bulk_lines")
            .max_height(220.0)
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut app.create_state.bulk_lines)
                        .hint_text("wedding planners\nfitness trackers\nbudget templates")
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(8),
                );
            });
    });
}

/// The most recent generated product.
fn result_panel(app: &mut DpfApp, ui: &mut Ui) {
    if let Some(product) = app.create_state.last_product.clone() {
        ui.group(|ui| {
            ui.label(
                RichText::new(format!("Result · {}", product.name))
                    .strong()
                    .color(Color32::LIGHT_GREEN),
            );
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} · {} tokens · {} ms",
                        product.metadata.model_used,
                        product.metadata.tokens_used,
                        product.metadata.generation_time_ms
                    ))
                    .size(10.5)
                    .color(Color32::GRAY),
                );
            });
            ScrollArea::vertical()
                .id_source("create_result_content")
                .max_height(260.0)
                .show(ui, |ui| {
                    let mut content = product.content.clone();
                    ui.add(
                        TextEdit::multiline(&mut content)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
        });
    }

    // Bulk log: what each line did, so a batch is auditable rather than a black box.
    if !app.create_state.bulk_log.is_empty() {
        ui.group(|ui| {
            ui.label(RichText::new("Batch log").strong());
            ScrollArea::vertical()
                .id_source("bulk_log")
                .max_height(200.0)
                .show(ui, |ui| {
                    for line in &app.create_state.bulk_log {
                        let color = if line.starts_with('❌') {
                            Color32::RED
                        } else if line.starts_with('✅') {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::LIGHT_BLUE
                        };
                        ui.label(RichText::new(line).size(11.0).color(color));
                    }
                });
        });
    }
}

/// Generate one product with the prompt currently on screen.
fn run_single(app: &mut DpfApp, all_templates: &[Template]) {
    let Some(id) = app.create_state.selected_template.clone() else {
        return;
    };

    if let Err(msg) = validate_required(app, all_templates, &id) {
        app.create_state.status = Some(format!("❌ {msg}"));
        return;
    }

    let params = collect_params(app);
    let prompt_override = app.create_state.prompt_preview.clone();
    app.create_state.is_generating = true;
    app.create_state.status = Some("Generating…".to_string());

    let result = app
        .generator
        .generate_blocking_with_prompt(&id, params, prompt_override);
    app.create_state.is_generating = false;

    match result {
        Ok(product) => {
            app.create_state.status = Some(format!(
                "✅ Generated '{}' using {} ({} tokens, {} ms)",
                product.name,
                product.metadata.model_used,
                product.metadata.tokens_used,
                product.metadata.generation_time_ms
            ));
            // Fire-and-forget: a dead endpoint must not block the UI.
            crate::webhooks::dispatch(
                &app.config.webhook,
                &app.webhook_log,
                crate::webhooks::WebhookEvent::ProductGenerated,
                serde_json::json!({
                    "name": product.name,
                    "template_id": product.template_id,
                    "model": product.metadata.model_used,
                    "tokens": product.metadata.tokens_used,
                    "format": format!("{:?}", product.format),
                }),
            );
            app.create_state.last_product = Some(product);
        }
        Err(e) => {
            app.create_state.status = Some(format!("❌ Generation failed: {e}"));
        }
    }
}

/// Generate one product per non-empty line of the bulk box.
///
/// Each line is treated as the value of the chosen parameter (or appended to the prompt when no
/// parameter is chosen), so the batch is a real variation set rather than the same product N times.
fn run_bulk(app: &mut DpfApp, all_templates: &[Template]) {
    let Some(id) = app.create_state.selected_template.clone() else {
        app.create_state.status = Some("❌ Choose a template first.".to_string());
        return;
    };

    let items: Vec<String> = app
        .create_state
        .bulk_lines
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if items.is_empty() {
        app.create_state.status =
            Some("❌ Add one item per line in the bulk box.".to_string());
        return;
    }
    if items.len() > BULK_MAX {
        app.create_state.status = Some(format!(
            "❌ {} items is over the {BULK_MAX}-per-run limit. Split the list.",
            items.len()
        ));
        return;
    }
    if let Err(msg) = validate_required(app, all_templates, &id) {
        app.create_state.status = Some(format!("❌ {msg}"));
        return;
    }

    let vary = app.create_state.bulk_param.clone();
    let base_params = collect_params(app);
    let base_prompt = app.create_state.prompt_preview.clone();

    app.create_state.is_generating = true;
    app.create_state.bulk_log.clear();

    let mut ok = 0usize;
    let mut failed = 0usize;

    for (i, item) in items.iter().enumerate() {
        app.create_state
            .bulk_log
            .push(format!("… {}/{} {item}", i + 1, items.len()));

        // Point the chosen parameter at this line, or append the line to the prompt.
        let mut params = base_params.clone();
        let prompt = if !vary.is_empty() {
            if let Some(obj) = params.as_object_mut() {
                obj.insert(
                    vary.clone(),
                    serde_json::Value::String(item.clone()),
                );
            }
            // Rebuild from the template so the variation is substituted properly.
            app.generator
                .preview_template(&id, &params)
                .ok()
                .or_else(|| base_prompt.clone())
        } else {
            base_prompt
                .clone()
                .map(|p| format!("{p}\n\nFocus this one on: {item}."))
        };

        let result = app
            .generator
            .generate_blocking_with_prompt(&id, params, prompt);

        match result {
            Ok(product) => {
                ok += 1;
                app.create_state
                    .bulk_log
                    .push(format!("✅ {item} → {}", product.name));
                app.create_state.last_product = Some(product);
            }
            Err(e) => {
                failed += 1;
                app.create_state
                    .bulk_log
                    .push(format!("❌ {item} → {e}"));
            }
        }
    }

    app.create_state.is_generating = false;
    app.create_state.status = Some(format!(
        "{} — {ok} generated, {failed} failed",
        if failed == 0 { "✅ Batch complete" } else { "⚠️ Batch finished with errors" }
    ));
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

/// Reject generation when a `required` parameter is empty so the user gets an
/// explanation instead of a model call that quietly invents the missing value.
fn validate_required(app: &DpfApp, templates: &[Template], id: &str) -> Result<(), String> {
    if let Some(t) = templates.iter().find(|t| t.id == id) {
        for p in &t.parameters {
            if p.required {
                let v = app
                    .create_state
                    .param_values
                    .get(&p.name)
                    .cloned()
                    .unwrap_or_default();
                if v.trim().is_empty() {
                    return Err(format!("'{}' is required.", p.name));
                }
            }
        }
    }
    Ok(())
}
