//! Publishing module — Marketplace Publishing
//!
//! Pluggable adapter pattern for marketplace platforms.
//! Currently implemented: Etsy, Gumroad
//! Stubs: Shopify, Payhip
//!
//! Keys are stored via OS keychain (keyring crate) — not plaintext.

use crate::database::Database;
use chrono::{DateTime, Utc};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Error type ────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum PublishError {
    Network(String),
    Auth(String),
    Validation(String),
    Platform(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishError::Network(m) => write!(f, "Network error: {}", m),
            PublishError::Auth(m) => write!(f, "Auth error: {}", m),
            PublishError::Validation(m) => write!(f, "Validation error: {}", m),
            PublishError::Platform(m) => write!(f, "Platform error: {}", m),
        }
    }
}

// ── Publish Log ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishLog {
    pub id: usize,
    pub product_id: usize,
    pub product_name: String,
    pub platform: String,
    pub listing_url: Option<String>,
    pub listing_id: Option<String>,
    pub status: PublishStatus,
    pub error_message: Option<String>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishStatus {
    Pending,
    Published,
    Failed,
    Removed,
}

// ── Platform Formatting Config ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFormat {
    pub name: String,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
    pub max_file_size_mb: u32,
    pub max_tags: usize,
    pub max_title_length: usize,
    pub max_description_length: usize,
    pub allowed_formats: Vec<String>,
    pub digital_download: bool,
}

impl PlatformFormat {
    pub fn defaults() -> HashMap<String, Self> {
        let mut m = HashMap::new();
        m.insert("etsy".into(), PlatformFormat {
            name: "Etsy".into(),
            thumbnail_width: 3000,
            thumbnail_height: 3000,
            max_file_size_mb: 20,
            max_tags: 13,
            max_title_length: 140,
            max_description_length: 5000,
            allowed_formats: vec!["pdf".into(), "zip".into(), "png".into(), "jpg".into()],
            digital_download: true,
        });
        m.insert("gumroad".into(), PlatformFormat {
            name: "Gumroad".into(),
            thumbnail_width: 1280,
            thumbnail_height: 720,
            max_file_size_mb: 50,
            max_tags: 0,  // Gumroad doesn't use tags
            max_title_length: 255,
            max_description_length: 10000,
            allowed_formats: vec!["pdf".into(), "zip".into(), "epub".into(), "mp4".into()],
            digital_download: true,
        });
        m.insert("shopify".into(), PlatformFormat {
            name: "Shopify".into(),
            thumbnail_width: 2048,
            thumbnail_height: 2048,
            max_file_size_mb: 20,
            max_tags: 0,
            max_title_length: 255,
            max_description_length: 5000,
            allowed_formats: vec!["pdf".into(), "zip".into(), "jpg".into(), "png".into()],
            digital_download: true,
        });
        m.insert("payhip".into(), PlatformFormat {
            name: "Payhip".into(),
            thumbnail_width: 1200,
            thumbnail_height: 1200,
            max_file_size_mb: 25,
            max_tags: 0,
            max_title_length: 150,
            max_description_length: 4000,
            allowed_formats: vec!["pdf".into(), "zip".into(), "jpg".into(), "png".into()],
            digital_download: true,
        });
        m
    }
}

// ── Publishing Adapter Trait ──────────────────────────────────────────

/// Each marketplace implements this trait.
/// All methods return Err gracefully when offline — no crash.
#[async_trait::async_trait]
pub trait PublishAdapter: Send + Sync {
    fn platform_name(&self) -> &'static str;

    /// Authenticate with stored credentials. Returns Ok if session/token is valid.
    async fn authenticate(&self, api_key: &str) -> Result<bool, PublishError>;

    /// List a product on the marketplace.
    async fn list_product(
        &self,
        api_key: &str,
        title: &str,
        description: &str,
        price: f64,
        file_path: Option<&str>,
    ) -> Result<(String, String), PublishError>;  // (listing_id, listing_url)

    /// Update an existing listing.
    async fn update_listing(
        &self,
        api_key: &str,
        listing_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        price: Option<f64>,
    ) -> Result<bool, PublishError>;

