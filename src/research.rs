//! Market research module - Etsy and Gumroad listing search.
//! Amazon is NOT implemented here; the research view says so on screen.

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
    /// Platform selection. These were local variables inside the frame before, so they reset
    /// every repaint and were never actually read by anything.
    /// Named `use_*` to avoid clashing with the `search_etsy` / `search_gumroad` methods.
    pub use_etsy: bool,
    pub use_gumroad: bool,
    /// Result of the last search, shown to the user. `None` until a search has run.
    pub last_search_note: Option<(bool, String)>,
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

impl CompetitionLevel {
    /// Human label for the UI. On the type itself so a new variant cannot be added without
    /// deciding what to call it on screen.
    pub fn name(&self) -> &'static str {
        match self {
            CompetitionLevel::Low => "low",
            CompetitionLevel::Medium => "medium",
            CompetitionLevel::High => "high",
            CompetitionLevel::Saturated => "saturated",
        }
    }
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
            use_etsy: true,
            use_gumroad: true,
            last_search_note: None,
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
        let products = Self::parse_listings(&html, "Etsy", &url);

        Ok(ResearchResult {
            platform: "Etsy".to_string(),
            query: query.to_string(),
            products,
            analyzed_at: chrono::Utc::now(),
        })
    }

    /// Extract product listings from a storefront page.
    ///
    /// Parses embedded JSON-LD (`<script type="application/ld+json">`), which is the standard
    /// machine-readable product format that storefronts publish for search engines.
    ///
    /// This replaced `parse_etsy_listings` / `parse_gumroad_listings`, which each took the HTML,
    /// ignored it completely and returned an empty `Vec`. Because the callers wrapped that in
    /// `Ok`, every search reported success and always found exactly zero products — a failure
    /// that looked like a working feature with no results.
    ///
    /// Honest limitation: a site that renders its results only in JavaScript and publishes no
    /// JSON-LD genuinely cannot be read this way. The caller reports that rather than showing a
    /// blank list.
    fn parse_listings(html: &str, platform: &str, fallback_url: &str) -> Vec<ProductListing> {
        let mut out: Vec<ProductListing> = Vec::new();

        for block in extract_json_ld_blocks(html) {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&block) else {
                continue;
            };
            collect_product_nodes(&value, platform, fallback_url, &mut out);
        }

        out.truncate(40);
        out
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

        let products = Self::parse_listings(&html, "Gumroad", &url);

        Ok(ResearchResult {
            platform: "Gumroad".to_string(),
            query: query.to_string(),
            products,
            analyzed_at: chrono::Utc::now(),
        })
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

/// Pull out every `<script type="application/ld+json">…</script>` block, free of regex risk.
fn extract_json_ld_blocks(html: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(start_rel) = lower[cursor..].find("application/ld+json") {
        let start = cursor + start_rel;
        let Some(open_end_rel) = html[start..].find('>') else {
            break;
        };
        let content_start = start + open_end_rel + 1;
        let Some(close_rel) = lower[content_start..].find("</script>") else {
            break;
        };
        let content_end = content_start + close_rel;
        blocks.push(html[content_start..content_end].trim().to_string());
        cursor = content_end + 1;
    }

    blocks
}

/// Walk a JSON-LD tree collecting anything typed as a Product.
///
/// Recurses everywhere rather than assuming a shape: storefronts nest products in
/// `ItemList.itemListElement`, in `@graph`, or at the top level, and the type may be a string or
/// an array of strings.
fn collect_product_nodes(
    value: &serde_json::Value,
    platform: &str,
    fallback_url: &str,
    out: &mut Vec<ProductListing>,
) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_product_nodes(item, platform, fallback_url, out);
            }
        }
        serde_json::Value::Object(map) => {
            if is_product_node(map) {
                if let Some(title) = map.get("name").and_then(|n| n.as_str()) {
                    let title = title.trim().to_string();
                    if !title.is_empty() {
                        let aggregate = map.get("aggregateRating");
                        out.push(ProductListing {
                            title,
                            price: extract_price(map),
                            currency: "USD".to_string(),
                            rating: aggregate
                                .and_then(|a| a.get("ratingValue"))
                                .and_then(|r| r.as_f64()),
                            reviews: aggregate
                                .and_then(|a| a.get("reviewCount"))
                                .and_then(|r| r.as_u64())
                                .map(|n| n as u32),
                            url: map
                                .get("url")
                                .and_then(|u| u.as_str())
                                .unwrap_or(fallback_url)
                                .to_string(),
                            image_url: extract_image(map),
                            seller: extract_seller(map),
                            tags: extract_tags(map),
                        });
                    }
                }
            }

            for child in map.values() {
                collect_product_nodes(child, platform, fallback_url, out);
            }
        }
        _ => {}
    }
}

