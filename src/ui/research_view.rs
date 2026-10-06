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
                // ── Market insight (wired: analyze_market had 0 call sites) ────────────────────
                // The engine that turns raw results into a buying decision — average price, the
                // price band, the words that recur, how crowded it is, and a 0-100 opportunity
                // score — was fully implemented and never shown. The user got listings and had to
                // do the arithmetic themselves.
                {
                    let insight = app.research.analyze_market(&results);
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong("Market insight");
                            let colour = if insight.opportunity_score >= 70 {
                                Color32::from_rgb(120, 210, 140)
                            } else if insight.opportunity_score >= 40 {
                                Color32::YELLOW
                            } else {
                                Color32::from_rgb(240, 120, 120)
                            };
                            ui.colored_label(colour, format!("opportunity {}/100", insight.opportunity_score));
                        });
                        ui.label(format!(
                            "Average price ${:.2} · range ${:.2}–${:.2} · competition: {}",
                            insight.avg_price,
                            insight.price_range.0,
                            insight.price_range.1,
                            insight.competition_level.name(),
                        ));
                        if !insight.top_keywords.is_empty() {
                            let words: Vec<String> = insight
                                .top_keywords
                                .iter()
                                .take(10)
                                .map(|(w, n)| format!("{w} ({n})"))
                                .collect();
                            ui.label(format!("Recurring words: {}", words.join(", ")));
                        }
                    });
                    ui.add_space(6.0);
                }

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

        ui.add_space(10.0);
        ui.separator();
        strategy_panel(app, ui);
    });
}

