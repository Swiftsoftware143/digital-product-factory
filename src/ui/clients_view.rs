//! Client Management tab — the `client_management` feature (Pro + Enterprise tiers).
//!
//! The licence system gated this slug while nothing implemented it. This is the module behind it:
//! a simple local client book (name, contact, company, status, notes) stored in SQLite.

use egui::*;

use crate::app::DpfApp;
use crate::client_manager::{Client, ClientStatus};
use crate::inline_help;

fn status_colour(status: ClientStatus) -> Color32 {
    match status {
        ClientStatus::Prospect => Color32::from_rgb(120, 170, 240),
        ClientStatus::Active => Color32::from_rgb(120, 210, 140),
        ClientStatus::Paused => Color32::from_rgb(235, 200, 120),
        ClientStatus::Closed => Color32::from_rgb(160, 160, 160),
    }
}

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("👥 Client Management");
            inline_help::help_button(ui, "clients", &mut app.active_help_topic);
        });
        ui.separator();

        // Summary
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} total", app.clients.clients.len())).strong());
            for s in ClientStatus::all() {
                ui.separator();
                ui.label(
                    RichText::new(format!("{}: {}", s.label(), app.clients.count_by(s)))
                        .color(status_colour(s)),
                );
            }
        });

        ui.separator();

        // Add / edit form
        ui.group(|ui| {
            let editing = app.client_editing.is_some();
            ui.heading(if editing { "Edit Client" } else { "Add Client" });

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(
                    TextEdit::singleline(&mut app.client_draft.name)
                        .hint_text("Required")
                        .desired_width(220.0),
                );
                ui.label("Company:");
                ui.add(
                    TextEdit::singleline(&mut app.client_draft.company)
                        .hint_text("Optional")
                        .desired_width(200.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Email:");
                ui.add(
                    TextEdit::singleline(&mut app.client_draft.email)
                        .hint_text("Optional")
                        .desired_width(220.0),
                );
                ui.label("Status:");
                ComboBox::from_id_source("client_status_combo")
                    .selected_text(app.client_draft.status.label())
                    .show_ui(ui, |ui| {
                        for s in ClientStatus::all() {
                            ui.selectable_value(&mut app.client_draft.status, s, s.label());
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Notes:");
                ui.add(
                    TextEdit::multiline(&mut app.client_draft.notes)
                        .hint_text("Anything worth remembering")
                        .desired_width(f32::INFINITY)
                        .desired_rows(2),
                );
            });

            ui.horizontal(|ui| {
                let can_save = app.client_draft.is_valid();
                if ui
                    .add_enabled(can_save, Button::new(if editing { "💾 Save" } else { "➕ Add" }))
                    .clicked()
                {
                    let draft = app.client_draft.clone();
                    let result = if let Some(id) = app.client_editing {
                        let mut c = draft;
                        c.id = id;
                        app.clients.update(c)
                    } else {
                        app.clients.add(draft).map(|_| ())
                    };
                    match result {
                        Ok(()) => {
                            app.client_draft = Client::new(0);
                            app.client_editing = None;
                            app.client_status_message = Some((false, "Saved.".into()));
                        }
                        Err(e) => app.client_status_message = Some((true, e)),
                    }
                }

                if editing && ui.button("Cancel").clicked() {
                    app.client_draft = Client::new(0);
                    app.client_editing = None;
                }

                if !can_save {
                    ui.label(
                        RichText::new("a name is required")
                            .size(11.0)
                            .color(Color32::GRAY),
                    );
                }

                if let Some((is_err, msg)) = &app.client_status_message {
                    let colour = if *is_err {
                        Color32::from_rgb(240, 120, 120)
                    } else {
                        Color32::from_rgb(120, 210, 140)
                    };
                    ui.colored_label(colour, msg);
                }
            });
        });

        ui.separator();

        // Search
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                TextEdit::singleline(&mut app.client_search)
                    .hint_text("Filter by name, email or company")
                    .desired_width(320.0),
            );
            if ui.button("Clear").clicked() {
                app.client_search.clear();
            }
        });

        ui.separator();

        // List. Cloned so the borrow ends before any mutation below.
        let rows: Vec<Client> = app
            .clients
            .search(&app.client_search)
            .into_iter()
            .cloned()
            .collect();

        if rows.is_empty() {
            ui.label("No clients yet. Add one above.");
        } else {
            let mut edit_id: Option<usize> = None;
            let mut delete_id: Option<usize> = None;

            ScrollArea::vertical().show(ui, |ui| {
                for c in &rows {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&c.name).strong().size(15.0));
                            ui.label(
                                RichText::new(c.status.label())
                                    .color(status_colour(c.status))
                                    .strong(),
                            );
                            if !c.company.is_empty() {
                                ui.label(RichText::new(format!("· {}", c.company)).weak());
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("🗑 Delete").clicked() {
                                    delete_id = Some(c.id);
                                }
                                if ui.small_button("✏ Edit").clicked() {
                                    edit_id = Some(c.id);
                                }
                            });
                        });
                        if !c.email.is_empty() {
                            ui.label(format!("✉ {}", c.email));
                        }
                        if !c.notes.is_empty() {
                            ui.label(RichText::new(&c.notes).weak());
                        }
                    });
                }
            });

            if let Some(id) = edit_id {
                if let Some(existing) = rows.iter().find(|c| c.id == id) {
                    app.client_draft = existing.clone();
                    app.client_editing = Some(id);
                    app.client_status_message = None;
                }
            }
            if let Some(id) = delete_id {
                match app.clients.delete(id) {
                    Ok(()) => {
                        if app.client_editing == Some(id) {
                            app.client_editing = None;
                            app.client_draft = Client::new(0);
                        }
                        app.client_status_message = Some((false, "Client deleted.".into()));
                    }
                    Err(e) => app.client_status_message = Some((true, e)),
                }
            }
        }
    });
}
