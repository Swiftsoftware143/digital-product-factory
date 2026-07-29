//! Admin control panel â€” license management, feature flags, pricing, platform formats

use rand::Rng;
use serde_json::{json, Value};

/// Admin sections in the control panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSection {
    Features,
    Pricing,
    Formats,
    Keys,
    Revocations,
}

/// Admin state â€” loaded configs and key generation tools
pub struct AdminState {
    pub admin_mode: bool,
    pub feature_tiers: Value,
    pub pricing_data: Value,
    pub platform_formats: Value,
    pub revoked_keys: Vec<String>,
    pub generate_key_input: String,
    pub generated_key: Option<String>,
    pub status_message: String,
    pub active_section: AdminSection,
}

impl Default for AdminState {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminState {
    /// Create a new AdminState, loading config files from disk
    pub fn new() -> Self {
        let mut state = Self {
            admin_mode: false,
            feature_tiers: Value::Null,
            pricing_data: Value::Null,
            platform_formats: Value::Null,
            revoked_keys: Vec::new(),
            generate_key_input: String::new(),
            generated_key: None,
            status_message: String::new(),
            active_section: AdminSection::Features,
        };
        state.load_configs();
        state
    }

    /// Load all config JSONs from the app directory, creating defaults if missing
    pub fn load_configs(&mut self) {
        let dir = std::env::current_dir().unwrap_or_default();

        // feature_tiers.json
        let path = dir.join("feature_tiers.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str(&s) {
                    self.feature_tiers = v;
                }
            }
        }
        if self.feature_tiers == Value::Null {
            self.feature_tiers = json!({
                "tiers": {
                    "personal": { "name": "Personal", "devices": 1, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","mockup_compositor"] },
                    "team": { "name": "Team", "devices": 5, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","analytics","publishing","bundles","scheduler","presets","mockup_compositor"] },
                    "agency": { "name": "Agency", "devices": 20, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","analytics","publishing","bundles","scheduler","presets","whitelabel","client_management","mockup_compositor"] },
                    "enterprise": { "name": "Enterprise", "devices": -1, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","analytics","publishing","bundles","scheduler","presets","whitelabel","client_management","custom_integrations","api_access","mockup_compositor"] }
                }
            });
            let _ = std::fs::write("feature_tiers.json", serde_json::to_string_pretty(&self.feature_tiers).unwrap_or_default());
        }

        // pricing.json
        let path = dir.join("pricing.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str(&s) {
                    self.pricing_data = v;
                }
            }
        }
        if self.pricing_data == Value::Null {
            self.pricing_data = json!({
                "personal": { "price": 0, "period": "free" },
                "team": { "price": 29, "period": "month" },
                "agency": { "price": 99, "period": "month" },
                "enterprise": { "price": 299, "period": "month" }
            });
            let _ = std::fs::write("pricing.json", serde_json::to_string_pretty(&self.pricing_data).unwrap_or_default());
        }

        // platform_formats.json
        let path = dir.join("platform_formats.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str(&s) {
                    self.platform_formats = v;
                }
            }
        }
        if self.platform_formats == Value::Null {
            self.platform_formats = json!({
                "etsy": {
                    "name": "Etsy",
                    "thumbnail_width": 3000,
                    "thumbnail_height": 3000,
                    "max_file_size_mb": 20,
                    "max_tags": 13,
                    "max_title_length": 140,
                    "max_description_length": 5000,
                    "allowed_formats": ["pdf", "zip", "png", "jpg"],
                    "digital_download": true
                },
                "gumroad": {
                    "name": "Gumroad",
                    "thumbnail_width": 1280,
                    "thumbnail_height": 720,
                    "max_file_size_mb": 50,
                    "max_tags": 0,
                    "max_title_length": 255,
                    "max_description_length": 10000,
                    "allowed_formats": ["pdf", "zip", "epub", "mp4"],
                    "digital_download": true
                },
                "shopify": {
                    "name": "Shopify",
                    "thumbnail_width": 2048,
                    "thumbnail_height": 2048,
                    "max_file_size_mb": 20,
                    "max_tags": 0,
                    "max_title_length": 255,
                    "max_description_length": 5000,
                    "allowed_formats": ["pdf", "zip", "jpg", "png"],
                    "digital_download": true
                },
                "payhip": {
                    "name": "Payhip",
                    "thumbnail_width": 1200,
                    "thumbnail_height": 1200,
                    "max_file_size_mb": 25,
                    "max_tags": 0,
                    "max_title_length": 150,
                    "max_description_length": 4000,
                    "allowed_formats": ["pdf", "zip", "jpg", "png"],
                    "digital_download": true
                }
            });
            let _ = std::fs::write("platform_formats.json", serde_json::to_string_pretty(&self.platform_formats).unwrap_or_default());
        }

