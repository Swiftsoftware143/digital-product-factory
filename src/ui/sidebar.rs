//! Sidebar — Navigation

use egui::*;
use crate::app::{DpfApp, Tab};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    SidePanel::left("sidebar")
        .resizable(false)
        .default_width(185.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("DPF");
            });

            ui.separator();

            // Main tabs
            ui.label("Main");
            if ui.selectable_label(app.current_tab == Tab::Dashboard, "Dashboard").clicked() {
                app.current_tab = Tab::Dashboard;
            }
            if ui.selectable_label(app.current_tab == Tab::Pipeline, "Pipeline").clicked() {
                app.current_tab = Tab::Pipeline;
            }
            if ui.selectable_label(app.current_tab == Tab::Create, "Create").clicked() {
                app.current_tab = Tab::Create;
            }
            if ui.selectable_label(app.current_tab == Tab::Research, "Research").clicked() {
                app.current_tab = Tab::Research;
            }
            if ui.selectable_label(app.current_tab == Tab::Templates, "Templates").clicked() {
                app.current_tab = Tab::Templates;
            }

            ui.separator();
            ui.label("Tools");
            if ui.selectable_label(app.current_tab == Tab::Bundles, "Bundles").clicked() {
                app.current_tab = Tab::Bundles;
            }
            if ui.selectable_label(app.current_tab == Tab::Scheduler, "Scheduler").clicked() {
                app.current_tab = Tab::Scheduler;
            }
            if ui.selectable_label(app.current_tab == Tab::Mockup, "Mockups").clicked() {
                app.current_tab = Tab::Mockup;
            }
            if ui.selectable_label(app.current_tab == Tab::Presets, "Presets").clicked() {
                app.current_tab = Tab::Presets;
            }
            if ui.selectable_label(app.current_tab == Tab::Contract, "Contracts").clicked() {
                app.current_tab = Tab::Contract;
            }

            ui.separator();
            ui.label("Business");
            if ui.selectable_label(app.current_tab == Tab::Analytics, "Analytics").clicked() {
                app.current_tab = Tab::Analytics;
            }
            if ui.selectable_label(app.current_tab == Tab::Publish, "Publish").clicked() {
                app.current_tab = Tab::Publish;
            }

            // Quality section
            ui.separator();
            ui.label("Quality");
            if ui.selectable_label(app.current_tab == Tab::QC, "QC Checklist").clicked() {
                app.current_tab = Tab::QC;
            }
            if ui.selectable_label(app.current_tab == Tab::Compliance, "Compliance").clicked() {
                app.current_tab = Tab::Compliance;
            }

            // Product Data section
            ui.separator();
            ui.label("Product Data");
            if ui.selectable_label(app.current_tab == Tab::Variants, "Variants").clicked() {
                app.current_tab = Tab::Variants;
            }

            // Library section
            ui.separator();
            ui.label("Library");
            if ui.selectable_label(app.current_tab == Tab::Adverts, "📢 Adverts").clicked() { app.current_tab = Tab::Adverts; }
            if ui.selectable_label(app.current_tab == Tab::LogoGenerator, "🎨 Logo Generator").clicked() { app.current_tab = Tab::LogoGenerator; }
            if ui.selectable_label(app.current_tab == Tab::VectorGenerator, "📐 Vector Generator").clicked() { app.current_tab = Tab::VectorGenerator; }
            if ui.selectable_label(app.current_tab == Tab::AssetLibrary, "Asset Library").clicked() {
                app.current_tab = Tab::AssetLibrary;
            }

            // Automation
            ui.separator();
            if ui.selectable_label(app.current_tab == Tab::Webhooks, "Webhooks").clicked() {
                app.current_tab = Tab::Webhooks;
            }

            ui.separator();
            if ui.selectable_label(app.current_tab == Tab::Admin, "Admin").clicked() {
                app.current_tab = Tab::Admin;
                app.admin.admin_mode = true;
            }
        });
}
