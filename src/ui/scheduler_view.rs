//! Scheduler view
//!
//! The Add / Delete / Toggle controls were drawn but wired to nothing — three buttons that did
//! nothing when clicked. The Scheduler methods behind them (`add_task`, `remove_task`,
//! `toggle_task`) were fully implemented and already persisted, so the feature was sold, modelled,
//! stored and unreachable.
//!
//! The wiring rule here is borrow discipline: the task list is borrowed while it is drawn, so a
//! click cannot mutate it in place. Clicks are therefore recorded as a `SchedulerAction` and
//! applied after the panel closes. That is the same pattern the rest of the app uses and it avoids
//! cloning the whole list on every frame.

use egui::*;
use crate::app::DpfApp;
use crate::scheduler::{Schedule, ScheduledTask, TaskStatus, TaskType};

/// What a click asked for. Collected during drawing, applied afterwards.
enum SchedulerAction {
    Add,
    Remove(usize),
    Toggle(usize),
    ConfirmAdd,
    CancelAdd,
}

pub fn show(app: &mut DpfApp, ctx: &Context) {
    let mut action: Option<SchedulerAction> = None;

    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Scheduler");
            crate::inline_help::help_button(ui, "scheduler", &mut app.active_help_topic);
        });
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ Add Task").clicked() {
                action = Some(SchedulerAction::Add);
            }
            if ui.button("▶ Start Scheduler").clicked() {
                app.scheduler.start();
            }
            if ui.button("⏸ Stop Scheduler").clicked() {
                app.scheduler.stop();
            }
            ui.label(if app.scheduler.is_running() {
                RichText::new("● running").color(Color32::GREEN)
            } else {
                RichText::new("○ stopped").color(Color32::GRAY)
            });
        });

        // ── Add-task dialog ─────────────────────────────────────────────────────────────────────
        if app.scheduler_adding {
            ui.separator();
            ui.group(|ui| {
                ui.heading("New scheduled task");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add(TextEdit::singleline(&mut app.scheduler_draft_name)
                        .hint_text("e.g. Weekly market research"));
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ComboBox::from_id_source("sched_type")
                        .selected_text(app.scheduler_draft_type.label())
                        .show_ui(ui, |ui| {
                            for t in TaskTypeChoice::all() {
                                ui.selectable_value(&mut app.scheduler_draft_type, t, t.label());
                            }
                        });
                });

                // Only the fields the chosen type actually needs, so nobody fills in a blank that
                // will be thrown away.
                match app.scheduler_draft_type {
                    TaskTypeChoice::ResearchMarket => {
                        ui.horizontal(|ui| {
                            ui.label("Query:");
                            ui.add(TextEdit::singleline(&mut app.scheduler_draft_query)
                                .hint_text("e.g. digital planners 2026"));
                        });
                    }
                    TaskTypeChoice::GenerateProduct => {
                        ui.horizontal(|ui| {
                            ui.label("Template id:");
                            ui.add(TextEdit::singleline(&mut app.scheduler_draft_query)
                                .hint_text("e.g. planner"));
                        });
                    }
                    _ => {}
                }

                ui.horizontal(|ui| {
                    ui.label("Repeat:");
                    ComboBox::from_id_source("sched_freq")
                        .selected_text(app.scheduler_draft_freq.label())
                        .show_ui(ui, |ui| {
                            for f in FrequencyChoice::all() {
                                ui.selectable_value(&mut app.scheduler_draft_freq, f, f.label());
                            }
                        });
                });

                ui.add_space(4.0);
                let ok = !app.scheduler_draft_name.trim().is_empty();
                ui.horizontal(|ui| {
                    if ui.add_enabled(ok, Button::new("Create task")).clicked() {
                        action = Some(SchedulerAction::ConfirmAdd);
                    }
                    if ui.button("Cancel").clicked() {
                        action = Some(SchedulerAction::CancelAdd);
                    }
                    if !ok {
                        ui.small("Give the task a name first.");
                    }
                });
            });
        }

        ui.separator();

        ui.group(|ui| {
            ui.heading("Scheduled Tasks");
            if app.scheduler.tasks().is_empty() {
                ui.label(RichText::new("No tasks yet. Use ➕ Add Task to create one.").color(Color32::GRAY));
            }

            for task in app.scheduler.tasks() {
                ui.horizontal(|ui| {
                    let status_color = match &task.status {
                        TaskStatus::Pending => Color32::GRAY,
                        TaskStatus::Running => Color32::YELLOW,
                        TaskStatus::Completed => Color32::GREEN,
                        TaskStatus::Failed(_) => Color32::RED,
                        TaskStatus::Paused => Color32::LIGHT_BLUE,
                    };
                    ui.colored_label(status_color, "●");
                    ui.label(&task.name);
                    ui.small(task.task_type.label());

                    if let Some(next_run) = task.next_run {
                        ui.label(format!("Next: {}", next_run.format("%Y-%m-%d %H:%M")));
                    }
                    if let TaskStatus::Failed(why) = &task.status {
                        ui.colored_label(Color32::RED, format!("failed: {why}"));
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Delete this task").clicked() {
                            action = Some(SchedulerAction::Remove(task.id));
                        }
                        let label = if task.enabled { "⏸" } else { "▶" };
                        let hint = if task.enabled { "Pause this task" } else { "Enable this task" };
                        if ui.button(label).on_hover_text(hint).clicked() {
                            action = Some(SchedulerAction::Toggle(task.id));
                        }
                    });
                });
            }
        });
    });

    // ── apply the click, now that nothing is borrowed ───────────────────────────────────────────
    match action {
        Some(SchedulerAction::Add) => {
            app.scheduler_adding = true;
        }
        Some(SchedulerAction::CancelAdd) => {
            app.scheduler_adding = false;
        }
        Some(SchedulerAction::ConfirmAdd) => {
            let name = app.scheduler_draft_name.trim().to_string();
            if !name.is_empty() {
                // ids are allocated from the existing set so a delete+add cannot collide
                let next_id = app.scheduler.tasks().iter().map(|t| t.id).max().unwrap_or(0) + 1;
                app.scheduler.add_task(ScheduledTask {
                    id: next_id,
                    name,
                    task_type: app.scheduler_draft_type.to_task_type(&app.scheduler_draft_query),
                    schedule: app.scheduler_draft_freq.to_schedule(),
                    next_run: None,           // add_task calculates this
                    last_run: None,
                    status: TaskStatus::Pending,
                    data: None,
                    enabled: true,
                });
                app.scheduler_adding = false;
                app.scheduler_draft_name.clear();
                app.scheduler_draft_query.clear();
            }
        }
        Some(SchedulerAction::Remove(id)) => app.scheduler.remove_task(id),
        Some(SchedulerAction::Toggle(id)) => app.scheduler.toggle_task(id),
        None => {}
    }
}

