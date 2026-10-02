//! Sidebar — Navigation
//!
//! Every tab is **gated on the active licence tier**. A tab the current tier does not include is
//! shown dimmed with a lock and cannot be opened; clicking it opens the licence dialog and explains
//! which tier unlocks it. Feature slugs come from `feature_tiers.json`, so the pricing table and the
//! gating can never drift apart.

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::license_manager::LicenseManager;

/// Which feature slug a tab requires. Anything the free Personal tier includes keeps the app
/// usable with no licence at all.
fn feature_for(tab: Tab) -> &'static str {
    match tab {
        // Free (Personal) — the core loop
        Tab::Dashboard | Tab::Pipeline | Tab::Settings => "pipeline",
        Tab::Create => "ai_generation",
        Tab::Templates => "templates",
        Tab::Research => "market_research",
        Tab::Contract => "contract_generator",
        Tab::Presets => "presets",
        Tab::Variants => "variants",
        // Team
        Tab::Mockup => "mockup_compositor",
        Tab::Bundles => "bundles",
        Tab::Scheduler => "scheduler",
        Tab::Analytics => "analytics",
        Tab::Publish => "publishing",
        Tab::QC => "qc",
        Tab::AssetLibrary => "assets",
        Tab::Webhooks => "webhooks",
        Tab::Adverts => "adverts",
        Tab::LogoGenerator => "logo_generator",
        Tab::VectorGenerator => "vector_generator",
        // Agency / Enterprise
        Tab::Clients => "client_management",
        Tab::Compliance => "compliance",
        Tab::Admin => "admin_panel",
    }
}

/// Draw one nav entry, gated on the licence.
fn tab_item(app: &mut DpfApp, ui: &mut Ui, tab: Tab, label: &str) {
    let feature = feature_for(tab);
    let allowed = app.license_manager.has_feature(feature);
    let selected = app.current_tab == tab;

    if allowed {
        if ui.selectable_label(selected, label).clicked() {
            app.current_tab = tab;
            if tab == Tab::Admin {
                app.admin.admin_mode = true;
            }
        }
        return;
    }

    let required = LicenseManager::required_tier_for(feature);
    let hint = match required {
        Some(t) => format!(
            "{} needs the {} plan ({}). Click to enter a licence key.",
            label,
            t.display_name(),
            t.price_label()
        ),
        None => format!("{} is not included in this licence.", label),
    };

    let resp = ui
        .add_enabled(false, SelectableLabel::new(false, format!("🔒 {}", label)))
        .on_hover_text(hint.clone());
    // A disabled widget swallows input, so overlay our own click sense on the same rect.
    let click = ui.interact(resp.rect, ui.id().with(("locked", label)), Sense::click());
    if click.clicked() {
        app.license_message = Some((false, hint));
        app.show_license_dialog = true;
    }
}

pub fn show(app: &mut DpfApp, ctx: &Context) {
    SidePanel::left("sidebar")
        .resizable(false)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("DPF");
                ui.label(
                    RichText::new(app.license_manager.tier_name())
                        .size(11.0)
                        .color(Color32::GRAY),
                );
            });

            ui.separator();
            ui.label("Main");
            tab_item(app, ui, Tab::Dashboard, "Dashboard");
            tab_item(app, ui, Tab::Pipeline, "Pipeline");
            tab_item(app, ui, Tab::Create, "Create");
            tab_item(app, ui, Tab::Research, "Research");
            tab_item(app, ui, Tab::Templates, "Templates");

            ui.separator();
            ui.label("Tools");
            tab_item(app, ui, Tab::Bundles, "Bundles");
            tab_item(app, ui, Tab::Scheduler, "Scheduler");
            tab_item(app, ui, Tab::Mockup, "Mockups");
            tab_item(app, ui, Tab::Presets, "Presets");
            tab_item(app, ui, Tab::Contract, "Contracts");

            ui.separator();
            ui.label("Business");
            tab_item(app, ui, Tab::Analytics, "Analytics");
            tab_item(app, ui, Tab::Publish, "Publish");

            ui.separator();
            ui.label("Quality");
            tab_item(app, ui, Tab::QC, "QC Checklist");
            tab_item(app, ui, Tab::Compliance, "Compliance");
            tab_item(app, ui, Tab::Clients, "👥 Clients");

            ui.separator();
            ui.label("Product Data");
            tab_item(app, ui, Tab::Variants, "Variants");

            ui.separator();
            ui.label("Library");
            tab_item(app, ui, Tab::Adverts, "📢 Adverts");
            tab_item(app, ui, Tab::LogoGenerator, "🎨 Logo Generator");
            tab_item(app, ui, Tab::VectorGenerator, "📐 Vector Generator");
            tab_item(app, ui, Tab::AssetLibrary, "Asset Library");

            ui.separator();
            tab_item(app, ui, Tab::Webhooks, "Webhooks");

            ui.separator();
            tab_item(app, ui, Tab::Admin, "Admin");

            ui.separator();
            if ui.button("🔑 Licence").clicked() {
                app.show_license_dialog = true;
            }
            if ui.button("⚙ Settings").clicked() {
                app.show_settings = true;
            }
        });
}
