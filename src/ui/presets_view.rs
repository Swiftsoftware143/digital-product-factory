//! Industry Presets View - Pre-configured workflows for different business models

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::presets::{IndustryPreset, PresetStage, ModuleType};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("🎯 Industry Presets");
        ui.label("Choose a pre-configured workflow for your business model:");
        ui.add_space(20.0);
        
        // Get all presets
        let presets: Vec<_> = app.preset_registry.list()
            .into_iter()
            .cloned()
            .collect();
        
        // Display presets in a grid
        let available_width = ui.available_width();
        let card_width = 300.0;
        let columns = (available_width / card_width).max(1.0) as usize;
        
        ScrollArea::vertical().show(ui, |ui| {
            for chunk in presets.chunks(columns) {
                ui.horizontal(|ui| {
                    for preset in chunk {
                        show_preset_card(app, ui, preset);
                    }
                });
                ui.add_space(10.0);
            }
            
            ui.add_space(30.0);
            
            // Show selected preset details if any
            if let Some(selected_id) = &app.selected_preset_id {
                let preset = app.preset_registry.get(selected_id).cloned();
                if let Some(preset) = preset {
                    show_preset_details(app, ui, &preset);
                }
            }
        });
    });
}

fn show_preset_card(app: &mut DpfApp, ui: &mut Ui, preset: &IndustryPreset) {
    let is_selected = app.selected_preset_id.as_ref() == Some(&preset.id);
    
    Frame::group(ui.style())
        .fill(if is_selected { Color32::from_rgb(40, 60, 80) } else { ui.style().visuals.panel_fill })
        .show(ui, |ui| {
            ui.set_min_width(280.0);
            ui.set_max_width(280.0);
            
            ui.horizontal(|ui| {
                ui.label(RichText::new(&preset.emoji).size(32.0));
                ui.vertical(|ui| {
                    ui.strong(&preset.name);
                    ui.label(RichText::new(&preset.description).size(11.0).color(Color32::GRAY));
                });
            });
            
            ui.add_space(10.0);
            
            // Show stage count
            ui.label(format!("{} stages", preset.stages.len()));
            
            // Show best for tags
            ui.horizontal_wrapped(|ui| {
                for best in preset.best_for.iter().take(3) {
                    ui.label(RichText::new(format!("• {}", best)).size(10.0).color(Color32::LIGHT_BLUE));
                }
            });
            
            ui.add_space(10.0);
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("View Details").clicked() {
                    app.selected_preset_id = Some(preset.id.clone());
                }
                
                if ui.button("Load Pipeline").clicked() {
                    load_preset_into_pipeline(app, preset);
                }
            });
        });
}

fn show_preset_details(app: &mut DpfApp, ui: &mut Ui, preset: &IndustryPreset) {
    ui.separator();
    ui.heading(format!("{} {}", preset.emoji, preset.name));
    ui.label(&preset.description);
    ui.add_space(15.0);
    
    // Best for section
    ui.collapsing("✅ Best For", |ui| {
        for best in &preset.best_for {
            ui.label(format!("• {}", best));
        }
    });
    
    ui.add_space(10.0);
    
    // Pipeline stages
    ui.heading("📋 Pipeline Stages");
    ui.add_space(10.0);
    
    for (i, stage) in preset.stages.iter().enumerate() {
        show_stage_card(ui, stage, i + 1);
        ui.add_space(8.0);
    }
    
    ui.add_space(15.0);
    
    // Quick tips
    ui.collapsing("💡 Quick Tips", |ui| {
        for tip in &preset.quick_tips {
            ui.label(RichText::new(format!("• {}", tip)).size(12.0).color(Color32::LIGHT_YELLOW));
        }
    });
    
    ui.add_space(20.0);
    
    // Load button
    if ui.button(RichText::new(format!("🚀 Load '{}' Pipeline", preset.name)).size(16.0)).clicked() {
        load_preset_into_pipeline(app, preset);
    }
}

fn show_stage_card(ui: &mut Ui, stage: &PresetStage, number: usize) {
    Frame::group(ui.style())
        .fill(Color32::from_gray(35))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width() - 20.0);
            
            ui.horizontal(|ui| {
                // Stage number and emoji
                ui.label(RichText::new(format!("{} {}", stage.emoji, number)).size(18.0));
                
                ui.vertical(|ui| {
                    ui.strong(&stage.name);
                    ui.label(RichText::new(&stage.description).size(11.0).color(Color32::GRAY));
                });
            });
            
            ui.add_space(8.0);
            
            // Actions
            ui.label("Actions:");
            for action in &stage.actions {
                ui.label(RichText::new(format!("  → {}", action)).size(11.0));
            }
            
            ui.add_space(5.0);
            
            // Recommended modules
            ui.horizontal_wrapped(|ui| {
                ui.label("Modules:");
                for module in &stage.recommended_modules {
                    ui.label(RichText::new(format!("{} {}", module.icon(), module.name())).size(10.0).color(Color32::LIGHT_GREEN));
                }
            });
            
            ui.add_space(5.0);
            
            // Output
            ui.label(RichText::new(format!("📤 Output: {}", stage.output_description)).size(11.0).color(Color32::LIGHT_BLUE));
        });
}

fn load_preset_into_pipeline(app: &mut DpfApp, preset: &IndustryPreset) {
    // Create sample ideas for each stage in the preset
    for (i, stage) in preset.stages.iter().enumerate() {
        let idea = crate::pipeline::ProductIdea {
            id: app.pipeline.ideas.len() + i,
            title: format!("{}: Sample {}", stage.emoji, stage.name),
            description: stage.description.clone(),
            stage: crate::pipeline::PipelineStage::Idea, // Start all in Idea stage
            product_type: preset.name.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            priority: crate::pipeline::Priority::Medium,
            tags: vec![preset.id.clone(), stage.name.to_lowercase().as_str().replace(" ", "-")],
            estimated_value: 0.0,
            actual_value: None,
            notes: format!(
                "Preset: {}\n\nStage: {}\n\nActions:\n{}\n\nOutput: {}",
                preset.name,
                stage.name,
                stage.actions.join("\n"),
                stage.output_description
            ),
            platform: vec![],
        };
        
        app.pipeline.add_idea(&app.db, idea);
    }
    
    // Switch to pipeline tab
    app.current_tab = Tab::Pipeline;
    
    // Store the loaded preset for reference
    app.loaded_preset_id = Some(preset.id.clone());
}