        // revoked_keys.json
        let path = dir.join("revoked_keys.json");
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str::<Value>(&s) {
                    if let Some(keys) = v.get("revoked_keys").and_then(|k| k.as_array()) {
                        self.revoked_keys = keys.iter().filter_map(|k| k.as_str().map(String::from)).collect();
                    }
                }
            }
        }
        if self.revoked_keys.is_empty() {
            let keys_val = json!({ "revoked_keys": [] });
            let _ = std::fs::write("revoked_keys.json", serde_json::to_string_pretty(&keys_val).unwrap_or_default());
        }
    }

    /// Save a config back to its file by name: "features", "pricing", "formats", or "revocations"
    pub fn save_config(&mut self, name: &str) {
        let dir = std::env::current_dir().unwrap_or_default();
        let result = match name {
            "features" => {
                let path = dir.join("feature_tiers.json");
                std::fs::write(&path, serde_json::to_string_pretty(&self.feature_tiers).unwrap_or_default())
            }
            "pricing" => {
                let path = dir.join("pricing.json");
                std::fs::write(&path, serde_json::to_string_pretty(&self.pricing_data).unwrap_or_default())
            }
            "formats" => {
                let path = dir.join("platform_formats.json");
                std::fs::write(&path, serde_json::to_string_pretty(&self.platform_formats).unwrap_or_default())
            }
            "revocations" => {
                let path = dir.join("revoked_keys.json");
                let data = json!({ "revoked_keys": self.revoked_keys });
                std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default())
            }
            _ => return,
        };

        match result {
            Ok(_) => self.status_message = format!("Saved {}", name),
            Err(e) => self.status_message = format!("Error saving {}: {}", name, e),
        }
    }

    /// Generate a license key in format: DPF-{TIER_CODE}-{RAND4}-{RAND4}-{SUM}
    pub fn generate_key(&mut self, tier: &str, devices: u32) -> String {
        let tier_code = match tier.to_lowercase().as_str() {
            "personal" | "p" => "P",
            "team" | "t" => "T",
            "agency" | "a" => "A",
            "enterprise" | "e" => "E",
            _ => "X",
        };

        let mut rng = rand::thread_rng();
        let rand4_1: String = (0..4).map(|_| {
            let idx = rng.gen_range(0..36);
            std::char::from_digit(idx, 36).unwrap().to_ascii_uppercase()
        }).collect();
        let rand4_2: String = (0..4).map(|_| {
            let idx = rng.gen_range(0..36);
            std::char::from_digit(idx, 36).unwrap().to_ascii_uppercase()
        }).collect();

        let base = format!("DPF-{}-{}-{}", tier_code, rand4_1, rand4_2);
        let checksum: u32 = base.bytes().map(|b| b as u32).sum();
        let sum_str = format!("{:02X}", checksum % 256);
        let key = format!("{}-{}", base, sum_str);

        self.generated_key = Some(key.clone());
        self.status_message = format!("Generated key for tier '{}' ({} devices)", tier, devices);
        key
    }

    /// Revoke a license key â€” adds to revoked list and saves
    pub fn revoke_key(&mut self, key: &str) {
        let trimmed = key.trim().to_string();
        if trimmed.is_empty() {
            self.status_message = "Cannot revoke an empty key".to_string();
            return;
        }
        if self.revoked_keys.contains(&trimmed) {
            self.status_message = format!("Key already revoked: {}", trimmed);
            return;
        }
        self.revoked_keys.push(trimmed.clone());
        self.save_config("revocations");
        self.status_message = format!("Revoked key: {}", trimmed);
    }

    /// Return count of feature tiers
    pub fn tier_count(&self) -> usize {
        self.feature_tiers
            .get("tiers")
            .and_then(|t| t.as_object())
            .map(|o| o.len())
            .unwrap_or(0)
    }

    /// Return count of platform formats
    pub fn format_count(&self) -> usize {
        self.platform_formats
            .as_object()
            .map(|o| o.len())
            .unwrap_or(0)
    }
}
