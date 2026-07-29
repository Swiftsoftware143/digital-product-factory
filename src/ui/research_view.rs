//! Market research view

use egui::*;
use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Market Research");
        ui.separator();

        // Search input
        ui.group(|ui| {
            let mut search_etsy = true;
            let mut search_gumroad = true;
            let mut search_amazon = false;

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(egui::TextEdit::singleline(&mut app.research.search_query).hint_text("Enter product type or keyword..."));

                if ui.button("🔍 Search").clicked() {
                    // Trigger search
                }
            });

            ui.horizontal(|ui| {
                ui.label("Platforms:");
                ui.checkbox(&mut search_etsy, "Etsy");
                ui.checkbox(&mut search_gumroad, "Gumroad");
                ui.checkbox(&mut search_amazon, "Amazon");
            });
        });

        ui.separator();

        // Trending searches
        ui.group(|ui| {
            ui.heading("Trending Searches");
            ui.horizontal_wrapped(|ui| {
                for trend in app.research.trending_searches() {
                    if ui.button(&trend).clicked() {
                        app.research.search_query = trend;
                    }
                }
            });
        });

        ui.separator();

        // Results area
        ui.group(|ui| {
            ui.heading("Results");
            ui.label("Search results will appear here");
            // TODO: Display actual research results
        });
    });
}
