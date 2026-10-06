//! Sidebar — Navigation
//!
//! Every tab is **gated on the active licence tier**. A tab the current tier does not include is
//! shown dimmed with a lock and cannot be opened; clicking it opens the licence dialog and explains
//! which tier unlocks it. Feature slugs come from `feature_tiers.json`, so the pricing table and the
//! gating can never drift apart.

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::theme::{self, TEAL, TEXT_DIM, AMBER};
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

/// A small-caps section label. Deliberately quiet: it separates groups without competing with the
/// tab names, which is the whole point of a sidebar.
fn section(ui: &mut Ui, label: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(label.to_uppercase())
                .size(10.0)
                .strong()
                .color(TEXT_DIM.gamma_multiply(0.85)),
        );
    });
    ui.add_space(2.0);
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
        // No price in the hint: the software says which plan unlocks the tab, and the sales page
        // says what it costs.
        Some(t) => format!(
            "{} needs the {} plan. Click to enter a licence key.",
            label,
            t.display_name()
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
    // An explicit frame, not the inherited panel fill: the sidebar must sit visually ABOVE the
    // content area or the whole window reads as one flat field with no depth.
    let frame = Frame::none()
        .fill(theme::SURFACE)
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .stroke(Stroke::new(1.0, theme::BLUE.gamma_multiply(0.35)));

    SidePanel::left("sidebar")
        .resizable(false)
        .default_width(210.0)
        .frame(frame)
        .show(ctx, |ui| {
            // Brand header. The rule under the wordmark is the brand teal, so the first thing on
            // screen is unmistakably this product rather than a default framework panel.
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("DIGITAL PRODUCT").size(9.0).color(TEXT_DIM));
                ui.label(
                    RichText::new("FACTORY")
                        .size(19.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(2.0);
                // The plan badge: which plan you hold, never what it cost.
                let licensed = app.license_manager.is_licensed();
                ui.label(
                    RichText::new(app.license_manager.tier_name())
                        .size(10.5)
                        .strong()
                        .color(if licensed { TEAL } else { TEXT_DIM }),
                );
            });
            ui.add_space(6.0);
            // A full-width teal rule, drawn rather than a separator so it carries the brand colour.
            let (rect, _) = ui.allocate_exact_size(
                vec2(ui.available_width(), 2.0),
                Sense::hover(),
            );
            ui.painter()
                .rect_filled(rect, Rounding::same(1.0), TEAL.gamma_multiply(0.9));
            ui.add_space(2.0);

            section(ui, "Start here");
            tab_item(app, ui, Tab::Dashboard, "Dashboard");
            tab_item(app, ui, Tab::Pipeline, "Pipeline");
            tab_item(app, ui, Tab::Create, "Create");
            tab_item(app, ui, Tab::Research, "Research");
            tab_item(app, ui, Tab::Templates, "Templates");

            section(ui, "Create & plan");
            tab_item(app, ui, Tab::Bundles, "Bundles");
            tab_item(app, ui, Tab::Scheduler, "Scheduler");
            tab_item(app, ui, Tab::Mockup, "Mockups");
            tab_item(app, ui, Tab::Presets, "Presets");
            tab_item(app, ui, Tab::Contract, "Contracts");

            section(ui, "Sell & measure");
            tab_item(app, ui, Tab::Analytics, "Analytics");
            tab_item(app, ui, Tab::Publish, "Publish");

            section(ui, "Run the business");
            tab_item(app, ui, Tab::QC, "QC Checklist");
            tab_item(app, ui, Tab::Compliance, "Compliance");
            tab_item(app, ui, Tab::Clients, "👥 Clients");

            section(ui, "Product data");
            tab_item(app, ui, Tab::Variants, "Variants");

            section(ui, "Assets & marketing");
            tab_item(app, ui, Tab::Adverts, "📢 Adverts");
            tab_item(app, ui, Tab::LogoGenerator, "🎨 Logo Generator");
            tab_item(app, ui, Tab::VectorGenerator, "📐 Vector Generator");
            tab_item(app, ui, Tab::AssetLibrary, "Asset Library");

            ui.separator();
            tab_item(app, ui, Tab::Webhooks, "Webhooks");

            ui.separator();
            // The Admin panel is OWNER-ONLY and is not merely locked for everyone else: it is not
            // drawn at all. A customer should never learn that a key-generation screen exists,
            // because the only thing that knowledge buys them is an attempt to reach it. Reaching it
            // requires an OWNER licence (see `LicenseManager::is_owner`), which is not for sale.
            if app.license_manager.is_owner() {
                tab_item(app, ui, Tab::Admin, "Admin");
                ui.separator();
            }

            if ui.button("🔑 Licence").clicked() {
                app.show_license_dialog = true;
            }
            if ui.button("⚙ Settings").clicked() {
                // Route to the Settings TAB. The dialog flag is kept as well because the existing
                // dialog covers the same ground; the Tab variant was defined and routed but never
                // constructed, so one of the two settings surfaces was dead.
                app.current_tab = Tab::Settings;
                app.show_settings = false;
            }
        });
}
