//! License management
//!
//! ## OFFLINE ACTIVATION — read this before "improving" the validation below
//!
//! This product has **no licence server**, so activation is validated entirely **offline**: a key
//! encodes its tier and carries a check code derived from a compile-time salt, and it is refused if
//! it appears in `revoked_keys.json`.
//!
//! What this DOES do: enforce the tier model honestly inside the app — a Personal (free) user cannot
//! reach a paid module through the UI, because every tab is gated on the active tier's feature list.
//!
//! What this does NOT do: it is **not tamper-proof**. Anyone who can patch the binary can bypass it.
//! Real enforcement needs one of:
//!   * online activation against a server (the only approach that also supports revocation at scale), or
//!   * code signing + notarisation so a patched binary fails to launch.
//! Treat the check below as "honest gating for honest users", not as DRM.

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Compile-time salt for the key check code. Mint keys with the same value —
/// see `scripts/dpf-mint-license.py`.
pub const SALT: &str = "dpf-1.4.2-license";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    Personal,
    Team,
    Agency,
    Enterprise,
}

impl LicenseTier {
    /// Lower-case slug as used in `feature_tiers.json` and in key text.
    pub fn slug(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "personal",
            LicenseTier::Team => "team",
            LicenseTier::Agency => "agency",
            LicenseTier::Enterprise => "enterprise",
        }
    }

    /// The UPPER-CASE token that appears as segment 2 of a key.
    pub fn key_token(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "PERSONAL",
            LicenseTier::Team => "TEAM",
            LicenseTier::Agency => "AGENCY",
            LicenseTier::Enterprise => "ENTERPRISE",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "personal" => Some(LicenseTier::Personal),
            "team" => Some(LicenseTier::Team),
            "agency" => Some(LicenseTier::Agency),
            "enterprise" => Some(LicenseTier::Enterprise),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "Personal",
            LicenseTier::Team => "Team",
            LicenseTier::Agency => "Agency",
            LicenseTier::Enterprise => "Enterprise",
        }
    }

    /// Seat limit. `-1` means unlimited (Enterprise).
    pub fn max_devices(&self) -> i64 {
        match self {
            LicenseTier::Personal => 1,
            LicenseTier::Team => 5,
            LicenseTier::Agency => 20,
            LicenseTier::Enterprise => -1,
        }
    }

    pub fn price_label(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "Free",
            LicenseTier::Team => "$29/month",
            LicenseTier::Agency => "$99/month",
            LicenseTier::Enterprise => "$299/month",
        }
    }

    pub fn all() -> [LicenseTier; 4] {
        [
            LicenseTier::Personal,
            LicenseTier::Team,
            LicenseTier::Agency,
            LicenseTier::Enterprise,
        ]
    }

    /// Fallback feature list, used only if `feature_tiers.json` cannot be read.
    /// Kept in sync with that file and with the published pricing table.
    pub fn builtin_features(&self) -> &'static [&'static str] {
        match self {
            LicenseTier::Personal => &[
                "pipeline",
                "ai_generation",
                "templates",
                "market_research",
                "contract_generator",
                "export",
                "presets",
                "variants",
            ],
            LicenseTier::Team => &[
                "pipeline",
                "ai_generation",
                "templates",
                "market_research",
                "contract_generator",
                "export",
                "presets",
                "variants",
                "analytics",
                "publishing",
                "bundles",
                "scheduler",
                "adverts",
                "qc",
                "assets",
                "webhooks",
                "mockup_compositor",
                "logo_generator",
                "vector_generator",
            ],
            LicenseTier::Agency => &[
                "pipeline",
                "ai_generation",
                "templates",
                "market_research",
                "contract_generator",
                "export",
                "presets",
                "variants",
                "analytics",
                "publishing",
                "bundles",
                "scheduler",
                "adverts",
                "qc",
                "assets",
                "webhooks",
                "mockup_compositor",
                "logo_generator",
                "vector_generator",
                "whitelabel",
                "client_management",
                "compliance",
                "custom_integrations",
            ],
            LicenseTier::Enterprise => &[
                "pipeline",
                "ai_generation",
                "templates",
                "market_research",
                "contract_generator",
                "export",
                "presets",
                "variants",
                "analytics",
                "publishing",
                "bundles",
                "scheduler",
                "adverts",
                "qc",
                "assets",
                "webhooks",
                "mockup_compositor",
                "logo_generator",
                "vector_generator",
                "whitelabel",
                "client_management",
                "compliance",
                "custom_integrations",
                "api_access",
                "admin_panel",
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub tier: LicenseTier,
    pub max_devices: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: LicenseStatus,
    pub activated_devices: Vec<Device>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub activated_at: DateTime<Utc>,
}

/// FNV-1a over `body + SALT`, rendered as 4 chars from an unambiguous alphabet
/// (no I/O/0/1). `scripts/dpf-mint-license.py` implements the identical algorithm.
pub fn check_code(body: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let salted = format!("{}{}", body, SALT);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in salted.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut out = String::with_capacity(4);
    for _ in 0..4 {
        out.push(ALPHABET[(h % 32) as usize] as char);
        h /= 32;
    }
    out
}

/// Mint a key for a tier. `block` is the opaque middle segment (any A-Z0-9 text).
/// Kept here so the app and the minting script share one definition.
pub fn mint_key(tier: &LicenseTier, block: &str) -> String {
    let block = block.trim().to_ascii_uppercase();
    let body = format!("{}-{}", tier.key_token(), block);
    format!("DPF-{}-{}", body, check_code(&body))
}

/// Read the tier feature list from `feature_tiers.json`, falling back to the built-in table.
fn features_from_json(tier: &LicenseTier) -> Option<Vec<String>> {
    let raw = std::fs::read_to_string("feature_tiers.json").ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let list = v.get("tiers")?.get(tier.slug())?.get("features")?.as_array()?;
    Some(
        list.iter()
            .filter_map(|s| s.as_str().map(|s| s.to_string()))
            .collect(),
    )
}

/// Feature slugs enabled for a tier. `feature_tiers.json` is authoritative; the
/// built-in list is the safety net if the file is missing or malformed.
pub fn features_for_tier(tier: &LicenseTier) -> Vec<String> {
    features_from_json(tier)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| tier.builtin_features().iter().map(|s| s.to_string()).collect())
}

/// Keys listed in `revoked_keys.json` are refused at activation.
fn is_revoked(key: &str) -> bool {
    let Ok(raw) = std::fs::read_to_string("revoked_keys.json") else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    v.get("revoked_keys")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str())
                .any(|s| s.trim().eq_ignore_ascii_case(key))
        })
        .unwrap_or(false)
}

