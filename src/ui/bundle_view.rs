//! Bundle builder view

use egui::*;
use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Bundle Builder");
        ui.separator();

        // Auto-bundle strategies
        ui.group(|ui| {
            ui.heading("Auto-Bundle Strategies");
            ui.horizontal(|ui| {
                if ui.button("By Category").clicked() {
                    // Auto-bundle by category
                }
                if ui.button("By Value").clicked() {
                    // Auto-bundle by value tiers
                }
                if ui.button("Seasonal").clicked() {
                    // Create seasonal bundle
                }
            });
        });

        ui.separator();

        // Manual bundle builder
        ui.group(|ui| {
            ui.heading("Manual Bundle");
            ui.label("Select products to bundle:");
            // TODO: Product selection list
        });

        ui.separator();

        // Bundle stats
        ui.group(|ui| {
            ui.heading("Bundle Statistics");
            ui.label("Bundle stats will appear here");
        });
    });
}
