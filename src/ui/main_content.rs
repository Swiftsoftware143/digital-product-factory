//! Main content area — routes to current tab

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::inline_help;
use crate::qc::{QcResult, QcCheck, QcStatus};
use crate::compliance::DenylistScanner;
use super::adverts_view;
use super::{pipeline_view, analytics_view, publish_view, mockup_view, admin_view, variants_view, clients_view};

pub fn show(app: &mut DpfApp, ctx: &Context) {
    // Show modal dialogs
    if app.show_add_sale_dialog {
        analytics_view::show_add_sale_dialog(app, ctx);
    }

    match app.current_tab {
        Tab::Dashboard => show_dashboard(app, ctx),
        Tab::Pipeline => pipeline_view::show(app, ctx),
        Tab::Create => show_create(app, ctx),
        Tab::Research => show_research(app, ctx),
        Tab::Templates => show_templates(app, ctx),
        Tab::Bundles => show_bundles(app, ctx),
        Tab::Scheduler => show_scheduler(app, ctx),
        Tab::Presets => super::presets_view::show(app, ctx),
        Tab::Contract => show_contract(app, ctx),
        Tab::Analytics => analytics_view::show(app, ctx),
        Tab::Publish => publish_view::show(app, ctx),
        Tab::Mockup => mockup_view::show(app, ctx),
        Tab::Settings => show_settings(app, ctx),
        Tab::Admin => admin_view::show(app, ctx),
        Tab::Variants => variants_view::show(app, ctx),
        Tab::Clients => clients_view::show(app, ctx),
        // New tabs from remote
        Tab::QC => show_qc(app, ctx),
        Tab::Compliance => show_compliance(app, ctx),
        Tab::AssetLibrary => show_asset_library(app, ctx),
        Tab::Webhooks => show_webhooks(app, ctx),
        Tab::Adverts => adverts_view::show(app, ctx),
        Tab::LogoGenerator => crate::ui::logo_view::show(app, ctx),
        Tab::VectorGenerator => crate::ui::vector_view::show(app, ctx),
    }

    // Help overlay (persistent across tabs)
    if let Some(topic_id) = &app.active_help_topic.clone() {
        if topic_id == "__index__" {
            inline_help::show_help_index(ctx, &mut app.active_help_topic);
        } else {
            inline_help::show_help_popup(ctx, topic_id, &mut app.active_help_topic);
        }
    }
}

fn show_dashboard(app: &mut DpfApp, ctx: &Context) {
    // First-run guidance. Without a provider key nothing can generate, and a new user would
    // otherwise only discover that when their first Generate fails with a terse error.
    let needs_key = app.has_no_api_key();
    let mut open_settings = false;

    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Dashboard");
            inline_help::help_button(ui, "dashboard", &mut app.active_help_topic);
        });
        ui.separator();

        if needs_key {
            Frame::group(ui.style())
                .fill(Color32::from_rgb(64, 52, 22))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(160, 130, 50)))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⚠ No AI provider key yet")
                                .strong()
                                .color(Color32::from_rgb(250, 210, 120)),
                        );
                        ui.label("— the app needs one before it can generate anything.");
                        if ui.button("⚙ Open Settings").clicked() {
                            open_settings = true;
                        }
                    });
                    ui.label(
                        RichText::new(
                            "Bring your own key: paste a key from OpenAI, Anthropic, Google, \
                             DeepSeek or Moonshot in Settings — only one is needed to start. \
                             See ❓ Help → \"Bring Your Own Key\".",
                        )
                        .size(11.0),
                    );
                });
            ui.add_space(6.0);
        }

        ui.horizontal(|ui| {
            stat_card(ui, "Total Ideas", &app.pipeline.ideas.len().to_string());
            stat_card(ui, "In Progress", &app.pipeline.ideas_by_stage(crate::pipeline::PipelineStage::Creating).len().to_string());
            stat_card(ui, "Selling", &app.pipeline.ideas_by_stage(crate::pipeline::PipelineStage::Selling).len().to_string());

            let total_rev: f64 = app.analytics.records.iter().map(|r| r.net_revenue).sum();
            stat_card(ui, "Total Revenue", &format!("${:.0}", total_rev));
        });

        ui.separator();
        ui.label("Recent Sales");
        let recent: Vec<_> = app.analytics.records.iter().rev().take(5).collect();
        if recent.is_empty() {
            ui.label("  No sales recorded yet.");
        } else {
            for r in &recent {
                ui.label(format!("  {} · {} — ${:.2}", r.sale_date.format("%Y-%m-%d"), r.product_name, r.net_revenue));
            }
        }
    });

    if open_settings {
        app.show_settings = true;
    }
}

