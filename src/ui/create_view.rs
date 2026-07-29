//! Product creation view

use egui::*;
use crate::app::DpfApp;
use crate::templates::TemplateCategory;

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
        
        // Template selection
        ui.group(|ui| {
            ui.heading("1. Select Template");
            
            // Category filter
            ui.horizontal(|ui| {
                ui.label("Category:");
                if ui.button("All").clicked() {
                    // Show all
                }
                for category in TemplateCategory::all() {
                    if ui.button(category.name()).clicked() {
                        // Filter by category
                    }
                }
            });
            
            ui.separator();
            
            // Template grid
            ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                let templates = app.generator.get_template_registry().list();
                
                for template in templates {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&template.name);
                                ui.label(RichText::new(&template.description).size(12.0).color(Color32::GRAY));
                                ui.horizontal(|ui| {
                                    for tag in &template.tags {
                                        ui.label(RichText::new(format!("# {}", tag)).size(10.0).color(Color32::LIGHT_BLUE));
                                    }
                                });
                            });
                            
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Select").clicked() {
                                    // Select this template
                                }
                                ui.label(format!("🔥 {}", template.trending_score));
                            });
                        });
                    });
                    ui.add_space(8.0);
                }
            });
        });
        
        ui.separator();
        
        // Parameter configuration
        ui.group(|ui| {
            ui.heading("2. Configure Parameters");
            ui.label("Select a template above to configure parameters");
            // TODO: Show parameter form when template selected
        });
        
        ui.separator();
        
        // Generation
        ui.group(|ui| {
            ui.heading("3. Generate");
            
            ui.horizontal(|ui| {
                if ui.button("Preview Prompt").clicked() {
                    // Show prompt preview
                }
                
                if ui.button(RichText::new("⚡ Generate Product").strong()).clicked() {
                    // Start generation
                }
            });
        });
    });
}
