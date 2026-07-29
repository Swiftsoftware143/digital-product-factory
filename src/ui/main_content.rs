//! Main content area — routes to current tab

use egui::*;
use crate::app::{DpfApp, Tab};
use crate::inline_help;
use crate::qc::{QcResult, QcCheck, QcStatus};
use crate::compliance::DenylistScanner;
use super::adverts_view;
use super::{pipeline_view, analytics_view, publish_view, mockup_view, admin_view, variants_view};

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
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Dashboard");
            inline_help::help_button(ui, "dashboard", &mut app.active_help_topic);
        });
        ui.separator();

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
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Templates");
        inline_help::help_button(ui, "templates", &mut app.active_help_topic);
        ui.label("Browse and manage product templates");
    });
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

                let mut current_search = app.asset_search.clone();
                app.asset_library.search_query = std::mem::take(&mut current_search);
                let filtered: Vec<_> = app.asset_library.assets.clone().to_vec();
                app.asset_library.search_query = current_search;

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
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut app.webhook_port);
                ui.label("Default: 9823");
            });

            ui.horizontal(|ui| {
                let is_running = app.webhook_state.is_running();
                let status = if is_running { "🟢 Running" } else { "🔴 Stopped" };
                ui.label(format!("Status: {}", status));
            });

            ui.horizontal(|ui| {
                if ui.button("Start Webhook").clicked() {
                    let port: u16 = app.webhook_port.parse().unwrap_or(9823);
                    app.webhook_state = crate::webhook::WebhookState::new(true, port);
                    app.webhook_state.running.store(true, std::sync::atomic::Ordering::Relaxed);
                    app.webhook_status_message = format!("Webhook listening on localhost:{}", port);
                }
                if ui.button("Stop").clicked() {
                    app.webhook_state.running.store(false, std::sync::atomic::Ordering::Relaxed);
                    app.webhook_state.enabled = false;
                    app.webhook_status_message = "Webhook stopped.".into();
                }
            });

            if !app.webhook_status_message.is_empty() {
                ui.colored_label(Color32::YELLOW, &app.webhook_status_message);
            }
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
