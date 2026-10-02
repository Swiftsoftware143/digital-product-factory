//! Bundle builder view
//!
//! Wires the Bundle Builder UI to the real `crate::bundler::Bundler` backend:
//! the three auto-bundle buttons call `auto_bundle`, the manual builder calls
//! `create_bundle`, and the statistics panel renders the real output of
//! `calculate_bundle_stats`.

use egui::*;
use crate::app::DpfApp;
use crate::bundler::{BundleStatus, BundleStrategy};
use crate::product_generator::{GeneratedProduct, ProductMetadata};
use crate::templates::OutputFormat;

/// Transient UI state for the Bundle Builder. `DpfApp` owns no bundle-UI fields
/// (and this file must not edit `app.rs`), so the ephemeral form state lives in
/// egui's own per-`Context` memory keyed by a stable `Id`.
#[derive(Clone)]
struct BundleUiState {
    /// pipeline idea ids ticked in the manual builder
    selected: Vec<usize>,
    manual_name: String,
    manual_desc: String,
    discount: u32,
    status: Option<String>,
    status_error: bool,
}

impl Default for BundleUiState {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
            manual_name: String::new(),
            manual_desc: String::new(),
            discount: 20,
            status: None,
            status_error: false,
        }
    }
}

/// Adapt the Pipeline's `ProductIdea`s into the `GeneratedProduct` shape the
/// bundler consumes. The Pipeline is the only product data the app holds at
/// runtime (the Create flow's generated product lives in `app.create_state`).
fn products_from_pipeline(app: &DpfApp) -> Vec<GeneratedProduct> {
    app.pipeline
        .ideas
        .iter()
        .map(|idea| GeneratedProduct {
            id: idea.id,
            name: idea.title.clone(),
            template_id: idea.product_type.clone(),
            content: idea.description.clone(),
            format: format_for_type(&idea.product_type),
            created_at: idea.created_at,
            metadata: ProductMetadata {
                model_used: "pipeline".to_string(),
                tokens_used: 0,
                generation_time_ms: 0,
                parameters: serde_json::json!({
                    "price": idea.estimated_value,
                    "tags": idea.tags,
                }),
            },
        })
        .collect()
}

/// Best-effort mapping of a pipeline product-type string to an output format,
/// for the bundle's ZIP export extension. Falls back to Markdown.
fn format_for_type(product_type: &str) -> OutputFormat {
    let t = product_type.to_lowercase();
    if t.contains("xls") || t.contains("sheet") || t.contains("budget") {
        OutputFormat::Xlsx
    } else if t.contains("doc") || t.contains("contract") {
        OutputFormat::Docx
    } else if t.contains("pdf") || t.contains("planner") || t.contains("journal") {
        OutputFormat::Pdf
    } else if t.contains("html") || t.contains("web") {
        OutputFormat::Html
    } else if t.contains("json") {
        OutputFormat::Json
    } else {
        OutputFormat::Markdown
    }
}

fn strategy_label(strategy: BundleStrategy) -> &'static str {
    match strategy {
        BundleStrategy::ByCategory => "By Category",
        BundleStrategy::ByValue => "By Value",
        BundleStrategy::Seasonal => "Seasonal",
    }
}

fn status_label(status: BundleStatus) -> &'static str {
    match status {
        BundleStatus::Draft => "Draft",
        BundleStatus::Published => "Published",
        BundleStatus::Archived => "Archived",
    }
}

/// Run an auto-bundle strategy against the current Pipeline products.
fn run_auto_bundle(app: &mut DpfApp, state: &mut BundleUiState, strategy: BundleStrategy) {
    let products = products_from_pipeline(app);
    if products.is_empty() {
        state.status_error = true;
        state.status = Some("No Pipeline products to bundle — add ideas on the Pipeline tab first.".to_string());
        return;
    }
    match app.bundler.auto_bundle(&products, strategy) {
        Ok(bundles) => {
            state.status_error = false;
            state.status = Some(if bundles.is_empty() {
                format!(
                    "{} produced no bundles — not enough products (or no category has 3+).",
                    strategy_label(strategy)
                )
            } else {
                format!("{} created {} bundle(s).", strategy_label(strategy), bundles.len())
            });
        }
        Err(e) => {
            state.status_error = true;
            state.status = Some(e);
        }
    }
}

/// Create a manual bundle from the ticked Pipeline products.
fn run_manual_bundle(app: &mut DpfApp, state: &mut BundleUiState) {
    if state.selected.len() < 2 {
        state.status_error = true;
        state.status = Some("A bundle needs at least 2 products.".to_string());
        return;
    }
    let chosen: Vec<GeneratedProduct> = products_from_pipeline(app)
        .into_iter()
        .filter(|p| state.selected.contains(&p.id))
        .collect();
    let name = if state.manual_name.trim().is_empty() {
        "Custom Bundle".to_string()
    } else {
        state.manual_name.trim().to_string()
    };
    let desc = if state.manual_desc.trim().is_empty() {
        "Custom bundle".to_string()
    } else {
        state.manual_desc.trim().to_string()
    };
    match app.bundler.create_bundle(&name, &desc, &chosen, state.discount) {
        Ok(bundle) => {
            state.status_error = false;
            state.status = Some(format!(
                "Created bundle '{}' with {} products.",
                bundle.name,
                bundle.product_ids.len()
            ));
            state.selected.clear();
        }
        Err(e) => {
            state.status_error = true;
            state.status = Some(e);
        }
    }
}