    /// Check status of a listing.
    async fn check_status(&self, api_key: &str, listing_id: &str) -> Result<String, PublishError>;

    /// Remove a listing from the marketplace.
    async fn remove_listing(&self, api_key: &str, listing_id: &str) -> Result<bool, PublishError>;
}

// ── Keychain Helpers ──────────────────────────────────────────────────

const KEYCHAIN_SERVICE: &str = "digital-product-factory";

pub fn store_api_key(platform: &str, key: &str) -> Result<(), PublishError> {
    let entry = Entry::new(KEYCHAIN_SERVICE, &format!("api_key_{}", platform))
        .map_err(|e| PublishError::Auth(format!("keychain error: {}", e)))?;
    entry.set_password(key)
        .map_err(|e| PublishError::Auth(format!("failed to store key: {}", e)))
}

pub fn get_api_key(platform: &str) -> Option<String> {
    let entry = Entry::new(KEYCHAIN_SERVICE, &format!("api_key_{}", platform)).ok()?;
    entry.get_password().ok()
}

pub fn delete_api_key(platform: &str) -> Result<(), PublishError> {
    let entry = Entry::new(KEYCHAIN_SERVICE, &format!("api_key_{}", platform))
        .map_err(|e| PublishError::Auth(format!("keychain error: {}", e)))?;
    entry.delete_password()
        .map_err(|e| PublishError::Auth(format!("failed to delete key: {}", e)))
}

// ── Publish Manager ───────────────────────────────────────────────────

pub struct PublishManager {
    pub adapters: HashMap<String, Box<dyn PublishAdapter>>,
    pub publish_logs: Vec<PublishLog>,
    pub platform_formats: HashMap<String, PlatformFormat>,
}

impl PublishManager {
    pub fn new(db: &Database) -> Self {
        let logs = db.load_publish_logs().unwrap_or_default();
        let mut pm = Self {
            adapters: HashMap::new(),
            publish_logs: logs,
            platform_formats: PlatformFormat::defaults(),
        };

        // Register adapters
        pm.adapters.insert("etsy".into(), Box::new(EtsyAdapter));
        pm.adapters.insert("gumroad".into(), Box::new(GumroadAdapter));
        pm.adapters.insert("shopify".into(), Box::new(ShopifyAdapter));
        pm.adapters.insert("payhip".into(), Box::new(PayhipAdapter));

        pm
    }

    /// Load platform formats from a JSON config file (updateable without rebuild)
    pub fn load_format_config(path: &str) -> HashMap<String, PlatformFormat> {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(formats) = serde_json::from_str::<HashMap<String, PlatformFormat>>(&data) {
                return formats;
            }
        }
        PlatformFormat::defaults()
    }

    /// Save formats to JSON (used when creating default config)
    pub fn save_formats_to_file(path: &str) {
        let formats = PlatformFormat::defaults();
        if let Ok(json) = serde_json::to_string_pretty(&formats) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Publish a product to a marketplace
    pub async fn publish(
        &mut self,
        db: &Database,
        product_id: usize,
        product_name: &str,
        platform: &str,
        title: &str,
        description: &str,
        price: f64,
        file_path: Option<&str>,
    ) -> Result<PublishLog, PublishError> {
        let adapter = self.adapters.get(platform)
            .ok_or_else(|| PublishError::Platform(format!("no adapter for {}", platform)))?;

        let api_key = get_api_key(platform)
            .ok_or_else(|| PublishError::Auth(format!("no API key stored for {}", platform)))?;

        // Validate against format config
        if let Some(fmt) = self.platform_formats.get(platform) {
            if title.len() > fmt.max_title_length {
                return Err(PublishError::Validation(format!(
                    "title too long: {} > {} chars", title.len(), fmt.max_title_length
                )));
            }
            if description.len() > fmt.max_description_length {
                return Err(PublishError::Validation(format!(
                    "description too long: {} > {} chars", description.len(), fmt.max_description_length
                )));
            }
        }

        let (listing_id, listing_url) = adapter.list_product(&api_key, title, description, price, file_path).await?;

        let log = PublishLog {
            id: self.publish_logs.len() + 1,
            product_id,
            product_name: product_name.to_string(),
            platform: platform.to_string(),
            listing_url: Some(listing_url),
            listing_id: Some(listing_id),
            status: PublishStatus::Published,
            error_message: None,
            published_at: Utc::now(),
        };

        self.publish_logs.push(log.clone());
        db.save_publish_log(&log).ok();

        Ok(log)
    }

    /// Check if a platform has credentials stored
    pub fn has_credentials(platform: &str) -> bool {
        get_api_key(platform).is_some()
    }

    /// Get publish logs for a specific product
    pub fn logs_for_product(&self, product_id: usize) -> Vec<&PublishLog> {
        self.publish_logs.iter().filter(|l| l.product_id == product_id).collect()
    }
}