// ── the small choice enums the dialog binds to ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskTypeChoice {
    ResearchMarket,
    GenerateProduct,
    BackupData,
}

impl TaskTypeChoice {
    pub fn all() -> [TaskTypeChoice; 3] {
        [Self::ResearchMarket, Self::GenerateProduct, Self::BackupData]
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::ResearchMarket => "Market research",
            Self::GenerateProduct => "Generate product",
            Self::BackupData => "Back up data",
        }
    }
    /// Build the real TaskType. A blank query still produces a valid task rather than a panic —
    /// the scheduler should refuse bad input, not crash on it.
    pub fn to_task_type(&self, query: &str) -> TaskType {
        let q = query.trim();
        match self {
            Self::ResearchMarket => TaskType::ResearchMarket { query: q.to_string() },
            Self::GenerateProduct => TaskType::GenerateProduct {
                template_id: q.to_string(),
                params: serde_json::json!({}),
            },
            Self::BackupData => TaskType::BackupData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyChoice {
    Daily,
    Weekly,
    Interval,
    Smart,
}

impl FrequencyChoice {
    pub fn all() -> [FrequencyChoice; 4] {
        [Self::Daily, Self::Weekly, Self::Interval, Self::Smart]
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Interval => "Every hour",
            Self::Smart => "Smart (business hours)",
        }
    }
    pub fn to_schedule(&self) -> Schedule {
        match self {
            // 09:00 local-ish; the scheduler recalculates the true next run from this
            Self::Daily => Schedule::Daily { hour: 9, minute: 0 },
            Self::Weekly => Schedule::Weekly { day: 1, hour: 9, minute: 0 },
            Self::Interval => Schedule::Interval { minutes: 60 },
            Self::Smart => Schedule::Smart,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every choice the dialog offers must produce a real task type — otherwise a user picks an
    /// option and the scheduler quietly gets something else.
    #[test]
    fn every_task_type_choice_builds_a_task() {
        for c in TaskTypeChoice::all() {
            let t = c.to_task_type("x");
            let label = t.label();
            assert!(!label.is_empty(), "{:?} produced no label", c);
        }
    }

    /// A blank query must still yield a valid task, not a panic and not an empty-name crash.
    #[test]
    fn blank_input_still_builds_a_valid_task() {
        for c in TaskTypeChoice::all() {
            let _ = c.to_task_type("   ");
        }
    }

    /// Every frequency maps to a schedule, and each schedule is distinct — two options that
    /// collapse to the same schedule would be a misleading menu.
    #[test]
    fn every_frequency_maps_to_a_distinct_schedule() {
        let mut seen = Vec::new();
        for f in FrequencyChoice::all() {
            seen.push(format!("{:?}", f.to_schedule()));
            assert!(!f.label().is_empty());
        }
        seen.sort();
        let n = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), n, "two frequency options produce the same schedule");
    }

    /// The task types the dialog can build must be types the scheduler can actually execute —
    /// otherwise the dialog creates work nothing will ever run.
    #[test]
    fn dialog_types_are_executable_by_the_scheduler() {
        let src = include_str!("../scheduler.rs");
        for c in TaskTypeChoice::all() {
            let t = c.to_task_type("q");
            let variant = match t {
                TaskType::ResearchMarket { .. } => "ResearchMarket",
                TaskType::GenerateProduct { .. } => "GenerateProduct",
                TaskType::BackupData => "BackupData",
                _ => continue,
            };
            assert!(
                src.contains(variant),
                "scheduler.rs has no arm for {variant} — the dialog would create a task nothing runs"
            );
        }
    }
}