fn stat_card(ui: &mut Ui, label: &str, value: &str) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(150.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(value).size(32.0).strong());
            ui.label(label);
        });
    });
}

fn show_create(app: &mut DpfApp, ctx: &Context) {
    super::create_view::show(app, ctx);
}

fn show_research(app: &mut DpfApp, ctx: &Context) {
    super::research_view::show(app, ctx);
}

fn show_templates(app: &mut DpfApp, ctx: &Context) {
    // ── Nested UI-only types/helpers, scoped to this tab so the rest of the
    //    file (owned by other lanes) is untouched. ───────────────────────
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ExportChoice {
        Markdown,
        Html,
        Pdf,
        Docx,
        Xlsx,
        Json,
        Zip,
    }

    impl ExportChoice {
        fn all() -> [ExportChoice; 7] {
            use ExportChoice::*;
            [Markdown, Html, Pdf, Docx, Xlsx, Json, Zip]
        }
        fn label(self) -> &'static str {
            use ExportChoice::*;
            match self {
                Markdown => "Markdown",
                Html => "HTML",
                Pdf => "PDF",
                Docx => "DOCX",
                Xlsx => "XLSX",
                Json => "JSON",
                Zip => "ZIP",
            }
        }
        /// Whether `Exporter` genuinely produces this format.
        ///
        /// All seven do now. This used to exclude PDF, DOCX and XLSX, which wrote a different
        /// file type inside — a printable .html, a .docx.md and a .csv respectively. Those were
        /// implemented for real, so leaving them flagged here would have told users a working
        /// export was still broken.
        fn is_real(self) -> bool {
            true
        }
        /// Caveats worth surfacing. Nothing here is a placeholder any more.
        fn note(self) -> &'static str {
            use ExportChoice::*;
            match self {
                Pdf => "PDF uses the built-in Helvetica font, so characters outside that set are substituted.",
                _ => "",
            }
        }
        fn output_format(self) -> Option<crate::templates::OutputFormat> {
            use crate::templates::OutputFormat;
            use ExportChoice::*;
            match self {
                Markdown => Some(OutputFormat::Markdown),
                Html => Some(OutputFormat::Html),
                Pdf => Some(OutputFormat::Pdf),
                Docx => Some(OutputFormat::Docx),
                Xlsx => Some(OutputFormat::Xlsx),
                Json => Some(OutputFormat::Json),
                Zip => None, // handled by export_zip, not export
            }
        }
    }

    #[derive(Clone)]
    struct TemplatesUiState {
        filter: String,
        selected_idea: Option<usize>,
        choice: ExportChoice,
        status: Option<String>,
        status_error: bool,
    }

    impl Default for TemplatesUiState {
        fn default() -> Self {
            Self {
                filter: String::new(),
                selected_idea: None,
                choice: ExportChoice::Markdown,
                status: None,
                status_error: false,
            }
        }
    }

    fn format_label(f: crate::templates::OutputFormat) -> &'static str {
        use crate::templates::OutputFormat::*;
        match f {
            Markdown => "Markdown",
            Html => "HTML",
            Pdf => "PDF",
            Docx => "DOCX",
            Xlsx => "XLSX",
            Json => "JSON",
        }
    }

    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .replace(' ', "_")
    }

    fn idea_to_product(
        idea: &crate::pipeline::ProductIdea,
    ) -> crate::product_generator::GeneratedProduct {
        crate::product_generator::GeneratedProduct {
            id: idea.id,
            name: idea.title.clone(),
            template_id: idea.product_type.clone(),
            content: idea.description.clone(),
            format: crate::templates::OutputFormat::Markdown,
            created_at: idea.created_at,
            metadata: crate::product_generator::ProductMetadata {
                model_used: "pipeline".to_string(),
                tokens_used: 0,
                generation_time_ms: 0,
                parameters: serde_json::json!({
                    "price": idea.estimated_value,
                    "tags": idea.tags,
                }),
            },
        }
    }

    /// Write a source product out via the real `app.exporter` paths.
    fn run_export(app: &DpfApp, state: &mut TemplatesUiState) {
        // Source: an explicitly picked Pipeline product, else the last product
        // generated by the Create flow.
        let source = state
            .selected_idea
            .and_then(|id| app.pipeline.ideas.iter().find(|i| i.id == id))
            .map(idea_to_product)
            .or_else(|| app.create_state.last_product.clone());

        let Some(mut product) = source else {
            state.status_error = true;
            state.status = Some(
                "Nothing to export — generate a product in Create, or pick a Pipeline product."
                    .to_string(),
            );
            return;
        };
        if let Some(f) = state.choice.output_format() {
            product.format = f;
        }

        if state.choice == ExportChoice::Zip {
            // ZIP is a batch export: all Pipeline products, or just the picked one.
            let items: Vec<crate::product_generator::GeneratedProduct> =
                if state.selected_idea.is_some() {
                    vec![product.clone()]
                } else {
                    app.pipeline.ideas.iter().map(idea_to_product).collect()
                };
            if items.is_empty() {
                state.status_error = true;
                state.status = Some("No products available for ZIP export.".to_string());
                return;
            }
            let default_name = format!("{}_export.zip", sanitize_name(&product.name));
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("ZIP archive", &["zip"])
                .set_file_name(&default_name)
                .save_file()
            {
                let path = path.to_string_lossy().to_string();
                match app.exporter.export_zip(&items, &path) {
                    Ok(p) => {
                        state.status_error = false;
                        state.status = Some(format!(
                            "Wrote {} product(s) to ZIP: {}",
                            items.len(),
                            p
                        ));
                    }
                    Err(e) => {
                        state.status_error = true;
                        state.status = Some(e);
                    }
                }
            }
        } else if let Some(dir) = rfd::FileDialog::new()
            .set_title("Choose export folder")
            .pick_folder()
        {
            let dir = dir.to_string_lossy().to_string();
            match app.exporter.export(&product, &dir) {
                Ok(p) => {
                    state.status_error = false;
                    state.status = Some(format!("Exported {}: {}", state.choice.label(), p));
                }
                Err(e) => {
                    state.status_error = true;
                    state.status = Some(e);
                }
            }
        }
    }

    let state_id = Id::new("dpf_templates_ui_state");
    let mut state: TemplatesUiState =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<TemplatesUiState>(state_id).clone());

    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Templates");
            inline_help::help_button(ui, "templates", &mut app.active_help_topic);
        });
        ui.label("Browse and manage product templates");
        ui.separator();

        // ── Catalogue filter ──────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut state.filter);
            if ui.button("Clear").clicked() {
                state.filter.clear();
            }
        });

        // ── Template catalogue (name · description · category · tags ·
        //    trending score · output format) ───────────────────────────
        let filter = state.filter.trim().to_lowercase();
        let registry = app.generator.get_template_registry();
        let mut templates = registry.list();
        if !filter.is_empty() {
            templates.retain(|t| {
                t.name.to_lowercase().contains(&filter)
                    || t.description.to_lowercase().contains(&filter)
                    || t.category.name().to_lowercase().contains(&filter)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&filter))
            });
        }
        templates.sort_by(|a, b| b.trending_score.cmp(&a.trending_score));

        ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
            ui.label(format!("{} template(s)", templates.len()));
            for t in &templates {
                Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&t.name);
                            ui.label(
                                RichText::new(&t.description)
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Category: {}", t.category.name()))
                                        .size(11.0)
                                        .color(Color32::LIGHT_BLUE),
                                );
                                for tag in &t.tags {
                                    ui.label(
                                        RichText::new(format!("#{}", tag))
                                            .size(10.0)
                                            .color(Color32::LIGHT_GREEN),
                                    );
                                }
                            });
                            if let Some(peak) = &t.seasonal_peak {
                                ui.label(
                                    RichText::new(format!("Seasonal peak: {}", peak))
                                        .size(10.0)
                                        .weak(),
                                );
                            }
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("🔥 {}", t.trending_score));
                            ui.label(format!("Format: {}", format_label(t.output_format)));
                        });
                    });
                });
                ui.add_space(4.0);
            }
            if templates.is_empty() {
                ui.label("No templates match your search.");
            }
        });

        ui.separator();

        // ── Export panel ──────────────────────────────────────────────
        ui.group(|ui| {
            ui.heading("Export a Product");
            ui.label(
                RichText::new(
                    "Writes a product to disk via `app.exporter`. PDF, DOCX and XLSX are \
                     placeholders — see the note under the format picker.",
                )
                .size(11.0)
                .weak(),
            );

            ui.horizontal(|ui| {
                ui.label("Source:");
                let selected_label = state
                    .selected_idea
                    .and_then(|id| app.pipeline.ideas.iter().find(|i| i.id == id))
                    .map(|i| i.title.clone())
                    .unwrap_or_else(|| {
                        if app.create_state.last_product.is_some() {
                            "<last generated product>".to_string()
                        } else {
                            "<none — generate or pick one>".to_string()
                        }
                    });
                egui::ComboBox::from_label("Pipeline product")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.selected_idea,
                            None,
                            "<use last generated product>",
                        );
                        for idea in &app.pipeline.ideas {
                            ui.selectable_value(
                                &mut state.selected_idea,
                                Some(idea.id),
                                &idea.title,
                            );
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Format:");
                for choice in ExportChoice::all() {
                    let text = if choice.is_real() {
                        choice.label().to_string()
                    } else {
                        format!("{} ⚠", choice.label())
                    };
                    if ui
                        .selectable_label(state.choice == choice, text)
                        .clicked()
                    {
                        state.choice = choice;
                    }
                }
            });
            if !state.choice.note().is_empty() {
                ui.colored_label(Color32::YELLOW, state.choice.note());
            }

            ui.horizontal(|ui| {
                if ui.button(RichText::new("⬇ Export").strong()).clicked() {
                    run_export(app, &mut state);
                }
                ui.label(
                    RichText::new("ZIP exports all Pipeline products unless one is picked above.")
                        .size(10.0)
                        .weak(),
                );
            });

            if let Some(msg) = &state.status {
                let color = if state.status_error {
                    Color32::RED
                } else {
                    Color32::LIGHT_GREEN
                };
                ui.colored_label(color, msg);
            }
        });
    });

    ctx.data_mut(|d| d.insert_temp(state_id, state));
}