// ── Etsy Adapter ──────────────────────────────────────────────────────

pub struct EtsyAdapter;

const ETSY_API_BASE: &str = "https://openapi.etsy.com/v3/application";

#[async_trait::async_trait]
impl PublishAdapter for EtsyAdapter {
    fn platform_name(&self) -> &'static str { "etsy" }

    async fn authenticate(&self, api_key: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("empty API key".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let resp = client
            .get(format!("{}/listings", ETSY_API_BASE))
            .header("x-api-key", api_key)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PublishError::Network("Etsy API timeout".into())
                } else if e.is_connect() {
                    PublishError::Network("Failed to connect to Etsy API".into())
                } else {
                    PublishError::Network(e.to_string())
                }
            })?;

        match resp.status().as_u16() {
            200 | 201 => Ok(true),
            401 | 403 => {
                let text = resp.text().await.unwrap_or_default();
                Err(PublishError::Auth(format!("Etsy auth failed: {}", text)))
            }
            _ => {
                let text = resp.text().await.unwrap_or_default();
                Err(PublishError::Platform(format!("Etsy API error: {}", text)))
            }
        }
    }

    async fn list_product(
        &self,
        api_key: &str,
        title: &str,
        description: &str,
        price: f64,
        _file_path: Option<&str>,
    ) -> Result<(String, String), PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Etsy".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let body = serde_json::json!({
            "quantity": 1,
            "title": title,
            "description": description,
            "price": price,
            "who_made": "i_did",
            "when_made": "made_to_order",
            "taxonomy_id": 1,
            "type": "digital"
        });

        let resp = client
            .post(format!("{}/listings", ETSY_API_BASE))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PublishError::Network("Etsy API timeout".into())
                } else if e.is_connect() {
                    PublishError::Network("Failed to connect to Etsy API".into())
                } else {
                    PublishError::Network(e.to_string())
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(PublishError::Auth(format!("Etsy auth failed: {}", text)));
            }
            return Err(PublishError::Platform(format!("Etsy API error: {} - {}", status, text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Etsy response: {}", e)))?;

        let listing_id = data["listing_id"].to_string();
        // The response contains the numeric listing_id; build the URL
        let listing_url = format!("https://www.etsy.com/listing/{}", listing_id);

        Ok((listing_id, listing_url))
    }

    async fn update_listing(
        &self,
        api_key: &str,
        listing_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        price: Option<f64>,
    ) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Etsy".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let mut body = serde_json::Map::new();
        if let Some(t) = title {
            body.insert("title".into(), serde_json::Value::String(t.to_string()));
        }
        if let Some(d) = description {
            body.insert("description".into(), serde_json::Value::String(d.to_string()));
        }
        if let Some(p) = price {
            body.insert("price".into(), serde_json::Value::Number(
                serde_json::Number::from_f64(p).unwrap_or(serde_json::Number::from(0))
            ));
        }

        let resp = client
            .put(format!("{}/listings/{}", ETSY_API_BASE, listing_id))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(true)
        } else {
            let text = resp.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                Err(PublishError::Auth(format!("Etsy auth failed: {}", text)))
            } else {
                Err(PublishError::Platform(format!("Etsy update error: {} - {}", status, text)))
            }
        }
    }

    async fn check_status(&self, api_key: &str, listing_id: &str) -> Result<String, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Etsy".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let resp = client
            .get(format!("{}/listings/{}", ETSY_API_BASE, listing_id))
            .header("x-api-key", api_key)
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(PublishError::Auth(format!("Etsy auth failed: {}", text)));
            }
            return Err(PublishError::Platform(format!("Etsy status error: {} - {}", status, text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Etsy response: {}", e)))?;

        let state = data["state"].as_str().unwrap_or("unknown").to_string();
        Ok(state)
    }

    async fn remove_listing(&self, api_key: &str, listing_id: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Etsy".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        // Etsy deactivates listings via a PUT to /listings/:id with state=inactive
        let body = serde_json::json!({
            "state": "inactive"
        });

        let resp = client
            .put(format!("{}/listings/{}", ETSY_API_BASE, listing_id))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(true)
        } else {
            let text = resp.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                Err(PublishError::Auth(format!("Etsy auth failed: {}", text)))
            } else {
                Err(PublishError::Platform(format!("Etsy remove error: {} - {}", status, text)))
            }
        }
    }
}

