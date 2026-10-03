//! Admin control panel — license management, feature flags, pricing, platform formats

use rand::Rng;
use serde_json::{json, Value};

/// Admin sections in the control panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSection {
    Features,
    Formats,
    Keys,
    Revocations,
}

/// Admin state — loaded configs and key generation tools
pub struct AdminState {
    pub admin_mode: bool,
    pub feature_tiers: Value,
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
                    "agency": { "name": "Agency", "devices": 20, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","analytics","publishing","bundles","scheduler","presets","client_management","mockup_compositor"] },
                    "enterprise": { "name": "Enterprise", "devices": -1, "features": ["pipeline","ai_generation","templates","market_research","contract_generator","export","analytics","publishing","bundles","scheduler","presets","client_management","mockup_compositor"] }
                }
            });
            let _ = std::fs::write("feature_tiers.json", serde_json::to_string_pretty(&self.feature_tiers).unwrap_or_default());
        }

        // NOTE: no pricing.json any more.
        //
        // The app used to load a price table (and WRITE a default one containing $29/$99/$299
        // per month into the user's working directory on first run). Prices are set on the sales
        // page, not in the software: a licence can be sold one-time or as a subscription, and a
        // price baked into a shipped binary is wrong for whichever customer got the other deal.
        // The app has no business knowing what anyone paid.

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

    /// Generate a licence key using the SAME scheme as activation
    /// (`license_manager::mint_key`), so a key minted here always activates.
    ///
    /// ⚠️ The previous implementation emitted a legacy **5-segment** key
    /// (`DPF-{P|T|A|E}-{R4}-{R4}-{SUM}`) with a single-letter tier and its own
    /// byte-sum checksum. `LicenseManager::activate` requires **4 segments**
    /// (`DPF-<TIER_TOKEN>-<BLOCK>-<CCCC>`) and a check code computed over
    /// `"<TIER_TOKEN>-<BLOCK>"`, so **every key this panel produced was rejected on
    /// activation**. Do not reintroduce a local scheme here: always call `mint_key`,
    /// so the app, this admin panel and `scripts/dpf-mint-license.py` can never disagree.
    /// (`admin_panel_key_passes_the_activation_check` in the tests guards this.)
    pub fn generate_key(&mut self, tier: &str, devices: u32) -> String {
        use crate::license_manager::{mint_key, LicenseTier};

        let t = LicenseTier::from_slug(tier).unwrap_or(LicenseTier::Personal);

        // Unambiguous alphabet (no I/O/0/1) — the same alphabet the check code uses.
        const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::thread_rng();
        let block: String = (0..8)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect();

        let key = mint_key(&t, &block);
        self.generated_key = Some(key.clone());
        self.status_message = format!(
            "Generated {} key — {} seat(s) per activation. Give the key to the customer; \
             they activate it from the 🔑 Licence button.",
            t.display_name(),
            if devices == 0 { t.max_devices() } else { devices as i64 }
        );
        key
    }

    /// Revoke a license key — adds to revoked list and saves
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license_manager::{check_code, LicenseTier};

    /// Regression guard. The Admin panel used to mint legacy **5-segment** keys
    /// (`DPF-{P|T|A|E}-{R4}-{R4}-{SUM}`) with its own byte-sum checksum, while
    /// `LicenseManager::activate` requires **4 segments** and a check code over
    /// `"<TIER_TOKEN>-<BLOCK>"`. So every key this panel produced was **rejected when the
    /// customer tried to activate it**. Any key minted here must satisfy the activation contract.
    #[test]
    fn admin_panel_key_passes_the_activation_check() {
        for tier in ["personal", "team", "agency", "enterprise"] {
            let mut a = AdminState::default();
            let key = a.generate_key(tier, 1);

            let parts: Vec<&str> = key.split('-').collect();
            assert_eq!(
                parts.len(),
                4,
                "the admin panel must emit the 4-segment activation format, got {key}"
            );
            assert_eq!(parts[0], "DPF", "keys must start with DPF: {key}");

            let body = format!("{}-{}", parts[1], parts[2]);
            assert!(
                check_code(&body).eq_ignore_ascii_case(parts[3]),
                "admin-minted key {key} failed its own check code — it would be refused on activation"
            );

            assert_eq!(
                LicenseTier::from_slug(parts[1]),
                LicenseTier::from_slug(tier),
                "the tier token must round-trip through activation for '{tier}' (key: {key})"
            );
        }
    }
}