/// The Strategy panel — the deep-thinking layer.
///
/// Deliberately ONE button and ONE call. A strategy chat would burn the user's own API credit in
/// a loop; a single brief does not. The user picks the provider and model, or leaves them on Auto.
fn strategy_panel(app: &mut DpfApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Strategy");
        crate::inline_help::help_button(ui, "strategy", &mut app.active_help_topic);
    });
    ui.label(
        RichText::new(
            "Decide what to build, not just generate it. One call, on your own AI key.",
        )
        .weak(),
    );
    ui.add_space(6.0);

    // Refresh which providers the user actually holds keys for, so the picker never lies.
    let available = {
        let router = crate::llm_router::LLMRouter::new(
            app.config.openai_key.clone(),
            app.config.anthropic_key.clone(),
            app.config.google_key.clone(),
            app.config.deepseek_key.clone(),
            app.config.moonshot_key.clone(),
        );
        router.available_providers()
    };
    app.strategy_state.available = available.clone();

    if available.is_empty() {
        ui.colored_label(
            Color32::from_rgb(220, 160, 60),
            "No AI provider key configured yet — add one in ⚙ Settings to use Strategy.",
        );
        return;
    }

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Niche / market:");
            ui.add(
                TextEdit::singleline(&mut app.strategy_state.request.niche)
                    .hint_text("e.g. ADHD planners for working parents")
                    .desired_width(330.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("You already sell:");
            ui.add(
                TextEdit::singleline(&mut app.strategy_state.request.existing_products)
                    .hint_text("optional — sharpens the advice a lot")
                    .desired_width(330.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Price positioning:");
            ui.add(
                TextEdit::singleline(&mut app.strategy_state.request.price_positioning)
                    .hint_text("optional — e.g. under $50 impulse buy")
                    .desired_width(330.0),
            );
        });

        ui.add_space(6.0);
        ui.separator();

        // ── Provider + model selection ────────────────────────────────────────────────
        // The app is NEVER locked to one model: providers retire model ids, so the model field
        // accepts any id typed by hand, with a suggestion list only as a convenience.
        let current = crate::llm_router::Provider::from_config_str(&app.strategy_state.provider_choice);
        let current_label = current
            .map(|p| p.label().to_string())
            .unwrap_or_else(|| "Auto (best available)".to_string());

        ui.horizontal(|ui| {
            ui.label("Use:");
            ComboBox::from_id_source("strategy_provider")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(current.is_none(), "Auto (best available)")
                        .clicked()
                    {
                        app.strategy_state.provider_choice.clear();
                        app.config.strategy_provider.clear();
                        app.strategy_state.model_choice.clear();
                        app.config.strategy_model.clear();
                    }
                    for p in &available {
                        if ui
                            .selectable_label(current == Some(*p), p.label())
                            .clicked()
                        {
                            let id = p.config_id().to_string();
                            app.strategy_state.provider_choice = id.clone();
                            app.config.strategy_provider = id;
                            // Reset the model so the new provider's default applies.
                            app.strategy_state.model_choice.clear();
                            app.config.strategy_model.clear();
                        }
                    }
                });

            // Which provider will actually run, told plainly before any money is spent.
            let resolved = crate::strategy::resolve_provider(
                &crate::llm_router::LLMRouter::new(
                    app.config.openai_key.clone(),
                    app.config.anthropic_key.clone(),
                    app.config.google_key.clone(),
                    app.config.deepseek_key.clone(),
                    app.config.moonshot_key.clone(),
                ),
                current,
            );
            if let Some(p) = resolved {
                let model = crate::strategy::resolve_model(p, Some(&app.strategy_state.model_choice));
                ui.label(RichText::new(format!("→ {} · {}", p.label(), model)).weak());
            }
        });

        ui.horizontal(|ui| {
            ui.label("Model:");
            let prov = crate::strategy::resolve_provider(
                &crate::llm_router::LLMRouter::new(
                    app.config.openai_key.clone(),
                    app.config.anthropic_key.clone(),
                    app.config.google_key.clone(),
                    app.config.deepseek_key.clone(),
                    app.config.moonshot_key.clone(),
                ),
                current,
            );
            if let Some(p) = prov {
                ComboBox::from_id_source("strategy_model")
                    .selected_text(if app.strategy_state.model_choice.is_empty() {
                        format!("default ({})", crate::llm_router::LLMProfile::strategic_model(p))
                    } else {
                        app.strategy_state.model_choice.clone()
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                app.strategy_state.model_choice.is_empty(),
                                format!("default ({})", crate::llm_router::LLMProfile::strategic_model(p)),
                            )
                            .clicked()
                        {
                            app.strategy_state.model_choice.clear();
                            app.config.strategy_model.clear();
                        }
                        for m in p.suggested_models() {
                            if ui
                                .selectable_label(app.strategy_state.model_choice == *m, *m)
                                .clicked()
                            {
                                app.strategy_state.model_choice = m.to_string();
                                app.config.strategy_model = m.to_string();
                            }
                        }
                    });
            }
            // Free text: a retired or brand-new model id is never a blocker.
            ui.add(
                TextEdit::singleline(&mut app.strategy_state.model_choice)
                    .hint_text("or type any model id")
                    .desired_width(200.0),
            )
            .changed()
            .then(|| {
                app.config.strategy_model = app.strategy_state.model_choice.clone();
            });
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let can_run = !app.strategy_state.request.niche.trim().is_empty();
            if ui
                .add_enabled(can_run, Button::new("🧠 Build strategy brief"))
                .clicked()
            {
                run_strategy(app);
            }
            ui.label(
                RichText::new("Uses one call on your own API key.").weak(),
            );
        });
    });

    if let Some(err) = &app.strategy_state.last_error {
        ui.add_space(6.0);
        ui.colored_label(Color32::from_rgb(220, 90, 90), format!("⚠ {err}"));
    }

    if let Some(brief) = &app.strategy_state.last_brief {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "Brief from {} · {}",
                brief.provider_label, brief.model
            ))
            .weak(),
        );
        ui.separator();
        ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            ui.label(markdown_ish(&brief.content));
        });
    }
}

/// Run the brief — ONE call, and record the outcome honestly either way.
fn run_strategy(app: &mut DpfApp) {
    app.strategy_state.last_error = None;
    app.strategy_state.last_brief = None;

    let router = crate::llm_router::LLMRouter::new(
        app.config.openai_key.clone(),
        app.config.anthropic_key.clone(),
        app.config.google_key.clone(),
        app.config.deepseek_key.clone(),
        app.config.moonshot_key.clone(),
    );
    let preferred = crate::llm_router::Provider::from_config_str(&app.config.strategy_provider);
    let model = app.config.strategy_model.clone();
    let req = app.strategy_state.request.clone();

    match crate::strategy::run(
        &router,
        &app.runtime,
        preferred,
        if model.trim().is_empty() { None } else { Some(&model) },
        &req,
    ) {
        Ok(brief) => app.strategy_state.last_brief = Some(brief),
        Err(e) => app.strategy_state.last_error = Some(e),
    }
}

/// egui's default Label has no markdown renderer; strip the `##` markers so headings read
/// cleanly instead of showing literal hashes at the user.
fn markdown_ish(s: &str) -> String {
    s.lines()
        .map(|l| {
            let t = l.trim_start();
            if let Some(rest) = t.strip_prefix("### ") {
                format!("▸ {}", rest)
            } else if let Some(rest) = t.strip_prefix("## ") {
                format!("\n▸ {}", rest)
            } else if let Some(rest) = t.strip_prefix("# ") {
                format!("▸ {}", rest)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
