//! Market Research tab.
//!
//! This tab was non-functional end to end while looking complete:
//!   * the `🔍 Search` button body was literally `// Trigger search` and did nothing;
//!   * the platform checkboxes were `let mut` bindings INSIDE the frame, so they were recreated
//!     every repaint and never read by anything;
//!   * the results area was a literal `// TODO: Display actual research results`.
//!
//! Meanwhile the real network backends existed and were never called, and their HTML parsers
//! returned an empty list regardless of input — so even a wired-up search would have reported
//! success and always found zero products.
//!
//! Now: the button runs the real search, the selection persists, the parsers read JSON-LD, and
//! when a marketplace genuinely cannot be read the user is told why instead of seeing a blank
//! panel.

use egui::*;

use crate::app::DpfApp;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Market Research");
            crate::inline_help::help_button(ui, "research", &mut app.active_help_topic);
        });
        ui.separator();

        let mut run_search = false;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(
                    TextEdit::singleline(&mut app.research.search_query)
                        .hint_text("Enter a product type or keyword…")
                        .desired_width(320.0),
                );
                if ui.button("🔍 Search").clicked() {
                    run_search = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Platforms:");
                ui.checkbox(&mut app.research.use_etsy, "Etsy");
                ui.checkbox(&mut app.research.use_gumroad, "Gumroad");
                ui.label(
                    RichText::new("(Amazon is not implemented)")
                        .size(11.0)
                        .color(Color32::GRAY),
                );
            });
        });

        ui.separator();

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

        // Results
        let results = app.research.search_results.clone();
        ui.group(|ui| {
            ui.heading("Results");

            if let Some((is_err, note)) = &app.research.last_search_note {
                let colour = if *is_err {
                    Color32::from_rgb(240, 120, 120)
                } else {
                    Color32::from_rgb(120, 210, 140)
                };
                ui.colored_label(colour, note);
            }

            if results.is_empty() {
                ui.label(RichText::new("No results yet — run a search above.").weak());
            } else {
                for r in &results {
                    ui.separator();
                    ui.label(
                        RichText::new(format!("{} · \"{}\"", r.platform, r.query)).strong(),
                    );

                    if r.products.is_empty() {
                        ui.label(
                            RichText::new(format!(
                                "{} returned no readable listings for this query.",
                                r.platform
                            ))
                            .weak(),
                        );
                        continue;
                    }

                    for p in &r.products {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&p.title).strong());
                            if let Some(price) = p.price {
                                ui.label(format!("${:.2}", price));
                            }
                            if let Some(rating) = p.rating {
                                ui.label(format!("★ {:.1}", rating));
                            }
                            if let Some(reviews) = p.reviews {
                                ui.label(format!("({} reviews)", reviews));
                            }
                        });
                        if let Some(seller) = &p.seller {
                            ui.label(RichText::new(format!("by {}", seller)).weak());
                        }
                    }
                }
            }
        });

        if run_search {
            run_research(app);
        }
    });
}

/// Run the selected searches and record an honest note about what actually came back.
fn run_research(app: &mut DpfApp) {
    let query = app.research.search_query.trim().to_string();

    if query.is_empty() {
        app.research.last_search_note =
            Some((true, "Type something to search for first.".to_string()));
        return;
    }
    if !app.research.use_etsy && !app.research.use_gumroad {
        app.research.last_search_note = Some((true, "Tick at least one platform.".to_string()));
        return;
    }

    let mut collected = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    let mut empty_platforms: Vec<&str> = Vec::new();

    if app.research.use_etsy {
        match app.research.search_etsy(&query) {
            Ok(r) => {
                if r.products.is_empty() {
                    empty_platforms.push("Etsy");
                }
                collected.push(r);
            }
            Err(e) => problems.push(format!("Etsy: {e}")),
        }
    }

    if app.research.use_gumroad {
        match app.research.search_gumroad(&query) {
            Ok(r) => {
                if r.products.is_empty() {
                    empty_platforms.push("Gumroad");
                }
                collected.push(r);
            }
            Err(e) => problems.push(format!("Gumroad: {e}")),
        }
    }

    let found: usize = collected.iter().map(|r| r.products.len()).sum();
    app.research.search_results = collected;

    app.research.last_search_note = Some(if !problems.is_empty() {
        (true, format!("Search problem — {}", problems.join(" · ")))
    } else if found == 0 {
        (
            false,
            format!(
                "{} returned no readable product data for \"{}\". These marketplaces render their \
                 listings in JavaScript and publish no machine-readable product data we can read, \
                 so this is a limitation of the source rather than an error on your side.",
                empty_platforms.join(" and "),
                query
            ),
        )
    } else {
        (
            false,
            format!("Found {} listing(s) for \"{}\".", found, query),
        )
    });
}
