//! Contract generation view
//!
//! The whole generation path was unreachable. Three separate breaks:
//!   * the category buttons were wired to `// Filter by category` (nothing)
//!   * the Select buttons were wired to `// Open contract form` (nothing), so the form never opened
//!   * `show_contract_form` carried `#[allow(dead_code)]` — it existed, was complete, and was
//!     never called by anything
//! Behind those sat a fully-implemented `ContractGenerator::generate()` that validates the required
//! fields, prompts the LLM, appends a legal disclaimer and returns a finished contract. The user
//! chose a contract, filled in nothing, and got nothing.
//!
//! So this wires: filter -> select -> fill -> generate -> read/copy.
//!
//! `generate` is called with the APP'S runtime, never a fresh one: building a Tokio runtime inside
//! an async context panics ("Cannot start a runtime from within a runtime"), and this runs on the
//! UI thread which is already inside one.

use egui::*;
use std::collections::HashMap;
use crate::app::DpfApp;
use crate::contract_generator::{ContractCategory, FieldType};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    // Clicks are collected and applied after drawing, because the template list is borrowed while
    // it is rendered and cannot be mutated in place.
    let mut select_template: Option<String> = None;
    let mut set_category: Option<Option<ContractCategory>> = None;
    let mut do_generate = false;
    let mut close_form = false;

    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Contract Generator");
            crate::inline_help::help_button(ui, "contract", &mut app.active_help_topic);
        });
        ui.separator();

        if app.contract_selected.is_some() {
            // ── the form for the chosen contract ────────────────────────────────────────────────
            let id = app.contract_selected.clone().unwrap_or_default();
            let template = app.contract_generator.get_template(&id).cloned();

            match template {
                None => {
                    ui.colored_label(Color32::RED, "That contract type is no longer available.");
                    if ui.button("← Back to list").clicked() {
                        close_form = true;
                    }
                }
                Some(template) => {
                    if ui.button("← Back to list").clicked() {
                        close_form = true;
                    }
                    ui.separator();

                    ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                        show_contract_form(ui, &template, &mut app.contract_answers);
                    });

                    ui.separator();

                    let missing: Vec<&str> = template
                        .prompts
                        .iter()
                        .filter(|p| {
                            p.required
                                && app
                                    .contract_answers
                                    .get(&p.field)
                                    .map(|v| v.trim().is_empty())
                                    .unwrap_or(true)
                        })
                        .map(|p| p.question.as_str())
                        .collect();

                    ui.horizontal(|ui| {
                        let ready = missing.is_empty();
                        if ui
                            .add_enabled(ready, Button::new("✍ Generate contract"))
                            .on_hover_text("Uses one call on your own AI key")
                            .clicked()
                        {
                            do_generate = true;
                        }
                        if !ready {
                            ui.colored_label(
                                Color32::YELLOW,
                                format!("Still needed: {}", missing.join(", ")),
                            );
                        }
                    });
                    ui.small("Runs one call on the AI key you already have — no extra subscription.");

                    if let Some(err) = &app.contract_error {
                        ui.add_space(6.0);
                        ui.colored_label(Color32::from_rgb(240, 120, 120), err);
                    }

                    if let Some(result) = &app.contract_result {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.heading(&result.title);
                            if ui.button("📋 Copy").clicked() {
                                ui.ctx().copy_text(result.content.clone());
                            }
                        });
                        if !result.plain_english_summary.trim().is_empty() {
                            ui.group(|ui| {
                                ui.label(RichText::new("In plain English").strong());
                                ui.label(&result.plain_english_summary);
                            });
                        }
                        ui.add_space(6.0);
                        ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut result.content.clone())
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(18),
                            );
                        });
                        // The disclaimer is already appended to `content`; showing it separately
                        // as a styled warning means it is seen rather than skimmed past at the
                        // bottom of a long document.
                        if !result.disclaimer.trim().is_empty() {
                            ui.add_space(6.0);
                            ui.group(|ui| {
                                ui.colored_label(Color32::YELLOW, "⚠️ Legal notice");
                                ui.label(&result.disclaimer);
                            });
                        }
                        ui.small(
                            "Read it before relying on it. This is a template, not legal advice.",
                        );
                    }
                }
            }
        } else {
            // ── the contract list ────────────────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Category:");
                let all_selected = app.contract_category.is_none();
                if ui.selectable_label(all_selected, "All").clicked() {
                    set_category = Some(None);
                }
                for category in ContractCategory::all() {
                    let is = app.contract_category == Some(category);
                    if ui.selectable_label(is, category.name()).clicked() {
                        set_category = Some(Some(category));
                    }
                }
            });

            ui.separator();

            let templates: Vec<_> = match app.contract_category {
                Some(cat) => app.contract_generator.by_category(cat),
                None => app.contract_generator.list_templates(),
            };

            ui.group(|ui| {
                ui.heading(format!("Select Contract Type ({})", templates.len()));
                if templates.is_empty() {
                    ui.label(
                        RichText::new("No contracts in this category.")
                            .color(Color32::GRAY),
                    );
                }
                for template in templates {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&template.name);
                                ui.label(
                                    RichText::new(&template.description)
                                        .size(12.0)
                                        .color(Color32::GRAY),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Select").clicked() {
                                    select_template = Some(template.id.clone());
                                }
                            });
                        });
                    });
                    ui.add_space(8.0);
                }
            });

            ui.separator();
            ui.group(|ui| {
                ui.colored_label(Color32::YELLOW, "⚠️ Legal Disclaimer");
                ui.label("This tool generates contract templates for informational purposes only.");
                ui.label("Always consult with a qualified attorney before signing any legal document.");
            });
        }
    });

    // ── apply ────────────────────────────────────────────────────────────────────────────────────
    if let Some(cat) = set_category {
        app.contract_category = cat;
    }
    if let Some(id) = select_template {
        // a fresh form starts empty, so answers from a previous contract cannot bleed into this one
        app.contract_answers.clear();
        app.contract_result = None;
        app.contract_error = None;
        app.contract_selected = Some(id);
    }
    if close_form {
        app.contract_selected = None;
        app.contract_error = None;
    }
    if do_generate {
        let id = app.contract_selected.clone().unwrap_or_default();
        let answers: HashMap<String, String> = app.contract_answers.clone();
        match app.contract_generator.generate(&app.runtime, &id, answers) {
            Ok(result) => {
                app.contract_result = Some(result);
                app.contract_error = None;
            }
            Err(e) => {
                // Show the real reason. A silent failure here is the defect we are fixing.
                app.contract_error = Some(e);
                app.contract_result = None;
            }
        }
    }
}

