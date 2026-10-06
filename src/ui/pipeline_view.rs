//! Pipeline kanban view - optimized for speed

use egui::*;
use egui_extras::{TableBuilder, Column};
use crate::app::{DpfApp, Tab};
use crate::pipeline::{PipelineStage, ViewMode, Priority, ProductIdea};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        // Toolbar
        ui.horizontal(|ui| {
            ui.heading("Pipeline");
            // pipeline.stats() counts ideas per stage and existed with no caller, so the board
            // showed per-column counts but never a total or a per-stage summary in one place.
            {
                let st = app.pipeline.stats();
                ui.label(format!("{} idea(s) in the pipeline", st.total));
            }
            
            ui.separator();
            
            // View mode toggle
            ui.label("View:");
            if ui.selectable_label(app.pipeline.view_mode == ViewMode::Kanban, "Kanban").clicked() {
                app.pipeline.view_mode = ViewMode::Kanban;
            }
            if ui.selectable_label(app.pipeline.view_mode == ViewMode::List, "List").clicked() {
                app.pipeline.view_mode = ViewMode::List;
            }
            if ui.selectable_label(app.pipeline.view_mode == ViewMode::Calendar, "Calendar").clicked() {
                app.pipeline.view_mode = ViewMode::Calendar;
            }
            
            ui.separator();
            
            // Quick add button
            if ui.button("➕ Quick Add Idea").clicked() {
                app.pipeline.show_new_idea_dialog = true;
            }
            
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Search
                ui.add(
                    TextEdit::singleline(&mut app.pipeline.search_query)
                        .hint_text("🔍 Search ideas...")
                        .desired_width(200.0)
                );
            });
        });
        
        ui.separator();
        
        // Main content based on view mode
        match app.pipeline.view_mode {
            ViewMode::Kanban => show_kanban(app, ui),
            ViewMode::List => show_list(app, ui),
            ViewMode::Calendar => show_calendar(app, ui),
        }
    });
    
    // New idea dialog
    if app.pipeline.show_new_idea_dialog {
        show_new_idea_dialog(app, ctx);
    }
}

fn show_kanban(app: &mut DpfApp, ui: &mut Ui) {
    // Delete is requested while a card is borrowed from the board, so it is recorded here and
    // applied after every column has been drawn. Mutating the list mid-iteration would drop an
    // idea while it is still on screen.
    let mut delete_request: Option<usize> = None;

    ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for stage in PipelineStage::all() {
                ui.vertical(|ui| {
                    // Column header
                    let stage_color = stage.color();
                    let header_response = ui.group(|ui| {
                        ui.set_min_width(250.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(stage_color, stage.name());
                            let count = app.pipeline.ideas_by_stage(stage).len();
                            ui.label(format!("({})", count));
                        });
                    });
                    
                    ui.separator();
                    
                    // Drop target
                    let column_rect = ui.available_rect_before_wrap();
                    let column_response = ui.interact(column_rect, ui.id().with(stage), Sense::hover());
                    
                    // Draw column background
                    ui.painter().rect_filled(
                        column_rect,
                        0.0,
                        Color32::from_gray(30),
                    );
                    
                    // Ideas in this stage
                    let ideas: Vec<_> = app.pipeline.ideas_by_stage(stage)
                        .into_iter()
                        .cloned()
                        .collect();
                    
                    for idea in ideas {
                        show_idea_card(app, ui, &idea, &mut delete_request);
                    }
                    
                    // Handle drops
                    if let Some(dragged_id) = app.pipeline.drag_source {
                        if column_response.hovered() && app.pipeline.drag_source.is_some() {
                            app.pipeline.move_to_stage(&app.db, dragged_id, stage);
                            app.pipeline.drag_source = None;
                        }
                    }
                });
                
                ui.separator();
            }
        });
    });

    // Apply the delete now that nothing is borrowed.
    if let Some(id) = delete_request {
        app.pipeline.delete_idea(&app.db, id);
        if app.pipeline.selected_idea == Some(id) {
            app.pipeline.selected_idea = None;
        }
    }
}

fn show_idea_card(app: &mut DpfApp, ui: &mut Ui, idea: &ProductIdea, delete_request: &mut Option<usize>) {
    let card_id = ui.id().with(idea.id);
    
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(230.0);
        
        // Drag handle
        let drag_response = ui.horizontal(|ui| {
            ui.label("⋮⋮"); // Drag handle
            ui.strong(&idea.title);
        });
        
        // Make draggable
        if drag_response.response.drag_started() {
            app.pipeline.drag_source = Some(idea.id);
        }
        
        ui.label(RichText::new(&idea.description).size(12.0).color(Color32::GRAY));
        
        // Tags
        ui.horizontal_wrapped(|ui| {
            for tag in &idea.tags {
                ui.label(RichText::new(format!("# {}", tag)).size(10.0).color(Color32::LIGHT_BLUE));
            }
        });
        
        ui.separator();
        
        // Footer
        ui.horizontal(|ui| {
            // Priority indicator
            ui.colored_label(idea.priority.color(), idea.priority.name());
            
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Delete (wired: delete_idea had 0 call sites) — an idea could be created and never
                // removed, so a board filled with abandoned ideas had no way to shed them.
                if ui.small_button("🗑").on_hover_text("Delete this idea").clicked() {
                    *delete_request = Some(idea.id);
                }
                if idea.estimated_value > 0.0 {
                    ui.label(format!("${:.0}", idea.estimated_value));
                }
            });
        });
        
        // Click to select
        if drag_response.response.clicked() {
            app.pipeline.selected_idea = Some(idea.id);
        }
    });
}