fn show_bundles(app: &mut DpfApp, ctx: &Context) {
    super::bundle_view::show(app, ctx);
}

fn show_scheduler(app: &mut DpfApp, ctx: &Context) {
    super::scheduler_view::show(app, ctx);
}

fn show_contract(app: &mut DpfApp, ctx: &Context) {
    super::contract_view::show(app, ctx);
}

fn show_settings(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            inline_help::help_button(ui, "settings", &mut app.active_help_topic);
        });

        ui.group(|ui| {
            ui.label("API Keys");
            ui.add(egui::TextEdit::singleline(&mut app.config.openai_key).hint_text("OpenAI API Key"));
            ui.add(egui::TextEdit::singleline(&mut app.config.anthropic_key).hint_text("Anthropic API Key"));
        });

        ui.group(|ui| {
            ui.label("Preferences");
            ui.checkbox(&mut app.config.auto_save, "Auto-save");
            ui.checkbox(&mut app.config.dark_mode, "Dark mode");
        });
    });
}

// ── QC Checklist View ─────────────────────────────────────────────────

fn show_qc(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("✅ Pre-Publish QC Checklist");
            inline_help::help_button(ui, "qc", &mut app.active_help_topic);
        });
        ui.separator();

        let products: Vec<_> = app.pipeline.ideas.iter()
            .filter(|i| i.stage == crate::pipeline::PipelineStage::Review
                     || i.stage == crate::pipeline::PipelineStage::Listed)
            .collect();

        ui.group(|ui| {
            ui.label("Select Product to QC:");
            ui.horizontal(|ui| {
                for product in &products {
                    if ui.selectable_label(
                        app.qc_target_product_id == Some(product.id),
                        &product.title
                    ).clicked() {
                        app.qc_target_product_id = Some(product.id);
                        app.qc_current_result = None;
                        app.qc_manual_approve = false;
                    }
                }
            });
            if products.is_empty() {
                ui.colored_label(Color32::YELLOW, "Move a product to 'Review' stage first.");
            }
        });

        ui.horizontal(|ui| {
            ui.label("Target Platform:");
            for p in &["etsy", "gumroad", "shopify", "payhip"] {
                if ui.selectable_label(app.qc_target_platform == *p, *p).clicked() {
                    app.qc_target_platform = p.to_string();
                    app.qc_current_result = None;
                }
            }
        });

        if let Some(pid) = app.qc_target_product_id {
            if ui.button("▶ Run QC Check").clicked() {
                if let Some(product) = app.pipeline.ideas.iter().find(|i| i.id == pid) {
                    let file_path = product.notes.split('\n').find(|l| l.starts_with("file:"))
                        .map(|l| l[5..].trim().to_string());
                    let file_format = file_path.as_ref()
                        .and_then(|p| std::path::Path::new(p).extension())
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_string());

                    let result = app.qc_engine.run_checklist(
                        pid,
                        &product.title,
                        &app.qc_target_platform,
                        file_path.as_deref(),
                        file_format.as_deref(),
                        &app.publish_manager.platform_formats,
                    );
                    app.qc_current_result = Some(result);
                }
            }
        }

        ui.separator();

        if let Some(result) = &app.qc_current_result {
            let icon = if result.passed { "✅" } else { "❌" };
            ui.heading(format!("{} QC Result: {}", icon, result.product_name));

            for check in &result.checks {
                let (c_icon, c_color) = match check.status {
                    QcStatus::Pass => ("✅", Color32::GREEN),
                    QcStatus::Fail => ("❌", Color32::RED),
                    QcStatus::Warning => ("⚠️", Color32::YELLOW),
                    QcStatus::Skipped => ("⏭️", Color32::GRAY),
                };
                ui.horizontal(|ui| {
                    ui.colored_label(c_color, format!("{} {}", c_icon, check.name));
                    ui.label(&check.detail);
                });
            }

            ui.separator();

            if result.passed {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut app.qc_manual_approve, "I confirm all checks passed");
                    if app.qc_manual_approve {
                        ui.colored_label(Color32::GREEN, "✅ Approved for publish");
                    }
                });
            }
        } else {
            ui.label("Run a QC check to see results here.");
        }
    });
}