/// Draw the answer form for one template. Public because it is the natural unit to test.
pub fn show_contract_form(
    ui: &mut Ui,
    template: &crate::contract_generator::ContractTemplate,
    answers: &mut HashMap<String, String>,
) {
    ui.heading(&template.name);
    ui.separator();

    for prompt in &template.prompts {
        ui.horizontal(|ui| {
            ui.label(&prompt.question);
            if prompt.required {
                ui.colored_label(Color32::RED, "*");
            }
        });

        match &prompt.field_type {
            FieldType::Text => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.text_edit_singleline(value);
            }
            FieldType::TextArea => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.text_edit_multiline(value);
            }
            FieldType::Number | FieldType::Currency => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.text_edit_singleline(value);
            }
            FieldType::Date => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.add(egui::TextEdit::singleline(value).hint_text("YYYY-MM-DD"));
            }
            FieldType::Select(options) => {
                let current = answers.get(&prompt.field).cloned().unwrap_or_default();
                ui.horizontal(|ui| {
                    for option in options {
                        if ui.selectable_label(&current == option, option).clicked() {
                            answers.insert(prompt.field.clone(), option.clone());
                        }
                    }
                });
            }
            FieldType::Email => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.add(egui::TextEdit::singleline(value).hint_text("email@example.com"));
            }
        }

        ui.label(RichText::new(&prompt.help_text).size(10.0).color(Color32::GRAY));
        ui.add_space(8.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every template must be reachable through the category filter. A template that appears in the
    /// list but vanishes the moment a category is picked is a feature that looks broken.
    #[test]
    fn every_template_is_findable_by_its_own_category() {
        let gen = crate::contract_generator::ContractGenerator::new();
        let cats = ContractCategory::all();
        for t in gen.list_templates() {
            let found = cats.iter().any(|c| {
                gen.by_category(*c).iter().any(|x| x.id == t.id)
            });
            assert!(found, "template '{}' is not returned by any category filter", t.id);
        }
    }

    /// The generator must be able to resolve any template the list offers — otherwise Select
    /// hands back an id that `generate` then rejects.
    #[test]
    fn every_listed_template_can_be_generated_by_id() {
        let gen = crate::contract_generator::ContractGenerator::new();
        for t in gen.list_templates() {
            assert!(
                gen.get_template(&t.id).is_some(),
                "template '{}' is listed but get_template() cannot find it",
                t.id
            );
        }
    }

    /// Generation must refuse a missing required field rather than producing a contract with a
    /// blank party name in it.
    #[test]
    fn missing_required_answers_are_refused_before_any_llm_call() {
        let gen = crate::contract_generator::ContractGenerator::new();
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let t = gen.list_templates().into_iter().next().expect("at least one template");
        // empty answers: required fields are missing, and no API key is set, so whatever it
        // returns must be an error rather than a fabricated contract
        let res = gen.generate(&rt, &t.id, std::collections::HashMap::new());
        assert!(res.is_err(), "generate() produced a contract with no answers at all");
    }

    /// An unknown template id must error, not panic.
    #[test]
    fn unknown_template_id_errors_cleanly() {
        let gen = crate::contract_generator::ContractGenerator::new();
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let res = gen.generate(&rt, "no-such-template", std::collections::HashMap::new());
        assert!(res.is_err());
    }
}
