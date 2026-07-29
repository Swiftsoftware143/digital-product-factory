//! Bundle creation module - combine multiple products into bundles

use crate::product_generator::GeneratedProduct;
use crate::exporter::Exporter;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub product_ids: Vec<usize>,
    pub discount_percent: u32,
    pub total_value: f64,
    pub bundle_price: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: BundleStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BundleStatus {
    Draft,
    Published,
    Archived,
}

pub struct Bundler {
    bundles: Vec<Bundle>,
    exporter: Exporter,
}

impl Bundler {
    pub fn new() -> Self {
        Self {
            bundles: Vec::new(),
            exporter: Exporter::new(),
        }
    }
    
    pub fn create_bundle(
        &mut self,
        name: &str,
        description: &str,
        products: &[GeneratedProduct],
        discount_percent: u32,
    ) -> Result<Bundle, String> {
        if products.len() < 2 {
            return Err("Bundle must contain at least 2 products".to_string());
        }
        
        let total_value: f64 = products.iter()
            .map(|p| p.metadata.parameters.get("price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0))
            .sum();
        
        let discount_multiplier = 1.0 - (discount_percent as f64 / 100.0);
        let bundle_price = total_value * discount_multiplier;
        
        let bundle = Bundle {
            id: self.bundles.len(),
            name: name.to_string(),
            description: description.to_string(),
            product_ids: products.iter().map(|p| p.id).collect(),
            discount_percent,
            total_value,
            bundle_price,
            created_at: chrono::Utc::now(),
            status: BundleStatus::Draft,
        };
        
        self.bundles.push(bundle.clone());
        Ok(bundle)
    }
    
    pub fn auto_bundle(&mut self, products: &[GeneratedProduct], strategy: BundleStrategy) -> Result<Vec<Bundle>, String> {
        let mut bundles = Vec::new();
        
        match strategy {
            BundleStrategy::ByCategory => {
                // Group by template category
                let mut by_category: HashMap<String, Vec<&GeneratedProduct>> = HashMap::new();
                for product in products {
                    by_category.entry(product.template_id.clone())
                        .or_default()
                        .push(product);
                }
                
                for (category, category_products) in by_category {
                    if category_products.len() >= 3 {
                        let bundle = self.create_bundle(
                            &format!("{} Bundle", category),
                            &format!("Complete {} collection", category),
                            &category_products.iter().map(|&p| p.clone()).collect::<Vec<_>>(),
                            20, // 20% discount
                        )?;
                        bundles.push(bundle);
                    }
                }
            },
            BundleStrategy::ByValue => {
                // Create value-tiered bundles
                let mut sorted = products.to_vec();
                sorted.sort_by(|a, b| {
                    let a_val = a.metadata.parameters.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b_val = b.metadata.parameters.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    b_val.partial_cmp(&a_val).unwrap()
                });
                
                // Create premium bundle (top 5)
                if sorted.len() >= 5 {
                    let premium = self.create_bundle(
                        "Premium Collection",
                        "Our best products at one great price",
                        &sorted[..5],
                        25,
                    )?;
                    bundles.push(premium);
                }
                
                // Create starter bundle (next 5 or all if less)
                let remaining = if sorted.len() > 5 { &sorted[5..] } else { &sorted[..] };
                if remaining.len() >= 3 {
                    let starter = self.create_bundle(
                        "Starter Pack",
                        "Perfect for getting started",
                        &remaining[..remaining.len().min(5)],
                        30,
                    )?;
                    bundles.push(starter);
                }
            },
            BundleStrategy::Seasonal => {
                // Create seasonal bundles based on current month
                let month = chrono::Local::now().month();
                let (season, theme) = match month {
                    1..=3 => ("Winter", "cozy"),
                    4..=6 => ("Spring", "fresh start"),
                    7..=9 => ("Summer", "productivity"),
                    _ => ("Fall", "back to routine"),
                };
                
                if products.len() >= 4 {
                    let seasonal = self.create_bundle(
                        &format!("{} Collection", season),
                        &format!("{} themed digital products", theme),
                        &products[..products.len().min(6)],
                        20,
                    )?;
                    bundles.push(seasonal);
                }
            },
        }
        
        Ok(bundles)
    }
    
    pub fn export_bundle(&self, bundle: &Bundle, products: &[GeneratedProduct], output_dir: &str) -> Result<String, String> {
        let bundle_products: Vec<_> = products.iter()
            .filter(|p| bundle.product_ids.contains(&p.id))
            .cloned()
            .collect();
        
        let output_path = format!("{}/{}_bundle.zip", output_dir, self.sanitize_filename(&bundle.name));
        self.exporter.export_zip(&bundle_products, &output_path)
    }
    
    pub fn calculate_bundle_stats(&self, bundle: &Bundle) -> BundleStats {
        BundleStats {
            product_count: bundle.product_ids.len(),
            total_value: bundle.total_value,
            customer_savings: bundle.total_value - bundle.bundle_price,
            savings_percent: bundle.discount_percent,
            estimated_conversion: if bundle.discount_percent >= 30 { 8.0 } else { 5.0 },
        }
    }
    
    pub fn bundles(&self) -> &[Bundle] {
        &self.bundles
    }
    
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "_")
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum BundleStrategy {
    ByCategory,
    ByValue,
    Seasonal,
}

#[allow(dead_code)]
pub struct BundleStats {
    pub product_count: usize,
    pub total_value: f64,
    pub customer_savings: f64,
    pub savings_percent: u32,
    pub estimated_conversion: f64, // Percentage
}
