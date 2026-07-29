//! Dashboard view with quick stats and recent activity

use egui::*;
use crate::app::{DpfApp, Tab};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Dashboard");
        ui.separator();

        // Quick stats row
        ui.horizontal(|ui| {
            let stats = app.pipeline.stats();

            super::components::stat_card(
                ui,
                "Total Ideas",
                &stats.total.to_string(),
                Some("+3 this week")
            );

            super::components::stat_card(
                ui,
                "In Progress",
                &app.pipeline.ideas_by_stage(crate::pipeline::PipelineStage::Creating).len().to_string(),
                None
            );

            super::components::stat_card(
                ui,
                "Selling",
                &app.pipeline.ideas_by_stage(crate::pipeline::PipelineStage::Selling).len().to_string(),
                None
            );

            super::components::stat_card(
                ui,
                "Potential Value",
                &format!("${:.0}", stats.potential_value),
                None
            );
        });

        ui.separator();

        // Two column layout
        ui.columns(2, |columns| {
            // Left column - Pipeline overview
            columns[0].group(|ui| {
                ui.heading("Pipeline Overview");
                ui.separator();

                for (stage, count) in app.pipeline.stats().by_stage {
                    ui.horizontal(|ui| {
                        ui.colored_label(stage.color(), stage.name());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("{}", count));
                        });
                    });
                    ui.add_space(4.0);
                }
            });

            // Right column - Quick actions
            columns[1].group(|ui| {
                ui.heading("Quick Actions");
                ui.separator();

                if ui.button("➕ New Product Idea").clicked() {
                    app.pipeline.show_new_idea_dialog = true;
                }

                if ui.button("🔍 Research Market").clicked() {
                    app.current_tab = Tab::Research;
                }

                if ui.button("⚡ Generate Product").clicked() {
                    app.current_tab = Tab::Create;
                }

                if ui.button("📦 Create Bundle").clicked() {
                    app.current_tab = Tab::Bundles;
                }
            });
        });

        ui.separator();

        // Recent activity
        ui.group(|ui| {
            ui.heading("Recent Activity");
            ui.separator();

            // Show last 5 updated ideas
            let mut recent: Vec<_> = app.pipeline.ideas.iter().collect();
            recent.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            for idea in recent.iter().take(5) {
                ui.horizontal(|ui| {
                    ui.colored_label(idea.stage.color(), idea.stage.name());
                    ui.label(&idea.title);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(idea.updated_at.format("%H:%M").to_string());
                    });
                });
            }
        });
    });
}