pub struct LicenseManager {
    db: Arc<Database>,
    current_license: Option<License>,
}

impl LicenseManager {
    pub fn new(db: &Arc<Database>) -> Self {
        // Load a previously activated licence, so activation survives a restart.
        let current_license = db
            .load_active_license()
            .ok()
            .flatten()
            .filter(|l| l.status == LicenseStatus::Active);

        Self {
            db: db.clone(),
            current_license,
        }
    }

    pub fn is_licensed(&self) -> bool {
        self.current_license
            .as_ref()
            .map(|l| l.status == LicenseStatus::Active)
            .unwrap_or(false)
    }

    pub fn current_license(&self) -> Option<&License> {
        self.current_license.as_ref()
    }

    /// The active tier. An unlicensed install is on the free Personal tier, which is
    /// what makes gating work without a licence present.
    pub fn tier(&self) -> LicenseTier {
        self.current_license
            .as_ref()
            .filter(|l| l.status == LicenseStatus::Active)
            .map(|l| l.tier.clone())
            .unwrap_or(LicenseTier::Personal)
    }

    pub fn tier_name(&self) -> String {
        if self.is_licensed() {
            format!("{} ({})", self.tier().display_name(), self.tier().price_label())
        } else {
            "Personal (Free)".to_string()
        }
    }

    /// Is `feature` (a slug from feature_tiers.json) available on the active tier?
    pub fn has_feature(&self, feature: &str) -> bool {
        features_for_tier(&self.tier()).iter().any(|f| f == feature)
    }

    /// Features the active tier does NOT include — used for the upgrade hints in the UI.
    pub fn missing_features(&self) -> Vec<String> {
        let have = features_for_tier(&self.tier());
        features_for_tier(&LicenseTier::Enterprise)
            .into_iter()
            .filter(|f| !have.contains(f))
            .collect()
    }

    /// Minimal tier that unlocks `feature`, for the "upgrade to X" message.
    pub fn required_tier_for(feature: &str) -> Option<LicenseTier> {
        LicenseTier::all()
            .into_iter()
            .find(|t| features_for_tier(t).iter().any(|f| f == feature))
    }