fn show_list(app: &mut DpfApp, ui: &mut Ui) {
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .header(20.0, |mut header| {
            header.col(|ui| { ui.label("Stage"); });
            header.col(|ui| { ui.label("Title"); });
            header.col(|ui| { ui.label("Priority"); });
            header.col(|ui| { ui.label("Value"); });
            header.col(|ui| { ui.label("Updated"); });
        })
        .body(|mut body| {
            let ideas = app.pipeline.filtered_ideas();
            for idea in ideas {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.colored_label(idea.stage.color(), idea.stage.name());
                    });
                    row.col(|ui| {
                        ui.label(&idea.title);
                    });
                    row.col(|ui| {
                        ui.colored_label(idea.priority.color(), idea.priority.name());
                    });
                    row.col(|ui| {
                        ui.label(format!("${:.0}", idea.estimated_value));
                    });
                    row.col(|ui| {
                        ui.label(idea.updated_at.format("%m/%d").to_string());
                    });
                });
            }
        });
}

fn show_calendar(app: &mut DpfApp, ui: &mut Ui) {
    ui.label("Calendar view — scheduled drops and deadlines");
    ui.separator();

    // Built from real dated data rather than the TODO that used to sit here: scheduled tasks
    // (by their next run) and products that have reached Review or Listed.
    let mut entries: Vec<(chrono::DateTime<chrono::Utc>, String, String)> = Vec::new();

    for task in app.scheduler.tasks() {
        if let Some(next_run) = task.next_run {
            entries.push((
                next_run,
                if task.enabled { "scheduled" } else { "paused" }.to_string(),
                task.name.clone(),
            ));
        }
    }

    for idea in &app.pipeline.ideas {
        if matches!(
            idea.stage,
            crate::pipeline::PipelineStage::Review | crate::pipeline::PipelineStage::Listed
        ) {
            entries.push((
                idea.updated_at,
                format!("{:?}", idea.stage).to_lowercase(),
                idea.title.clone(),
            ));
        }
    }

    if entries.is_empty() {
        ui.label(
            "Nothing on the calendar yet. Add a task in the Scheduler tab, or move a product \
             into Review.",
        );
        return;
    }

    entries.sort_by_key(|(when, _, _)| *when);

    let mut current_day: Option<String> = None;
    for (when, kind, what) in entries {
        let day = when.format("%A %d %B %Y").to_string();
        if current_day.as_deref() != Some(day.as_str()) {
            ui.add_space(6.0);
            ui.label(RichText::new(&day).strong());
            current_day = Some(day);
        }
        ui.label(format!("    {} — {}", when.format("%H:%M"), what));
        ui.label(RichText::new(format!("        {}", kind)).size(11.0).weak());
    }
}

fn show_new_idea_dialog(app: &mut DpfApp, ctx: &Context) {
    Window::new("Quick Add Idea")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            let mut draft = app.pipeline.new_idea_draft.clone();
        let draft = &mut draft;
            
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut draft.title);
            });
            
            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_multiline(&mut draft.description);
            });
            
            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.text_edit_singleline(&mut draft.product_type);
            });
            
            ui.horizontal(|ui| {
                ui.label("Priority:");
                for priority in [Priority::Low, Priority::Medium, Priority::High, Priority::Urgent] {
                    if ui.selectable_label(draft.priority == priority, priority.name()).clicked() {
                        draft.priority = priority;
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Est. Value ($):");
                ui.text_edit_singleline(&mut draft.estimated_value);
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    app.pipeline.show_new_idea_dialog = false;
                    app.pipeline.new_idea_draft = Default::default();
                }
                
                if ui.button("Add Idea").clicked() {
                    let value = draft.estimated_value.parse().unwrap_or(0.0);
                    let idea = ProductIdea {
                        id: app.pipeline.ideas.len(),
                        title: draft.title.clone(),
                        description: draft.description.clone(),
                        stage: PipelineStage::Idea,
                        product_type: draft.product_type.clone(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        priority: draft.priority,
                        tags: vec![],
                        estimated_value: value,
                        actual_value: None,
                        notes: String::new(),
                        platform: vec![],
                    };
                    
                    app.pipeline.add_idea(&app.db, idea);
                    app.pipeline.show_new_idea_dialog = false;
                    app.pipeline.new_idea_draft = Default::default();
                }
            });
        });
}