/// Export a bundle as a ZIP via the real `Bundler::export_bundle` path.
fn export_bundle(
    app: &DpfApp,
    bundle: &crate::bundler::Bundle,
    products: &[GeneratedProduct],
    state: &mut BundleUiState,
) {
    if let Some(dir) = rfd::FileDialog::new()
        .set_title("Choose folder for the bundle ZIP")
        .pick_folder()
    {
        let dir = dir.to_string_lossy().to_string();
        match app.bundler.export_bundle(bundle, products, &dir) {
            Ok(path) => {
                state.status_error = false;
                state.status = Some(format!("Exported bundle ZIP: {}", path));
            }
            Err(e) => {
                state.status_error = true;
                state.status = Some(e);
            }
        }
    }
}

pub fn show(app: &mut DpfApp, ctx: &Context) {
    let state_id = Id::new("dpf_bundle_ui_state");
    let mut state: BundleUiState =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<BundleUiState>(state_id).clone());

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Bundle Builder");
        ui.separator();

        let product_count = app.pipeline.ideas.len();

        // ── Auto-bundle strategies ────────────────────────────────────
        ui.group(|ui| {
            ui.heading("Auto-Bundle Strategies");
            ui.label(format!(
                "Operating on {} product(s) from the Pipeline.",
                product_count
            ));
            ui.horizontal(|ui| {
                if ui.button("By Category").clicked() {
                    run_auto_bundle(app, &mut state, BundleStrategy::ByCategory);
                }
                if ui.button("By Value").clicked() {
                    run_auto_bundle(app, &mut state, BundleStrategy::ByValue);
                }
                if ui.button("Seasonal").clicked() {
                    run_auto_bundle(app, &mut state, BundleStrategy::Seasonal);
                }
            });
            ui.label(
                RichText::new(
                    "By Category needs 3+ products sharing a product type · By Value needs 5+ · Seasonal needs 4+",
                )
                .size(10.0)
                .weak(),
            );
        });

        ui.separator();

        // ── Manual bundle builder ─────────────────────────────────────
        ui.group(|ui| {
            ui.heading("Manual Bundle");
            if app.pipeline.ideas.is_empty() {
                ui.label("No products in the Pipeline yet — add ideas first.");
            } else {
                ui.label("Select products to bundle:");
                ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                    for idea in &app.pipeline.ideas {
                        let mut checked = state.selected.contains(&idea.id);
                        if ui
                            .checkbox(
                                &mut checked,
                                format!("{}  (${:.2})", idea.title, idea.estimated_value),
                            )
                            .changed()
                        {
                            if checked {
                                if !state.selected.contains(&idea.id) {
                                    state.selected.push(idea.id);
                                }
                            } else {
                                state.selected.retain(|id| *id != idea.id);
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut state.manual_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut state.manual_desc);
                });
                ui.horizontal(|ui| {
                    ui.label("Discount %:");
                    ui.add(DragValue::new(&mut state.discount).clamp_range(0..=90));
                    ui.label(format!("{} selected", state.selected.len()));
                });
                ui.horizontal(|ui| {
                    if ui.button("Create Bundle").clicked() {
                        run_manual_bundle(app, &mut state);
                    }
                    if ui.button("Clear Selection").clicked() {
                        state.selected.clear();
                    }
                });
            }
        });

        // ── Status line ───────────────────────────────────────────────
        if let Some(msg) = &state.status {
            let color = if state.status_error {
                Color32::RED
            } else {
                Color32::LIGHT_GREEN
            };
            ui.colored_label(color, msg);
        }

        ui.separator();

        // ── Bundle statistics (real calculate_bundle_stats output) ────
        ui.group(|ui| {
            ui.heading("Bundle Statistics");
            let bundles = app.bundler.bundles().to_vec();
            if bundles.is_empty() {
                ui.label("Bundle stats will appear here after you create a bundle.");
            } else {
                let all_products = products_from_pipeline(app);
                ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    for bundle in &bundles {
                        let stats = app.bundler.calculate_bundle_stats(bundle);
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(&bundle.name);
                                ui.label(RichText::new(status_label(bundle.status)).weak());
                            });
                            ui.label(&bundle.description);
                            ui.label(format!(
                                "Products: {} · Total value: ${:.2} · Bundle price: ${:.2}",
                                stats.product_count, stats.total_value, bundle.bundle_price
                            ));
                            ui.label(format!(
                                "Customer savings: ${:.2} ({:.0}% off) · Est. conversion: {:.1}%",
                                stats.customer_savings,
                                stats.savings_percent,
                                stats.estimated_conversion
                            ));
                            if ui.button("Export as ZIP").clicked() {
                                export_bundle(&*app, bundle, &all_products, &mut state);
                            }
                        });
                    }
                });
            }
        });
    });

    ctx.data_mut(|d| d.insert_temp(state_id, state));
}