    /// Activate a key offline. Validates format, tier, check code and the revocation list,
    /// then persists so it survives a restart.
    pub fn activate(&mut self, raw: &str) -> Result<(), String> {
        let key = raw.trim().to_ascii_uppercase();
        if key.is_empty() {
            return Err("Please enter a licence key.".to_string());
        }

        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() != 4 || parts[0] != "DPF" {
            return Err(
                "That does not look like a key. The format is DPF-TIER-XXXXXXXX-CCCC.".to_string(),
            );
        }

        let tier = LicenseTier::from_slug(parts[1]).ok_or_else(|| {
            format!(
                "'{}' is not a tier we recognise. Expected PERSONAL, TEAM, AGENCY or ENTERPRISE.",
                parts[1]
            )
        })?;

        let body = format!("{}-{}", parts[1], parts[2]);
        if !check_code(&body).eq_ignore_ascii_case(parts[3]) {
            return Err("That key's check code is invalid — please re-check it.".to_string());
        }

        if is_revoked(&key) {
            return Err("That key has been revoked and can no longer be activated.".to_string());
        }

        let license = License {
            key: key.clone(),
            tier: tier.clone(),
            max_devices: tier.max_devices(),
            created_at: Utc::now(),
            expires_at: None,
            status: LicenseStatus::Active,
            activated_devices: Vec::new(),
            metadata: None,
        };

        self.db
            .save_license(&license)
            .map_err(|e| format!("Could not save the licence: {e}"))?;

        self.current_license = Some(license);
        Ok(())
    }

    pub fn deactivate(&mut self) {
        if let Some(l) = self.current_license.take() {
            let _ = self.db.delete_license(&l.key);
        }
        self.current_license = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-implementation pin. `scripts/dpf-mint-license.py` implements the same
    /// `check_code`; if these two strings ever stop matching, the tool and the app have
    /// drifted and EVERY minted key is dead on arrival.
    #[test]
    fn mint_matches_python() {
        assert_eq!(mint_key(&LicenseTier::Team, "AB12CD34"), "DPF-TEAM-AB12CD34-P9SU");
        assert_eq!(mint_key(&LicenseTier::Agency, "ZZ99YY88"), "DPF-AGENCY-ZZ99YY88-R5QY");
    }

    /// A key must round-trip through its own check code.
    #[test]
    fn check_code_validates_its_own_keys() {
        for t in LicenseTier::all() {
            let key = mint_key(&t, "TESTTEST");
            let parts: Vec<&str> = key.split('-').collect();
            assert_eq!(parts.len(), 4);
            assert_eq!(parts[0], "DPF");
            let body = format!("{}-{}", parts[1], parts[2]);
            assert!(check_code(&body).eq_ignore_ascii_case(parts[3]));
        }
    }

    /// A single wrong character must be rejected.
    #[test]
    fn tampered_key_is_rejected() {
        let key = mint_key(&LicenseTier::Team, "AB12CD34");
        let mut tampered = key.clone();
        tampered.pop();
        tampered.push(if key.ends_with('X') { 'Y' } else { 'X' });
        let parts: Vec<&str> = tampered.split('-').collect();
        let body = format!("{}-{}", parts[1], parts[2]);
        assert!(!check_code(&body).eq_ignore_ascii_case(parts[3]));
    }

    /// Each tier must be a strict superset of the one below it, or gating is incoherent.
    #[test]
    fn tiers_are_supersets() {
        let p = features_for_tier(&LicenseTier::Personal);
        let t = features_for_tier(&LicenseTier::Team);
        let a = features_for_tier(&LicenseTier::Agency);
        let e = features_for_tier(&LicenseTier::Enterprise);
        for f in &p {
            assert!(t.contains(f), "Team is missing Personal feature {f}");
        }
        for f in &t {
            assert!(a.contains(f), "Agency is missing Team feature {f}");
        }
        for f in &a {
            assert!(e.contains(f), "Enterprise is missing Agency feature {f}");
        }
    }

    /// The business rule this whole module exists for: the free tier must NOT include any
    /// paid module. If this ever fails, the product is giving away what it sells.
    #[test]
    fn free_tier_cannot_reach_paid_modules() {
        let p = features_for_tier(&LicenseTier::Personal);
        for f in [
            "analytics",
            "publishing",
            "bundles",
            "scheduler",
            "adverts",
            "qc",
            "assets",
            "webhooks",
            "whitelabel",
            "client_management",
            "compliance",
            "api_access",
            "admin_panel",
        ] {
            assert!(
                !p.contains(&f.to_string()),
                "PAID MODULE LEAK: the free tier must not include '{f}'"
            );
        }
    }
}
