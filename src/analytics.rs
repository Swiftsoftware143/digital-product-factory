//! Analytics module — Sales & Performance Tracking
//!
//! Local SQLite store for sales data, revenue tracking,
//! dashboard metrics, and CSV export.

use crate::database::Database;
use chrono::{DateTime, Utc, NaiveDate};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Data Models ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesRecord {
    pub id: usize,
    pub product_id: usize,
    pub product_name: String,
    pub platform: String,       // "etsy", "gumroad", "shopify", etc.
    pub units_sold: u32,
    pub revenue: f64,
    pub fee: f64,               // platform fees
    pub net_revenue: f64,       // revenue - fee
    pub sale_date: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSummary {
    pub total_revenue: f64,
    pub total_fees: f64,
    pub total_net: f64,
    pub total_units: u64,
    pub product_count: usize,
    pub by_product: Vec<ProductRevenue>,
    pub by_template_type: Vec<CategoryRevenue>,
    pub by_platform: Vec<PlatformRevenue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRevenue {
    pub product_id: usize,
    pub product_name: String,
    pub units_sold: u32,
    pub revenue: f64,
    pub net_revenue: f64,
    pub estimated_value: f64,
    pub margin: f64,  // net - estimated_value (positive = profitable)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRevenue {
    pub template_type: String,
    pub units_sold: u32,
    pub revenue: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRevenue {
    pub platform: String,
    pub units_sold: u32,
    pub revenue: f64,
    pub percentage: f64,
}

/// A single (date_label, net_revenue) pair for trend rendering
pub type TrendPoint = (String, f64);

/// Platform breakdown bar data — each entry has platform name,
/// revenue, and a percentage (0.0–100.0) for bar-width calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformBar {
    pub platform: String,
    pub revenue: f64,
    pub percentage: f64,
}

// ── Analytics Engine ──────────────────────────────────────────────────

pub struct Analytics {
    pub records: Vec<SalesRecord>,
    pub dirty: bool,
}

impl Analytics {
    pub fn new(db: &Database) -> Self {
        let records = db.load_sales_records().unwrap_or_default();
        Self {
            records,
            dirty: false,
        }
    }

    /// Add a new sales record
    pub fn add_sale(&mut self, db: &Database, record: SalesRecord) {
        self.records.push(record.clone());
        db.save_sales_record(&record).ok();
        self.dirty = true;
    }

    /// Delete a sales record
    pub fn delete_sale(&mut self, db: &Database, id: usize) {
        self.records.retain(|r| r.id != id);
        db.delete_sales_record(id).ok();
        self.dirty = true;
    }

    /// Get records for a specific product
    pub fn records_for_product(&self, product_id: usize) -> Vec<&SalesRecord> {
        self.records.iter().filter(|r| r.product_id == product_id).collect()
    }

    /// Total revenue for a product (used for Kanban badge)
    pub fn product_total_revenue(&self, product_id: usize) -> f64 {
        self.records
            .iter()
            .filter(|r| r.product_id == product_id)
            .map(|r| r.net_revenue)
            .sum()
    }

    /// Returns (date_label, net_revenue) tuples for the last N days,
    /// aggregated by calendar day, ascending by date (left-to-right chart).
    pub fn trends(&self, days: i64) -> Vec<TrendPoint> {
        let today = Utc::now().date_naive();
        let mut day_map: HashMap<NaiveDate, f64> = HashMap::new();

        // Initialise all N days with zero so gaps show as $0
        for offset in 0..days {
            let d = today - chrono::Duration::days(offset);
            day_map.entry(d).or_insert(0.0);
        }

        for record in &self.records {
            let record_date = record.sale_date.date_naive();
            let diff = (today - record_date).num_days();
            if diff >= 0 && diff < days {
                *day_map.entry(record_date).or_insert(0.0) += record.net_revenue;
            }
        }

        let mut result: Vec<(NaiveDate, f64)> = day_map.into_iter().collect();
        // Sort by date ascending so chart reads left-to-right
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result.into_iter()
            .map(|(d, rev)| (d.format("%b %d").to_string(), rev))
            .collect()
    }

    /// Platform breakdown as percentage bars.
    /// Returns sorted vector (highest revenue first).
    pub fn platform_breakdown(&self) -> Vec<PlatformBar> {
        let total: f64 = self.records.iter().map(|r| r.revenue).sum();
        let mut plat_map: HashMap<String, f64> = HashMap::new();
        for r in &self.records {
            *plat_map.entry(r.platform.clone()).or_insert(0.0) += r.revenue;
        }

        let mut bars: Vec<PlatformBar> = plat_map
            .into_iter()
            .map(|(platform, revenue)| PlatformBar {
                percentage: if total > 0.0 { revenue / total * 100.0 } else { 0.0 },
                revenue,
                platform,
            })
            .collect();

        bars.sort_by(|a, b| b.revenue.partial_cmp(&a.revenue).unwrap_or(std::cmp::Ordering::Equal));
        bars
    }

    /// Build full revenue summary
    pub fn summary(&self, estimated_values: &HashMap<usize, f64>) -> RevenueSummary {
        let total_revenue: f64 = self.records.iter().map(|r| r.revenue).sum();
        let total_fees: f64 = self.records.iter().map(|r| r.fee).sum();
        let total_net = total_revenue - total_fees;
        let total_units: u64 = self.records.iter().map(|r| r.units_sold as u64).sum();

        // By product
        let mut product_map: HashMap<usize, (String, u32, f64, f64)> = HashMap::new();
        for r in &self.records {
            let entry = product_map.entry(r.product_id).or_insert((
                r.product_name.clone(), 0, 0.0, 0.0
            ));
            entry.1 += r.units_sold;
            entry.2 += r.revenue;
            entry.3 += r.net_revenue;
        }

        let by_product: Vec<ProductRevenue> = product_map
            .into_iter()
            .map(|(pid, (name, units, rev, net))| {
                let est = estimated_values.get(&pid).copied().unwrap_or(0.0);
                ProductRevenue {
                    product_id: pid,
                    product_name: name,
                    units_sold: units,
                    revenue: rev,
                    net_revenue: net,
                    estimated_value: est,
                    margin: net - est,
                }
            })
            .collect();

        // By template type (from notes field which stores template type)
        let mut cat_map: HashMap<String, (u32, f64)> = HashMap::new();
        for r in &self.records {
            if total_net > 0.0 {
                let entry = cat_map.entry(r.notes.clone()).or_insert((0, 0.0));
                entry.0 += r.units_sold;
                entry.1 += r.revenue;
            }
        }

        let by_template_type: Vec<CategoryRevenue> = cat_map
            .into_iter()
            .map(|(t, (units, rev))| CategoryRevenue {
                template_type: t,
                units_sold: units,
                revenue: rev,
                percentage: if total_net > 0.0 { rev / total_net * 100.0 } else { 0.0 },
            })
            .collect();

        // By platform
        let mut plat_map: HashMap<String, (u32, f64)> = HashMap::new();
        for r in &self.records {
            let entry = plat_map.entry(r.platform.clone()).or_insert((0, 0.0));
            entry.0 += r.units_sold;
            entry.1 += r.revenue;
        }

        let by_platform: Vec<PlatformRevenue> = plat_map
            .into_iter()
            .map(|(p, (units, rev))| PlatformRevenue {
                platform: p,
                units_sold: units,
                revenue: rev,
                percentage: if total_net > 0.0 { rev / total_net * 100.0 } else { 0.0 },
            })
            .collect();

        RevenueSummary {
            total_revenue,
            total_fees,
            total_net,
            total_units,
            product_count: by_product.len(),
            by_product,
            by_template_type,
            by_platform,
        }
    }

    /// Export full sales ledger as CSV string
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("ID,Product ID,Product Name,Platform,Units Sold,Revenue,Fees,Net Revenue,Sale Date,Notes\n");
        for r in &self.records {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.2},{:.2},{:.2},{},{}\n",
                r.id, r.product_id, r.product_name, r.platform,
                r.units_sold, r.revenue, r.fee, r.net_revenue,
                r.sale_date.format("%Y-%m-%d"), r.notes
            ));
        }
        csv
    }
}