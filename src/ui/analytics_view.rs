//! Analytics View — Sales & Performance Tracking
//!
//! Enhanced UI with delete buttons, date range filter,
//! 7-day trend strip chart, and platform breakdown bars.

use egui::*;
use crate::app::DpfApp;
use crate::inline_help;

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("📈 Analytics");
            inline_help::help_button(ui, "analytics", &mut app.active_help_topic);
        });
        ui.separator();

        let estimated = std::collections::HashMap::new();
        let summary = app.analytics.summary(&estimated);

        // ── Stat cards row ──────────────────────────────────────────
        ui.horizontal(|ui| {
            stat_card(ui, "Total Revenue", &format!("${:.2}", summary.total_revenue));
            stat_card(ui, "Total Fees", &format!("${:.2}", summary.total_fees));
            stat_card(ui, "Net Revenue", &format!("${:.2}", summary.total_net));
            stat_card(ui, "Total Sales", &summary.total_units.to_string());
            stat_card(ui, "Products", &summary.product_count.to_string());
        });

        ui.separator();

        // ── 7-Day Trend Chart (horizontal bar strip) ──────────────
        ui.heading("7-Day Revenue Trend");
        let trend = app.analytics.trends(7);
        let max_rev = trend.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
        let bar_height = 18.0;
        let full_width = ui.available_width() - 80.0;

        Frame::group(ui.style()).show(ui, |ui| {
            for (date_label, amount) in &trend {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(date_label.as_str()).size(11.0).weak());
                    let pct = if max_rev > 0.0 { *amount / max_rev } else { 0.0 };
                    let bar_w = (full_width * pct as f32).max(4.0);
                    let (color, hover) = if *amount > 0.0 {
                        (Color32::from_rgb(76, 175, 80), format!("${:.2}", amount))
                    } else {
                        (Color32::GRAY, "$0.00".to_string())
                    };
                    let resp = Frame::none()
                        .fill(color)
                        .rounding(Rounding::same(3.0))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(bar_w, bar_height));
                        })
                        .response;
                    resp.on_hover_text(hover);
                });
            }
            if trend.is_empty() {
                ui.label("  No sales data yet for the last 7 days.");
            }
        });

        ui.separator();

        // ── Revenue by Product ─────────────────────────────────────
        ui.heading("Revenue by Product");
        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            Frame::group(ui.style()).show(ui, |ui| {
                for rev in &summary.by_product {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ${:.2} ({} units)", rev.product_name, rev.revenue, rev.units_sold));
                    });
                }
                if summary.by_product.is_empty() {
                    ui.label("  No sales recorded yet.");
                }
            });
        });

        ui.separator();

        // ── Platform Breakdown (percentage bars) ───────────────────
        ui.heading("Platform Breakdown");
        let plat_bars = app.analytics.platform_breakdown();
        Frame::group(ui.style()).show(ui, |ui| {
            let bar_full = ui.available_width() - 140.0;
            for bar in &plat_bars {
                ui.horizontal(|ui| {
                    ui.set_min_width(80.0);
                    ui.label(bar.platform.to_string());
                    ui.set_min_width(50.0);
                    ui.label(RichText::new(format!("${:.2}", bar.revenue)).size(12.0).strong());
                    let pct = (bar.percentage / 100.0) as f32;
                    let bar_w = (bar_full * pct).max(2.0);
                    let color = platform_color(&bar.platform);
                    let resp = Frame::none()
                        .fill(color)
                        .rounding(Rounding::same(3.0))
                        .show(ui, |ui| {
                            ui.set_min_size(vec2(bar_w, 16.0));
                        })
                        .response;
                    resp.on_hover_text(format!("{:.1}% — ${:.2}", bar.percentage, bar.revenue));
                    ui.label(RichText::new(format!("{:.1}%", bar.percentage)).size(11.0).weak());
                });
            }
            if plat_bars.is_empty() {
                ui.label("  No sales data yet.");
            }
        });

        ui.separator();

        // ── Sales Records — Add button + Date Range Filter ─────────
        ui.horizontal(|ui| {
            ui.heading("Sales Records");
            if ui.button("➕ Add Sale").clicked() {
                app.show_add_sale_dialog = true;
            }
        });

        // Date range filter
        ui.horizontal(|ui| {
            ui.label("From:");
            let mut from_str = app.new_sale.filter_from.clone();
            let resp = ui.text_edit_singleline(&mut from_str);
            if resp.lost_focus() && from_str != app.new_sale.filter_from {
                app.new_sale.filter_from = from_str;
            }
            resp.on_hover_text("YYYY-MM-DD");

            ui.separator();

            ui.label("To:");
            let mut to_str = app.new_sale.filter_to.clone();
            let resp = ui.text_edit_singleline(&mut to_str);
            if resp.lost_focus() && to_str != app.new_sale.filter_to {
                app.new_sale.filter_to = to_str;
            }
            resp.on_hover_text("YYYY-MM-DD");

            ui.separator();

            if ui.button("Clear").clicked() {
                app.new_sale.filter_from.clear();
                app.new_sale.filter_to.clear();
            }
        });

        // Build filtered record list (collect by index to avoid borrow conflicts)
        let filter_from = app.new_sale.filter_from.clone();
        let filter_to = app.new_sale.filter_to.clone();
        let record_indices: Vec<usize> = app.analytics.records.iter()
            .enumerate()
            .filter(|(_, r)| {
                let d = r.sale_date.format("%Y-%m-%d").to_string();
                let pass_from = filter_from.is_empty() || d >= filter_from;
                let pass_to = filter_to.is_empty() || d <= filter_to;
                pass_from && pass_to
            })
            .map(|(i, _)| i)
            .collect();

        // Scrolling list of records with delete buttons
        ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
            Frame::group(ui.style()).show(ui, |ui| {
                for &idx in &record_indices {
                    let id = app.analytics.records[idx].id;
                    let date_str = app.analytics.records[idx].sale_date.format("%Y-%m-%d").to_string();
                    let pname = app.analytics.records[idx].product_name.clone();
                    let units = app.analytics.records[idx].units_sold;
                    let net = app.analytics.records[idx].net_revenue;
                    ui.horizontal(|ui| {
                        if ui.button("🗑").clicked() {
                            app.analytics.delete_sale(&app.db, id);
                        }
                        ui.label(date_str.clone());
                        ui.label(pname.clone());
                        ui.label(format!("{} units", units));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.colored_label(Color32::GREEN, format!("${:.2}", net));
                        });
                    });
                }
                if record_indices.is_empty() {
                    ui.label("  No sales records match the filter.");
                }
            });
        });

        ui.separator();

        // ── Export ──────────────────────────────────────────────────
        ui.heading("Export");
        if ui.button("📥 Export CSV").clicked() {
            let csv = app.analytics.to_csv();
            if let Err(e) = std::fs::write("sales_export.csv", &csv) {
                tracing::error!("Failed to write CSV: {}", e);
            } else {
                tracing::info!("Exported sales to sales_export.csv");
            }
        }
    });
}

