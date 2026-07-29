//! Market research module - Etsy, Gumroad, Amazon scraping

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MarketResearch {
    client: Client,
    runtime: Arc<Runtime>,
    pub search_query: String,
    pub search_results: Vec<ResearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub platform: String,
    pub query: String,
    pub products: Vec<ProductListing>,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListing {
    pub title: String,
    pub price: Option<f64>,
    pub currency: String,
    pub rating: Option<f64>,
    pub reviews: Option<u32>,
    pub url: String,
    pub image_url: Option<String>,
    pub seller: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketInsight {
    pub avg_price: f64,
    pub price_range: (f64, f64),
    pub top_keywords: Vec<(String, u32)>,
    pub competition_level: CompetitionLevel,
    pub opportunity_score: u32, // 0-100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CompetitionLevel {
    Low,
    Medium,
    High,
    Saturated,
}

impl MarketResearch {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            runtime,
            search_query: String::new(),
            search_results: Vec::new(),
        }
    }

    pub fn search_etsy(&self, query: &str) -> Result<ResearchResult, String> {
        self.runtime.block_on(async {
            self.search_etsy_async(query).await
        })
    }

    async fn search_etsy_async(&self, query: &str) -> Result<ResearchResult, String> {
        // Note: This is a simplified implementation
        // Real implementation would use Etsy's API or proper scraping
        let url = format!("https://www.etsy.com/search?q={}", urlencoding::encode(query));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Etsy: {}", e))?;

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Parse HTML (simplified)
        let products = self.parse_etsy_listings(&html);

        Ok(ResearchResult {
            platform: "Etsy".to_string(),
            query: query.to_string(),
            products,
            analyzed_at: chrono::Utc::now(),
        })
    }

    fn parse_etsy_listings(&self, _html: &str) -> Vec<ProductListing> {
        // This is a placeholder - real implementation would parse HTML
        // Etsy uses JavaScript rendering, so this would need Puppeteer/Playwright in production
        Vec::new()
    }

    pub fn search_gumroad(&self, query: &str) -> Result<ResearchResult, String> {
        self.runtime.block_on(async {
            self.search_gumroad_async(query).await
        })
    }

    async fn search_gumroad_async(&self, query: &str) -> Result<ResearchResult, String> {
        let url = format!("https://gumroad.com/discover?query={}", urlencoding::encode(query));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Gumroad: {}", e))?;

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let products = self.parse_gumroad_listings(&html);

        Ok(ResearchResult {
            platform: "Gumroad".to_string(),
            query: query.to_string(),
            products,
            analyzed_at: chrono::Utc::now(),
        })
    }

    fn parse_gumroad_listings(&self, _html: &str) -> Vec<ProductListing> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn analyze_market(&self, results: &[ResearchResult]) -> MarketInsight {
        let all_products: Vec<_> = results.iter()
            .flat_map(|r| r.products.clone())
            .collect();

        if all_products.is_empty() {
            return MarketInsight {
                avg_price: 0.0,
                price_range: (0.0, 0.0),
                top_keywords: vec![],
                competition_level: CompetitionLevel::Low,
                opportunity_score: 50,
            };
        }

        // Calculate price stats
        let prices: Vec<f64> = all_products.iter()
            .filter_map(|p| p.price)
            .collect();

        let avg_price = if prices.is_empty() {
            0.0
        } else {
            prices.iter().sum::<f64>() / prices.len() as f64
        };

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(0.0, f64::max);

        // Extract keywords from titles
        let mut keyword_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for product in &all_products {
            let words: Vec<_> = product.title
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(|w| w.to_string())
                .collect();

            for word in words {
                *keyword_counts.entry(word).or_insert(0) += 1;
            }
        }

        let mut top_keywords: Vec<_> = keyword_counts.into_iter().collect();
        top_keywords.sort_by(|a, b| b.1.cmp(&a.1));
        let top_keywords = top_keywords.into_iter().take(10).collect();

        // Determine competition level
        let competition_level = match all_products.len() {
            0..=10 => CompetitionLevel::Low,
            11..=50 => CompetitionLevel::Medium,
            51..=200 => CompetitionLevel::High,
            _ => CompetitionLevel::Saturated,
        };

        // Calculate opportunity score (simplified)
        let opportunity_score = match competition_level {
            CompetitionLevel::Low => 85,
            CompetitionLevel::Medium => 70,
            CompetitionLevel::High => 50,
            CompetitionLevel::Saturated => 30,
        };

        MarketInsight {
            avg_price,
            price_range: (min_price, max_price),
            top_keywords,
            competition_level,
            opportunity_score,
        }
    }

    pub fn trending_searches(&self) -> Vec<String> {
        vec![
            "planner 2026".to_string(),
            "digital journal".to_string(),
            "budget tracker".to_string(),
            "social media templates".to_string(),
            "notion template".to_string(),
            "resume template".to_string(),
            "wedding planner".to_string(),
            "fitness tracker".to_string(),
        ]
    }
}
