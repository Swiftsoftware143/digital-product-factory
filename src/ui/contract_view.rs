//! Contract generation view

use egui::*;
use std::collections::HashMap;
use crate::app::DpfApp;
use crate::contract_generator::{ContractCategory, FieldType};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Contract Generator");
        ui.separator();
        
        // Category selection
        ui.horizontal(|ui| {
            ui.label("Category:");
            for category in ContractCategory::all() {
                if ui.button(category.name()).clicked() {
                    // Filter by category
                }
            }
        });
        
        ui.separator();
        
        // Template selection
        ui.group(|ui| {
            ui.heading("Select Contract Type");
            
            let generator = &app.contract_generator;
            for template in generator.list_templates() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&template.name);
                            ui.label(RichText::new(&template.description).size(12.0).color(Color32::GRAY));
                        });
                        
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Select").clicked() {
                                // Open contract form
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });
        
        ui.separator();
        
        // Legal disclaimer
        ui.group(|ui| {
            ui.colored_label(Color32::YELLOW, "⚠️ Legal Disclaimer");
            ui.label("This tool generates contract templates for informational purposes only.");
            ui.label("Always consult with a qualified attorney before signing any legal document.");
        });
    });
}

#[allow(dead_code)]
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
            },
            FieldType::TextArea => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.text_edit_multiline(value);
            },
            FieldType::Number | FieldType::Currency => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.text_edit_singleline(value);
            },
            FieldType::Date => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.add(egui::TextEdit::singleline(value).hint_text("YYYY-MM-DD"));
            },
            FieldType::Select(options) => {
                let current = answers.get(&prompt.field).cloned().unwrap_or_default();
                
                ui.horizontal(|ui| {
                    for option in options {
                        if ui.selectable_label(&current == option, option).clicked() {
                            answers.insert(prompt.field.clone(), option.clone());
                        }
                    }
                });
            },
            FieldType::Email => {
                let value = answers.entry(prompt.field.clone()).or_default();
                ui.add(egui::TextEdit::singleline(value).hint_text("email@example.com"));
            },
        }
        
        ui.label(RichText::new(&prompt.help_text).size(10.0).color(Color32::GRAY));
        ui.add_space(8.0);
    }
}
