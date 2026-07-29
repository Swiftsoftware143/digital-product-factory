//! Scheduler view

use egui::*;
use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Scheduler");
        ui.separator();

        // Add task button
        ui.horizontal(|ui| {
            if ui.button("➕ Add Task").clicked() {
                // Open add task dialog
            }
            if ui.button("▶ Start Scheduler").clicked() {
                app.scheduler.start();
            }
            if ui.button("⏸ Stop Scheduler").clicked() {
                app.scheduler.stop();
            }
        });

        ui.separator();

        // Task list
        ui.group(|ui| {
            ui.heading("Scheduled Tasks");

            for task in app.scheduler.tasks() {
                ui.horizontal(|ui| {
                    // Status indicator
                    let status_color = match task.status {
                        crate::scheduler::TaskStatus::Pending => Color32::GRAY,
                        crate::scheduler::TaskStatus::Running => Color32::YELLOW,
                        crate::scheduler::TaskStatus::Completed => Color32::GREEN,
                        crate::scheduler::TaskStatus::Failed(_) => Color32::RED,
                        crate::scheduler::TaskStatus::Paused => Color32::LIGHT_BLUE,
                    };
                    ui.colored_label(status_color, "●");

                    ui.label(&task.name);

                    if let Some(next_run) = task.next_run {
                        ui.label(format!("Next: {}", next_run.format("%Y-%m-%d %H:%M")));
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            // Delete task
                        }
                        if ui.button(if task.enabled { "⏸" } else { "▶" }).clicked() {
                            // Toggle task
                        }
                    });
                });
            }
        });
    });
}
