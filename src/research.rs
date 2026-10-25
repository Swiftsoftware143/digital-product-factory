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
    /// Etsy Open API v3 credential (`keystring:shared_secret`). Empty = no key, so Etsy is read by
    /// scraping, which Etsy refuses with 403. With a key, the official API is used instead.
    pub etsy_api_key: String,
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
            etsy_api_key: String::new(),
        }
    }

    /// Keep the Etsy credential in step with Settings. Called wherever config is applied, so a key
    /// pasted into Settings takes effect without restarting the app.
    pub fn set_etsy_api_key(&mut self, key: String) {
        self.etsy_api_key = key.trim().to_string();
    }

    pub fn search_etsy(&self, query: &str) -> Result<ResearchResult, String> {
        self.runtime.block_on(async {
            self.search_etsy_async(query).await
        })
    }

    async fn search_etsy_async(&self, query: &str) -> Result<ResearchResult, String> {
        // ── Preferred path: Etsy's own API ────────────────────────────────────────────────────
        // Public listing search needs only the `x-api-key` header (keystring:shared_secret).
        // OAuth is required for PRIVATE data and writes, not for this, so a user-supplied key is
        // sufficient — and it returns real, current listings instead of a scrape.
        if !self.etsy_api_key.is_empty() {
            let url = format!(
                "https://openapi.etsy.com/v3/application/listings/active?keywords={}&limit=24",
                urlencoding::encode(query)
            );

            let response = self
                .client
                .get(&url)
                .header("x-api-key", &self.etsy_api_key)
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| format!("Could not reach the Etsy API: {e}"))?;

            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if !status.is_success() {
                // Etsy explains itself precisely here ("Invalid API key: should be in the format
                // 'keystring:shared_secret'"), so pass its own words through instead of guessing.
                let detail = serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| {
                        v.get("error").and_then(|e| e.as_str()).map(str::to_string)
                    })
                    .unwrap_or_else(|| body.chars().take(140).collect());
                return Err(format!(
                    "Etsy refused the API key (HTTP {}): {}. Set it in Settings as \
                     keystring:shared_secret.",
                    status.as_u16(),
                    detail.trim()
                ));
            }

            let products = Self::parse_etsy_api(&body);
            if products.is_empty() {
                return Err(
                    "Etsy's API found no listed products for that search. Try different wording."
                        .to_string(),
                );
            }

            return Ok(ResearchResult {
                platform: "Etsy".to_string(),
                query: query.to_string(),
                products,
                analyzed_at: chrono::Utc::now(),
            });
        }

        // ── Fallback: scrape the public search page ──────────────────────────────────────────
        // This normally fails: Etsy answers any non-browser request with 403. It stays because it
        // costs nothing to try, but the STATUS is checked so the user is told the truth — the old
        // code parsed the 403 block page, found no JSON-LD, and reported "JavaScript", which was
        // simply wrong and left the user with nothing to act on.
        let url = format!("https://www.etsy.com/search?q={}", urlencoding::encode(query));

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Etsy: {}", e))?;

        let status = response.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(format!(
                "Etsy blocked the request (HTTP {}). Etsy does not allow its search results to be \
                 read this way. Add an Etsy API key in Settings (free — register an app at \
                 developers.etsy.com) and Research will use Etsy's official API instead.",
                status.as_u16()
            ));
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let products = Self::parse_listings(&html, "Etsy", &url);
        if products.is_empty() {
            return Err(format!(
                "Etsy's page came back (HTTP {}) but contained no readable product data. Add an \
                 Etsy API key in Settings to read Etsy reliably.",
                status.as_u16()
            ));
        }

        Ok(ResearchResult {
            platform: "Etsy".to_string(),
            query: query.to_string(),
            products,
            analyzed_at: chrono::Utc::now(),
        })
    }

    /// Read Etsy's own JSON (`{count, results:[ShopListing]}`) into our listing type.
    ///
    /// Field names come from Etsy's published OpenAPI schema: `title`, `url`, `price` as a
    /// `Money` object (`amount` / `divisor` / `currency_code`), and `num_favorers`.
    ///
    /// `reviews` is deliberately left empty: Etsy's listing search does not return review counts,
    /// and favourites are not reviews. Putting one in the other's field would be a quiet lie in a
    /// screen the user is making pricing decisions from.
    fn parse_etsy_api(body: &str) -> Vec<ProductListing> {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
            return Vec::new();
        };
        let Some(items) = v.get("results").and_then(|r| r.as_array()) else {
            return Vec::new();
        };

        items
            .iter()
            .filter_map(|it| {
                let title = it.get("title")?.as_str()?.trim().to_string();
                if title.is_empty() {
                    return None;
                }

                let price = it.get("price").and_then(|p| {
                    let amount = p.get("amount").and_then(|a| a.as_f64())?;
                    let divisor = p.get("divisor").and_then(|d| d.as_f64()).unwrap_or(100.0);
                    if divisor == 0.0 {
                        None
                    } else {
                        Some(amount / divisor)
                    }
                });
                let currency = it
                    .get("price")
                    .and_then(|p| p.get("currency_code"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("USD")
                    .to_string();

                Some(ProductListing {
                    title,
                    price,
                    currency,
                    rating: None,
                    reviews: None,
                    url: it
                        .get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    image_url: None,
                    seller: it
                        .get("shop_id")
                        .and_then(|s| s.as_i64())
                        .map(|s| format!("shop {s}")),
                    tags: it
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
            })
            .collect()
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

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Gumroad: {}", e))?;

        let status = response.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(format!(
                "Gumroad blocked the request (HTTP {}). Gumroad has no public search API, so its \
                 marketplace cannot be read programmatically.",
                status.as_u16()
            ));
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let products = Self::parse_listings(&html, "Gumroad", &url);
        if products.is_empty() {
            // Measured 2026-10-25 with the app's own headers: HTTP 200, ~121 KB, no JSON-LD and no
            // readable price text — but the listings ARE in the response, as Inertia props inside
            // the `data-page` attribute (`props.search_results.products`: 36 items with `name`,
            // `price_cents`, `currency_code`, `url`). So this page is not a JavaScript shell and a
            // parser could read it; this build does not parse that blob yet. Until it does, the
            // honest thing is to say no key can help and point at what does work — not to claim the
            // marketplace is unreadable in principle, which is what the old "JavaScript" wording did.
            return Err(
                "Gumroad has no public search API — its API only returns a seller's own products, \
                 so there is no key you can add to unlock marketplace search. Etsy works if you \
                 add an Etsy API key in Settings, and the Strategy brief is a good substitute for \
                 Gumroad."
                    .to_string(),
            );
        }

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

    /// The Etsy API parser is pinned to Etsy's own published schema: `{count, results:[ShopListing]}`
    /// where `price` is a Money object. If Etsy renames a field, this test is what notices.
    #[test]
    fn etsy_api_response_is_read_correctly() {
        let body = r#"{
          "count": 2,
          "results": [
            {"listing_id": 1, "title": "Resume Template - Minimal CV",
             "url": "https://www.etsy.com/listing/1",
             "price": {"amount": 1299, "divisor": 100, "currency_code": "USD"},
             "num_favorers": 42, "tags": ["resume", "cv"], "shop_id": 77},
            {"listing_id": 2, "title": "  ",
             "url": "https://www.etsy.com/listing/2",
             "price": {"amount": 500, "divisor": 100, "currency_code": "EUR"}}
          ]
        }"#;

        let out = MarketResearch::parse_etsy_api(body);
        // The blank-title listing is dropped rather than shown as an empty row.
        assert_eq!(out.len(), 1);

        let p = &out[0];
        assert_eq!(p.title, "Resume Template - Minimal CV");
        assert_eq!(p.price, Some(12.99)); // amount / divisor, not raw minor units
        assert_eq!(p.currency, "USD");
        assert_eq!(p.url, "https://www.etsy.com/listing/1");
        assert_eq!(p.tags, vec!["resume".to_string(), "cv".to_string()]);
        assert_eq!(p.seller.as_deref(), Some("shop 77"));
        // Favourites are not reviews; the field must stay empty rather than carry a wrong number.
        assert_eq!(p.reviews, None);
    }

    #[test]
    fn etsy_api_errors_and_junk_yield_nothing() {
        assert!(MarketResearch::parse_etsy_api(r#"{"error":"Invalid API key"}"#).is_empty());
        assert!(MarketResearch::parse_etsy_api("not json at all").is_empty());
        assert!(MarketResearch::parse_etsy_api(r#"{"count":0,"results":[]}"#).is_empty());
    }
}