// ── Compliance View ───────────────────────────────────────────────────

fn show_compliance(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("⚖️ Compliance & Licensing");
            inline_help::help_button(ui, "compliance", &mut app.active_help_topic);
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("AI Disclosure Rules");
                    for rule in &app.disclosure_rules {
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", rule.platform));
                                if rule.requires_disclosure {
                                    ui.colored_label(Color32::YELLOW, "Required");
                                } else {
                                    ui.colored_label(Color32::GREEN, "Optional");
                                }
                            });
                            ui.label(&rule.disclosure_text_template);
                            ui.label(format!("Location: {}", rule.location));
                        });
                    }
                });

                ui.group(|ui| {
                    ui.heading("AI Tool Licenses");
                    for lic in crate::compliance::AiToolLicense::defaults() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} ({}):", lic.tool_name, lic.plan_tier));
                            if lic.commercial_use_allowed {
                                ui.colored_label(Color32::GREEN, "Commercial OK");
                            } else {
                                ui.colored_label(Color32::RED, "No commercial use");
                            }
                        });
                        if !lic.restrictions.is_empty() {
                            for r in &lic.restrictions {
                                ui.label(format!("  ⚠️ {}", r));
                            }
                        }
                    }
                });
            });

            ui.separator();
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("Trademark/IP Scan");
                    ui.label("Paste your product prompt here...");
                    ui.text_edit_multiline(&mut app.compliance_prompt);

                    if ui.button("🔍 Scan Prompt").clicked() {
                        let flags = app.denylist_scanner.scan(&app.compliance_prompt);
                        app.compliance_scan_result = flags;
                        app.compliance_show_warning = !app.compliance_scan_result.is_empty();
                    }

                    if app.compliance_show_warning {
                        if app.compliance_scan_result.is_empty() {
                            ui.colored_label(Color32::GREEN, "✅ No trademark/IP issues detected.");
                        } else {
                            ui.colored_label(Color32::YELLOW, "⚠️ Trademark/IP Risk Detected");
                            for line in &app.compliance_scan_result {
                                ui.label(line);
                            }
                        }
                    }
                });

                ui.group(|ui| {
                    ui.heading("Denylist");
                    ui.label(format!("{} protected terms loaded.", app.denylist_scanner.entries.len()));
                    ui.label("Edit `revoked_keys.json` or `platform_formats.json` to customize.");
                });
            });
        });
    });
}