// ── Helpers ───────────────────────────────────────────────────────────

fn stat_card(ui: &mut Ui, label: &str, value: &str) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(130.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(value).size(28.0).strong());
            ui.label(label);
        });
    });
}

/// Return a distinct colour for each platform name.
fn platform_color(platform: &str) -> Color32 {
    match platform.to_lowercase().as_str() {
        "etsy"    => Color32::from_rgb(241, 90, 41),
        "gumroad" => Color32::from_rgb(54, 137, 243),
        "shopify" => Color32::from_rgb(121, 191, 82),
        "amazon"  => Color32::from_rgb(255, 153, 0),
        "ebay"    => Color32::from_rgb(0, 84, 165),
        _         => Color32::from_rgb(130, 130, 130),
    }
}

// ── Add Sale Dialog ───────────────────────────────────────────────────

#[derive(Default)]
pub struct NewSaleDraft {
    pub product_name: String,
    pub platform: String,
    pub units_sold: u32,
    pub revenue: f64,
    pub fee: f64,
    // Date range filter fields
    pub filter_from: String,
    pub filter_to: String,
}

pub fn show_add_sale_dialog(app: &mut DpfApp, ctx: &Context) {
    egui::Window::new("Add Sale Record")
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Product:");
                ui.text_edit_singleline(&mut app.new_sale.product_name);
            });
            ui.horizontal(|ui| {
                ui.label("Platform:");
                ui.text_edit_singleline(&mut app.new_sale.platform);
            });
            ui.horizontal(|ui| {
                ui.label("Units:");
                ui.add(DragValue::new(&mut app.new_sale.units_sold).speed(1.0));
            });
            ui.horizontal(|ui| {
                ui.label("Revenue ($):");
                ui.add(DragValue::new(&mut app.new_sale.revenue).speed(0.5));
            });
            ui.horizontal(|ui| {
                ui.label("Fee ($):");
                ui.add(DragValue::new(&mut app.new_sale.fee).speed(0.25));
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let record = crate::analytics::SalesRecord {
                        id: app.analytics.records.len() + 1,
                        product_id: 0,
                        product_name: app.new_sale.product_name.clone(),
                        platform: app.new_sale.platform.clone(),
                        units_sold: app.new_sale.units_sold,
                        revenue: app.new_sale.revenue,
                        fee: app.new_sale.fee,
                        net_revenue: app.new_sale.revenue - app.new_sale.fee,
                        sale_date: chrono::Utc::now(),
                        recorded_at: chrono::Utc::now(),
                        notes: "".to_string(),
                    };
                    let _ = app.db.save_sales_record(&record);
                    app.analytics.records.push(record);
                    app.new_sale = NewSaleDraft::default();
                    app.show_add_sale_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    app.new_sale = NewSaleDraft::default();
                    app.show_add_sale_dialog = false;
                }
            });
        });
}