fn is_product_node(map: &serde_json::Map<String, serde_json::Value>) -> bool {
    match map.get("@type") {
        Some(serde_json::Value::String(s)) => s.eq_ignore_ascii_case("Product"),
        Some(serde_json::Value::Array(a)) => a
            .iter()
            .any(|x| x.as_str().map(|s| s.eq_ignore_ascii_case("Product")).unwrap_or(false)),
        _ => false,
    }
}

/// Read a price from `offers`, which may be an object or an array of objects.
fn extract_price(map: &serde_json::Map<String, serde_json::Value>) -> Option<f64> {
    let offers = map.get("offers")?;
    match offers {
        serde_json::Value::Object(o) => o
            .get("price")
            .and_then(as_price)
            .or_else(|| o.get("lowPrice").and_then(as_price)),
        serde_json::Value::Array(items) => items.iter().find_map(|o| {
            o.get("price")
                .and_then(as_price)
                .or_else(|| o.get("lowPrice").and_then(as_price))
        }),
        _ => None,
    }
}

/// Prices arrive as numbers ("12.99") or strings ("$12.99" / "USD 12.99").
fn as_price(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s
            .trim_start_matches(|c: char| !c.is_ascii_digit() && c != '.')
            .parse::<f64>()
            .ok(),
        _ => None,
    }
}

fn extract_image(map: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    match map.get("image") {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Array(a)) => a.first().and_then(|x| x.as_str()).map(String::from),
        Some(serde_json::Value::Object(o)) => {
            o.get("url").and_then(|u| u.as_str()).map(String::from)
        }
        _ => None,
    }
}

fn extract_seller(map: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    match map.get("brand") {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Object(o)) => {
            o.get("name").and_then(|n| n.as_str()).map(String::from)
        }
        _ => None,
    }
}

fn extract_tags(map: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    match map.get("keywords") {
        Some(serde_json::Value::String(s)) => s
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect(),
        Some(serde_json::Value::Array(a)) => {
            a.iter().filter_map(|x| x.as_str().map(String::from)).collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page with JSON-LD must actually yield products. This is the regression guard for the
    /// old stubs, which returned an empty Vec no matter what HTML they were given — so every
    /// search reported success with zero results, forever.
    #[test]
    fn json_ld_products_are_parsed_not_ignored() {
        let html = r#"<html><head>
        <script type="application/ld+json">
        {"@type":"ItemList","itemListElement":[
          {"@type":"Product","name":"Budget Planner","url":"https://ex.test/a",
           "image":["https://ex.test/a.png"],
           "brand":{"@type":"Brand","name":"Acme"},
           "keywords":"budget, planner",
           "offers":{"@type":"Offer","price":"12.99","priceCurrency":"USD"},
           "aggregateRating":{"ratingValue":4.7,"reviewCount":213}},
          {"@type":"Product","name":"  ","offers":{"price":5}}
        ]}
        </script></head><body>JS-rendered junk</body></html>"#;

        let found = MarketResearch::parse_listings(html, "Etsy", "https://ex.test");
        assert_eq!(found.len(), 1, "a blank-named node must be dropped: {found:?}");

        let p = &found[0];
        assert_eq!(p.title, "Budget Planner");
        assert_eq!(p.price, Some(12.99));
        assert_eq!(p.url, "https://ex.test/a");
        assert_eq!(p.image_url.as_deref(), Some("https://ex.test/a.png"));
        assert_eq!(p.seller.as_deref(), Some("Acme"));
        assert_eq!(p.rating, Some(4.7));
        assert_eq!(p.reviews, Some(213));
        assert_eq!(p.tags, vec!["budget".to_string(), "planner".to_string()]);
    }

    /// A JavaScript-only page has nothing for us to read. The parser must return empty rather
    /// than invent content — the caller then reports the limitation honestly.
    #[test]
    fn a_page_without_json_ld_yields_nothing() {
        let found = MarketResearch::parse_listings(
            "<html><body><div id=app></div></body></html>",
            "Etsy",
            "https://ex.test",
        );
        assert!(found.is_empty());
    }

    #[test]
    fn malformed_json_ld_does_not_panic() {
        let html = r#"<script type="application/ld+json">{ this is not json </script>"#;
        assert!(MarketResearch::parse_listings(html, "Etsy", "https://ex.test").is_empty());
    }

    #[test]
    fn price_strings_and_numbers_both_parse() {
        assert_eq!(as_price(&serde_json::json!(12.99)), Some(12.99));
        assert_eq!(as_price(&serde_json::json!("$12.99")), Some(12.99));
        assert_eq!(as_price(&serde_json::json!("USD 1,234.50")), None); // comma => not parsed
        assert_eq!(as_price(&serde_json::json!(null)), None);
    }
}