// ── Asset Library View ────────────────────────────────────────────────

fn show_asset_library(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🗂️ Asset Library");
            inline_help::help_button(ui, "asset_library", &mut app.active_help_topic);
            if ui.button("🔄 Refresh").clicked() {
                app.asset_library.load_from_db(&app.db);
            }
        });
        ui.separator();

        // Search
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut app.asset_search);
        });

        ui.separator();

        ui.horizontal(|ui| {
            // Left: asset grid
            ui.vertical(|ui| {
                ui.heading("All Assets");
                ui.set_min_width(400.0);

                // Filter with the library's own matcher (product name / tags / file format).
                // Previously this assigned search_query and then immediately overwrote it with the
                // emptied string, so the search box silently did nothing and `filtered` was always
                // every asset. Also removes the `unused_assignments` warning.
                app.asset_library.search_query = app.asset_search.clone();
                let filtered: Vec<_> = app
                    .asset_library
                    .filtered_assets()
                    .into_iter()
                    .cloned()
                    .collect();

                ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                    for asset in &filtered {
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(asset.product_name.to_string());
                                ui.label(format!("[{}]", asset.file_format));
                                if ui.small_button("Select").clicked() {
                                    app.asset_selected_id = Some(asset.id);
                                }
                            });
                            ui.label(format!("Size: {:.1} KB", asset.file_size as f64 / 1024.0));
                            if !asset.tags.is_empty() {
                                ui.label(format!("Tags: {}", asset.tags.join(", ")));
                            }
                            ui.label(format!("Created: {}", asset.created_at.format("%Y-%m-%d")));
                        });
                    }
                    if filtered.is_empty() {
                        ui.label("No assets found. Generate products first.");
                    }
                });
            });

            ui.separator();

            // Right: asset detail + version history
            ui.vertical(|ui| {
                ui.heading("Asset Details");
                if let Some(aid) = app.asset_selected_id {
                    let found_asset = app.asset_library.assets.clone().into_iter().find(|a| a.id == aid); if let Some(asset) = &found_asset {
                        ui.label(format!("Name: {}", asset.product_name));
                        ui.label(format!("Format: .{}", asset.file_format));
                        ui.label(format!("Size: {:.1} KB", asset.file_size as f64 / 1024.0));
                        ui.label(format!("Path: {}", asset.file_path));
                        if !asset.tags.is_empty() {
                            ui.label(format!("Tags: {}", asset.tags.join(", ")));
                        }

                        ui.separator();
                        ui.heading("Version History");

                        // Register new version
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut app.asset_version_notes);
                            ui.label("Change notes");
                            if ui.button("Save New Version").clicked() {
                                app.asset_library.register_version(
                                    &app.db,
                                    asset.product_id,
                                    &asset.file_path,
                                    &app.asset_version_notes,
                                );
                                app.asset_version_notes.clear();
                            }
                        });

                        let versions = app.asset_library.versions_for(asset.product_id);
                        if versions.is_empty() {
                            ui.label("  v1 (initial)");
                        } else {
                            for v in versions.iter().rev() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("v{} · {}", v.version, v.created_at.format("%Y-%m-%d")));
                                    if !v.change_notes.is_empty() {
                                        ui.label(&v.change_notes);
                                    }
                                    if ui.small_button("Rollback").clicked() {
                                        match app.asset_library.rollback_to(asset.product_id, v.version) {
                                            Ok(path) => ui.label(format!("Rolled back to: {}", path)),
                                            Err(e) => ui.colored_label(Color32::RED, &e),
                                        };
                                    }
                                });
                            }
                        }
                    } else {
                        ui.label("Asset not found.");
                    }
                } else {
                    ui.label("Select an asset from the list to view details.");
                }
            });
        });
    });
}