// ── Gumroad Adapter ───────────────────────────────────────────────────

pub struct GumroadAdapter;

const GUMROAD_API_BASE: &str = "https://api.gumroad.com/v2";

#[async_trait::async_trait]
impl PublishAdapter for GumroadAdapter {
    fn platform_name(&self) -> &'static str { "gumroad" }

    async fn authenticate(&self, api_key: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("empty API key".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let resp = client
            .get(format!("{}/products?access_token={}", GUMROAD_API_BASE, api_key))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PublishError::Network("Gumroad API timeout".into())
                } else if e.is_connect() {
                    PublishError::Network("Failed to connect to Gumroad API".into())
                } else {
                    PublishError::Network(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PublishError::Platform(format!("Gumroad API error: {}", text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Gumroad response: {}", e)))?;

        if data["success"] == true {
            Ok(true)
        } else {
            let msg = data["message"].as_str().unwrap_or("unknown error");
            Err(PublishError::Auth(format!("Gumroad auth failed: {}", msg)))
        }
    }

    async fn list_product(
        &self,
        api_key: &str,
        title: &str,
        description: &str,
        price: f64,
        _file_path: Option<&str>,
    ) -> Result<(String, String), PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Gumroad".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let params = [
            ("access_token", api_key.to_string()),
            ("name", title.to_string()),
            ("description", description.to_string()),
            ("price", price.to_string()),
            ("require_shipping", "false".to_string()),
        ];

        let resp = client
            .post(format!("{}/products", GUMROAD_API_BASE))
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PublishError::Network("Gumroad API timeout".into())
                } else if e.is_connect() {
                    PublishError::Network("Failed to connect to Gumroad API".into())
                } else {
                    PublishError::Network(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PublishError::Platform(format!("Gumroad API error: {}", text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Gumroad response: {}", e)))?;

        if data["success"] != true {
            let msg = data["message"].as_str().unwrap_or("unknown error");
            return Err(PublishError::Platform(format!("Gumroad error: {}", msg)));
        }

        let product = &data["product"];
        let listing_id = product["id"].as_str().unwrap_or("").to_string();
        let custom_permalink = product["custom_permalink"].as_str().unwrap_or("");
        let listing_url = format!("https://gumroad.com/l/{}", custom_permalink);

        Ok((listing_id, listing_url))
    }

    async fn update_listing(
        &self,
        api_key: &str,
        listing_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        price: Option<f64>,
    ) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Gumroad".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let mut params = vec![("access_token".to_string(), api_key.to_string())];
        if let Some(t) = title {
            params.push(("name".to_string(), t.to_string()));
        }
        if let Some(d) = description {
            params.push(("description".to_string(), d.to_string()));
        }
        if let Some(p) = price {
            params.push(("price".to_string(), p.to_string()));
        }

        let resp = client
            .put(format!("{}/products/{}", GUMROAD_API_BASE, listing_id))
            .form(&params)
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PublishError::Platform(format!("Gumroad update error: {}", text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Gumroad response: {}", e)))?;

        if data["success"] == true {
            Ok(true)
        } else {
            let msg = data["message"].as_str().unwrap_or("unknown error");
            Err(PublishError::Platform(format!("Gumroad update failed: {}", msg)))
        }
    }

    async fn check_status(&self, api_key: &str, listing_id: &str) -> Result<String, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Gumroad".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let resp = client
            .get(format!("{}/products/{}?access_token={}", GUMROAD_API_BASE, listing_id, api_key))
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PublishError::Platform(format!("Gumroad status error: {}", text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Gumroad response: {}", e)))?;

        if data["success"] == true {
            Ok("active".to_string())
        } else {
            let msg = data["message"].as_str().unwrap_or("unknown error");
            Err(PublishError::Platform(format!("Gumroad status check failed: {}", msg)))
        }
    }

    async fn remove_listing(&self, api_key: &str, listing_id: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("no API key configured for Gumroad".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| PublishError::Network(e.to_string()))?;

        let resp = client
            .delete(format!("{}/products/{}?access_token={}", GUMROAD_API_BASE, listing_id, api_key))
            .send()
            .await
            .map_err(|e| PublishError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PublishError::Platform(format!("Gumroad remove error: {}", text)));
        }

        let data: serde_json::Value = resp.json()
            .await
            .map_err(|e| PublishError::Network(format!("failed to parse Gumroad response: {}", e)))?;

        if data["success"] == true {
            Ok(true)
        } else {
            let msg = data["message"].as_str().unwrap_or("unknown error");
            Err(PublishError::Platform(format!("Gumroad remove failed: {}", msg)))
        }
    }
}

// ── Shopify Adapter (stub) ────────────────────────────────────────────

pub struct ShopifyAdapter;

#[async_trait::async_trait]
impl PublishAdapter for ShopifyAdapter {
    fn platform_name(&self) -> &'static str { "shopify" }

    async fn authenticate(&self, api_key: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("empty API key".into()));
        }
        // TODO: OAuth for Shopify
        Err(PublishError::Platform("Shopify adapter: not yet implemented".into()))
    }

    async fn list_product(
        &self,
        _api_key: &str, _title: &str, _description: &str,
        _price: f64, _file_path: Option<&str>,
    ) -> Result<(String, String), PublishError> {
        Err(PublishError::Platform("Shopify adapter: not yet implemented".into()))
    }

    async fn update_listing(
        &self, _api_key: &str, _listing_id: &str,
        _title: Option<&str>, _description: Option<&str>, _price: Option<f64>,
    ) -> Result<bool, PublishError> {
        Err(PublishError::Platform("Shopify adapter: not yet implemented".into()))
    }

    async fn check_status(&self, _api_key: &str, _listing_id: &str) -> Result<String, PublishError> {
        Err(PublishError::Platform("Shopify adapter: not yet implemented".into()))
    }

    async fn remove_listing(&self, _api_key: &str, _listing_id: &str) -> Result<bool, PublishError> {
        Err(PublishError::Platform("Shopify adapter: not yet implemented".into()))
    }
}

// ── Payhip Adapter (stub) ─────────────────────────────────────────────

pub struct PayhipAdapter;

#[async_trait::async_trait]
impl PublishAdapter for PayhipAdapter {
    fn platform_name(&self) -> &'static str { "payhip" }

    async fn authenticate(&self, api_key: &str) -> Result<bool, PublishError> {
        if api_key.is_empty() {
            return Err(PublishError::Auth("empty API key".into()));
        }
        Err(PublishError::Platform("Payhip adapter: not yet implemented".into()))
    }

    async fn list_product(
        &self, _api_key: &str, _title: &str, _description: &str,
        _price: f64, _file_path: Option<&str>,
    ) -> Result<(String, String), PublishError> {
        Err(PublishError::Platform("Payhip adapter: not yet implemented".into()))
    }

    async fn update_listing(
        &self, _api_key: &str, _listing_id: &str,
        _title: Option<&str>, _description: Option<&str>, _price: Option<f64>,
    ) -> Result<bool, PublishError> {
        Err(PublishError::Platform("Payhip adapter: not yet implemented".into()))
    }

    async fn check_status(&self, _api_key: &str, _listing_id: &str) -> Result<String, PublishError> {
        Err(PublishError::Platform("Payhip adapter: not yet implemented".into()))
    }

    async fn remove_listing(&self, _api_key: &str, _listing_id: &str) -> Result<bool, PublishError> {
        Err(PublishError::Platform("Payhip adapter: not yet implemented".into()))
    }
}