// ── Webhooks View ─────────────────────────────────────────────────────

fn show_webhooks(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🔌 Automation Webhooks");
            inline_help::help_button(ui, "webhooks", &mut app.active_help_topic);
        });
        ui.separator();

        ui.group(|ui| {
            ui.heading("Local HTTP Listener");
            ui.label(
                RichText::new("⚠ Not implemented in this build")
                    .strong()
                    .color(Color32::from_rgb(240, 180, 80)),
            );
            ui.label(
                "These controls do not start a real server — no socket is bound, so nothing can \
                 connect and the status below is not reporting a live listener. They are kept here \
                 as the shape of the planned feature. For automation today use Scheduler and \
                 Publishing, which are implemented.",
            );
            ui.add_space(4.0);

            // Deliberately disabled: previously these buttons reported "listening on localhost"
            // while binding nothing, which was a false capability claim.
            ui.add_enabled_ui(false, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut app.webhook_port);
                    ui.label("Default: 9823");
                });
                ui.horizontal(|ui| {
                    let _ = ui.button("Start Webhook");
                    let _ = ui.button("Stop");
                });
            });

            ui.label(
                RichText::new("Status: 🔴 Not running")
                    .color(Color32::from_rgb(220, 120, 120)),
            );
        });

        ui.separator();

        ui.group(|ui| {
            ui.heading("API Documentation");
            ui.label("POST /generate - Trigger headless generation");
            ui.label("GET  /status   - Health check");
            ui.label("GET  /schema   - API reference");

            ui.separator();
            ui.label("Request Payload (POST /generate):");
            let schema = crate::webhook::request_schema();
            ui.code(serde_json::to_string_pretty(&schema).unwrap_or_default());

            ui.separator();
            ui.label("Endpoint docs auto-served at GET /schema");
            ui.label("Add `callback_url` field to POST /generate for async result notification.");
        });
    });
}
