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
    Pro,
    Enterprise,
    /// The owner's own class. Not sold, not listed, not offered as an upgrade: it exists so the
    /// person who runs this business can reach the Admin panel and a customer never can.
    Owner,
}

impl LicenseTier {
    /// Lower-case slug as used in `feature_tiers.json` and in key text.
    pub fn slug(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "personal",
            LicenseTier::Team => "team",
            LicenseTier::Pro => "pro",
            LicenseTier::Enterprise => "enterprise",
            // `feature_tiers.json` has no "owner" section on purpose; owner features come from
            // `builtin_features` so there is no file a customer could edit to grant themselves it.
            LicenseTier::Owner => "owner",
        }
    }

    /// The UPPER-CASE token that appears as segment 2 of a key.
    pub fn key_token(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "PERSONAL",
            LicenseTier::Team => "TEAM",
            LicenseTier::Pro => "PRO",
            LicenseTier::Enterprise => "ENTERPRISE",
            LicenseTier::Owner => "OWNER",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "personal" => Some(LicenseTier::Personal),
            "team" => Some(LicenseTier::Team),
            "pro" => Some(LicenseTier::Pro),
            "enterprise" => Some(LicenseTier::Enterprise),
            "owner" => Some(LicenseTier::Owner),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LicenseTier::Personal => "Personal",
            LicenseTier::Team => "Team",
            LicenseTier::Pro => "Pro",
            LicenseTier::Enterprise => "Enterprise",
            LicenseTier::Owner => "Owner",
        }
    }

    /// Seat limit. `-1` means unlimited (Enterprise and the owner).
    pub fn max_devices(&self) -> i64 {
        match self {
            LicenseTier::Personal => 1,
            LicenseTier::Team => 5,
            LicenseTier::Pro => 20,
            LicenseTier::Enterprise => -1,
            LicenseTier::Owner => -1,
        }
    }

    // NOTE: there is deliberately no price label here any more.
    //
    // The app used to carry "$29/month", "$99/month", "$299/month" and showed them in the
    // sidebar and in licence hints. David sells through a sales page and can offer a licence
    // either as a one-time payment or as a subscription (beta testers get a one-off), so any
    // price baked into the software is wrong for some customer and ages badly the moment
    // pricing changes. The software's job is to say WHICH PLAN someone holds, not what they
    // paid for it.

    pub fn all() -> [LicenseTier; 4] {
        [
            LicenseTier::Personal,
            LicenseTier::Team,
            LicenseTier::Pro,
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
            LicenseTier::Pro => &[
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
                "client_management",
                "compliance",
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
                "client_management",
                "compliance",
            ],
            // The owner holds everything, including the Admin panel. Note `admin_panel` is the ONLY
            // feature that appears here and not in Enterprise: reaching it is an ownership fact, not
            // something that can be bought, so it is deliberately absent from the sold tiers.
            LicenseTier::Owner => &[
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
                "client_management",
                "compliance",
                "admin_panel",
            ],
        }
    }
    /// The next tier up from this one, if there is one.
    ///
    /// Used for the upgrade OFFER. The lowest tier that unlocks a given feature is the wrong thing
    /// to show a Personal user looking at a Pro tab — they would be told "upgrade to Pro"
    /// as if it were the only option. Offering the NEXT step is both cheaper for them and truer.
    pub fn next_up(&self) -> Option<LicenseTier> {
        let all = LicenseTier::all();
        let idx = all.iter().position(|t| t == self)?;
        all.get(idx + 1).cloned()
    }

    /// How much of the app this tier is missing, phrased for a human.
    ///
    /// Counts modules, never money. "What it costs" is answered on the sales page.
    pub fn locked_summary(&self) -> String {
        let mine = features_for_tier(self).len();
        let top = LicenseTier::all()
            .into_iter()
            .map(|t| features_for_tier(&t).len())
            .max()
            .unwrap_or(mine);
        let missing = top.saturating_sub(mine);
        match missing {
            0 => "Every module".to_string(),
            1 => "1 module".to_string(),
            n => format!("{n} modules"),
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
/// The keys listed in `revoked_keys.json`. Exposed so callers can pass the list to
/// `LicenseManager::validate_key` explicitly (which keeps that function pure and testable).
pub fn revoked_keys() -> Vec<String> {
    let Ok(raw) = std::fs::read_to_string("revoked_keys.json") else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    v.get("revoked_keys")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
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

    /// Is this the OWNER's own key?
    ///
    /// This — not a tier — is what unlocks the Admin panel. A customer cannot buy it, and it is
    /// deliberately checked separately from `has_feature` so that no future edit to a tier table
    /// can hand the Admin panel to a paying customer.
    pub fn is_owner(&self) -> bool {
        self.is_licensed() && self.tier() == LicenseTier::Owner
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
            // Plan name only — never a price. Pricing lives on the sales page.
            self.tier().display_name().to_string()
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

    /// The tier to OFFER this user, or None if they already hold everything.
    ///
    /// Returns the next tier up rather than the cheapest tier that unlocks some particular locked
    /// feature: a Personal user clicking an Enterprise tab should be offered Team, not told that
    /// Enterprise is their only option. Returns None at the top tier so a fully-licensed customer
    /// is never shown an upsell.
    pub fn required_tier_for_missing(&self) -> Option<LicenseTier> {
        let current = self.tier();
        if self.missing_features().is_empty() {
            return None; // holds everything — no offer to make
        }
        current.next_up()
    }

    /// Minimal tier that unlocks `feature`, for the "upgrade to X" message.
    pub fn required_tier_for(feature: &str) -> Option<LicenseTier> {
        LicenseTier::all()
            .into_iter()
            .find(|t| features_for_tier(t).iter().any(|f| f == feature))
    }

    /// Validate a licence key WITHOUT touching the database.
    ///
    /// `activate()` calls this and the tests call it directly, so the tests exercise the **real**
    /// validation path rather than a copy of it. `revoked` is passed in to keep this pure.
    pub fn validate_key(raw: &str, revoked: &[String]) -> Result<LicenseTier, String> {
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
                "'{}' is not a tier we recognise. Expected PERSONAL, TEAM, PRO or ENTERPRISE.",
                parts[1]
            )
        })?;

        let body = format!("{}-{}", parts[1], parts[2]);
        if !check_code(&body).eq_ignore_ascii_case(parts[3]) {
            return Err("That key's check code is invalid — please re-check it.".to_string());
        }

        if revoked.iter().any(|r| r.trim().eq_ignore_ascii_case(&key)) {
            return Err("That key has been revoked and can no longer be activated.".to_string());
        }

        Ok(tier)
    }

    pub fn activate(&mut self, raw: &str) -> Result<(), String> {
        // Activate a key offline: validate format, tier, check code and the revocation list,
        // then persist so it survives a restart.
        let key = raw.trim().to_ascii_uppercase();
        let revoked = revoked_keys();
        let tier = Self::validate_key(raw, &revoked)?;

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
        assert_eq!(mint_key(&LicenseTier::Pro, "ZZ99YY88"), "DPF-PRO-ZZ99YY88-TW78");
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
        let pr = features_for_tier(&LicenseTier::Pro);
        let e = features_for_tier(&LicenseTier::Enterprise);
        for f in &p {
            assert!(t.contains(f), "Team is missing Personal feature {f}");
        }
        for f in &t {
            assert!(pr.contains(f), "Pro is missing Team feature {f}");
        }
        for f in &pr {
            assert!(e.contains(f), "Enterprise is missing Pro feature {f}");
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
            "client_management",
            "compliance",
            "admin_panel",
        ] {
            assert!(
                !p.contains(&f.to_string()),
                "PAID MODULE LEAK: the free tier must not include '{f}'"
            );
        }
    }

    /// THE HEADLINE TEST: keys produced by the generator the owner will actually run must pass the
    /// app's **real** activation validation.
    ///
    /// Every literal below is verbatim output of the live tool:
    ///     python3 /opt/swift/scripts/dpf-mint-license.py <TIER> AB12CD34
    ///     python3 /opt/swift/scripts/dpf-mint-license.py --batch TEAM 3
    /// If the minting tool and the app ever drift apart, this fails HERE — not in front of a
    /// paying customer who cannot activate what they paid for.
    #[test]
    fn python_minted_keys_validate() {
        let cases = [
            ("DPF-PERSONAL-AB12CD34-225Y", LicenseTier::Personal),
            ("DPF-TEAM-AB12CD34-P9SU", LicenseTier::Team),
            ("DPF-PRO-AB12CD34-KYZ3", LicenseTier::Pro),
            ("DPF-ENTERPRISE-AB12CD34-HBV3", LicenseTier::Enterprise),
            // from a real `--batch TEAM 3` run
            ("DPF-TEAM-V8580S98-W8FU", LicenseTier::Team),
            ("DPF-TEAM-HGJZR43X-T6MC", LicenseTier::Team),
            ("DPF-TEAM-N1QQP0X1-KQ53", LicenseTier::Team),
        ];

        for (key, expected) in cases {
            let got = LicenseManager::validate_key(key, &[])
                .unwrap_or_else(|e| panic!("minted key {key} was REFUSED on activation: {e}"));
            assert_eq!(got, expected, "key {key} resolved to the wrong tier");
        }
    }

    /// A key minted by the LIVE control centre must activate.
    ///
    /// `dpf.swiftsoftware.net` is a separate service holding its own copy of the minting
    /// algorithm. That makes this the cross-system contract, and it is the only one that
    /// matters commercially: if the panel mints something this app rejects, the customer has
    /// paid for software they cannot unlock — and nothing in either codebase would notice,
    /// because each side's tests only prove it agrees with itself.
    ///
    /// The key below was minted by the running service during a smoke test on 2026-10-03.
    #[test]
    fn control_centre_minted_keys_validate() {
        let key = "DPF-TEAM-SMK0TEST-64YJ";
        let tier = LicenseManager::validate_key(key, &[])
            .unwrap_or_else(|e| panic!("the CONTROL CENTRE's key was refused: {e}"));
        assert_eq!(tier, LicenseTier::Team);

        // And the same key must be reproducible from the app's own minting code, proving both
        // implementations are computing the same thing rather than coincidentally agreeing.
        assert_eq!(mint_key(&LicenseTier::Team, "SMK0TEST"), key);
    }

    /// Customers paste messily. Stray whitespace and lower case must both still activate,
    /// because a rejected key here looks to the customer like a key that was never delivered.
    #[test]
    fn real_world_pasting_is_accepted() {
        for k in [
            "  DPF-TEAM-AB12CD34-P9SU  ",
            "DPF-TEAM-AB12CD34-P9SU\n",
            "dpf-team-ab12cd34-p9su",
            "Dpf-Team-Ab12Cd34-P9su",
        ] {
            assert!(
                LicenseManager::validate_key(k, &[]).is_ok(),
                "should have accepted {k:?}"
            );
        }
    }

    /// ...and every way a key really gets damaged must be refused, each with a message that
    /// tells the customer what to do rather than just failing.
    #[test]
    fn damaged_keys_are_refused_with_a_reason() {
        // One character changed in the block — the check code no longer matches.
        let err = LicenseManager::validate_key("DPF-TEAM-AB12CD35-P9SU", &[]).unwrap_err();
        assert!(err.contains("check code"), "unhelpful message: {err}");

        // A tier we do not sell. (PRO is a real tier now — this must be a token that is not one.)
        let err = LicenseManager::validate_key("DPF-ULTRA-AB12CD34-P9SU", &[]).unwrap_err();
        assert!(err.contains("not a tier"), "unhelpful message: {err}");

        // Truncated, as if an email client cut the line.
        let err = LicenseManager::validate_key("DPF-TEAM-AB12CD34", &[]).unwrap_err();
        assert!(err.contains("format"), "unhelpful message: {err}");

        // Nothing entered at all.
        assert!(LicenseManager::validate_key("   ", &[]).is_err());

        // Revoked keys must be refused even though their check code is perfect.
        let err = LicenseManager::validate_key(
            "DPF-TEAM-AB12CD34-P9SU",
            &["DPF-TEAM-AB12CD34-P9SU".to_string()],
        )
        .unwrap_err();
        assert!(err.contains("revoked"), "unhelpful message: {err}");
    }

    /// A revoked list must be matched case/whitespace insensitively, or a revocation could be
    /// trivially bypassed by re-typing the key differently.
    #[test]
    fn revocation_cannot_be_bypassed_by_formatting() {
        let revoked = vec!["  dpf-team-ab12cd34-p9su  ".to_string()];
        assert!(LicenseManager::validate_key("DPF-TEAM-AB12CD34-P9SU", &revoked).is_err());
    }

    /// The shipped tiers file must carry the UPGRADE GATES and nothing about money.
    ///
    /// A licence can be sold one-time or as a subscription (beta testers get a one-off), so any
    /// price compiled into the software is wrong for whichever customer got the other deal — and
    /// it ages badly the moment a price changes on the sales page. The app's job is to say WHICH
    /// plan you hold, never what you paid.
    ///
    /// This regression test exists because `price`/`period` fields WERE present in
    /// feature_tiers.json (0 / 29 / 99 / 299 per month). They were inert — no code read them — but
    /// inert wrong data gets read eventually.
    #[test]
    fn shipped_tiers_carry_gates_not_prices() {
        let raw = std::fs::read_to_string("feature_tiers.json")
            .expect("feature_tiers.json must ship beside the binary");

        let v: serde_json::Value = serde_json::from_str(&raw).expect("tiers file must be valid JSON");
        let tiers = v.get("tiers").expect("tiers object missing");

        for (slug, tier) in tiers.as_object().expect("tiers must be an object") {
            for forbidden in ["price", "period", "cost", "amount", "monthly", "yearly"] {
                assert!(
                    tier.get(forbidden).is_none(),
                    "tier '{slug}' carries a '{forbidden}' field — pricing belongs on the sales page, not in the software"
                );
            }
            // ...and the upgrade gates must still be present, or we broke gating instead.
            assert!(
                tier.get("features").and_then(|f| f.as_array()).map(|a| !a.is_empty()).unwrap_or(false),
                "tier '{slug}' lost its feature list — that is the upgrade gate"
            );
            assert!(
                tier.get("devices").is_some(),
                "tier '{slug}' lost its device count"
            );
        }

        // Belt and braces: no dollar figure anywhere in the tiers file.
        assert!(
            !raw.contains('$'),
            "feature_tiers.json contains a '$' — that is a price leaking into the software"
        );
    }

    /// The gates must still be a strict superset chain, and Strategy must not be gated (it runs on
    /// the customer's own key, so there is nothing for us to meter).
    #[test]
    fn upgrade_gates_still_ladder_correctly() {
        let (pm, tm, ag) = (
            LicenseTier::Personal.builtin_features(),
            LicenseTier::Team.builtin_features(),
            LicenseTier::Pro.builtin_features(),
        );

        for f in pm {
            assert!(tm.contains(f), "Team lost Personal feature {f}");
        }
        for f in tm {
            assert!(ag.contains(f), "Pro lost Team feature {f}");
        }

        // Research (and therefore the Strategy panel) must be available on the FREE tier.
        assert!(
            pm.contains(&"market_research"),
            "Strategy lives under market_research and must stay available on Personal"
        );
    }

    /// The upgrade offer must step up ONE tier, and must go quiet at the top.
    #[test]
    fn upgrade_offer_steps_up_and_then_stops() {
        assert_eq!(LicenseTier::Personal.next_up(), Some(LicenseTier::Team));
        assert_eq!(LicenseTier::Team.next_up(), Some(LicenseTier::Pro));
        assert_eq!(LicenseTier::Pro.next_up(), Some(LicenseTier::Enterprise));
        assert_eq!(
            LicenseTier::Enterprise.next_up(),
            None,
            "the top tier must offer nothing — a fully licensed customer must never see an upsell"
        );
    }

    /// The "N modules locked" summary must never mention money.
    #[test]
    fn locked_summary_counts_modules_not_money() {
        for t in LicenseTier::all() {
            let s = t.locked_summary();
            assert!(!s.is_empty(), "{t:?} has an empty summary");
            assert!(!s.contains('$'), "summary mentions money: {s}");
            assert!(
                s.contains("module"),
                "summary should count modules, got: {s}"
            );
        }
        // Enterprise holds everything
        assert_eq!(LicenseTier::Enterprise.locked_summary(), "Every module");
    }

    /// A fully-licensed user gets NO offer, and a free user is offered the NEXT step up
    /// rather than the top tier. (Exercised at tier level: building a LicenseManager needs a
    /// live Database handle, and the manager method is a thin composition of these pieces.)
    #[test]
    fn the_offer_is_the_next_step_and_goes_quiet_at_the_top() {
        assert_eq!(LicenseTier::Personal.next_up(), Some(LicenseTier::Team));
        assert_eq!(LicenseTier::Team.next_up(), Some(LicenseTier::Pro));
        assert_eq!(LicenseTier::Pro.next_up(), Some(LicenseTier::Enterprise));

        // `admin_panel` used to be the single feature separating Pro from Enterprise, and it was
        // removed from the sold tiers because it let a CUSTOMER generate licence keys. So the two
        // paid tiers now ship the same feature list, and what Enterprise adds is SEATS — 20 versus
        // unlimited. Asserted rather than assumed, so a future edit that silently drops a feature
        // from Enterprise (and so makes it a downgrade) fails here.
        let top = LicenseTier::Enterprise.builtin_features();
        let pro = LicenseTier::Pro.builtin_features();
        for f in pro {
            assert!(
                top.contains(f),
                "Enterprise must include everything Pro has; '{f}' is missing"
            );
        }
        // `-1` means unlimited, so it cannot be compared with `>`. Normalise first: unlimited is
        // strictly more than any finite seat count.
        let seats = |t: LicenseTier| -> i64 {
            let n = t.max_devices();
            if n < 0 {
                i64::MAX
            } else {
                n
            }
        };
        assert!(
            seats(LicenseTier::Enterprise) > seats(LicenseTier::Pro),
            "Enterprise must be worth buying: it is the unlimited-seat tier"
        );

        // The OWNER class must never be reachable as an upgrade: no tier may offer it, and it must
        // not appear in the sellable list at all.
        assert!(
            !LicenseTier::all().contains(&LicenseTier::Owner),
            "Owner is not a sellable tier and must never appear in all()"
        );
        for t in LicenseTier::all() {
            assert_ne!(t.next_up(), Some(LicenseTier::Owner));
        }

        // And the owner's feature set must be a strict superset of the top sold tier, or the owner
        // would hold LESS than a customer who bought Enterprise.
        let owner = LicenseTier::Owner.builtin_features();
        for f in top {
            assert!(owner.contains(f), "Owner must include Enterprise's '{f}'");
        }
        assert!(
            owner.contains(&"admin_panel"),
            "the Admin panel is the owner's, and only the owner's"
        );
        assert!(
            !top.contains(&"admin_panel"),
            "admin_panel must never be sold in a customer tier"
        );
    }

    /// The Owner class is the single gate on the Admin panel, and no SOLD tier may reach it.
    ///
    /// This is the regression guard for a real defect: `admin_panel` used to sit inside ENTERPRISE
    /// and there was an unguarded toggle in the status bar, so a paying customer could open the
    /// licence-key generator and mint themselves unlimited keys.
    #[test]
    fn only_the_owner_can_reach_the_admin_panel() {
        // The real, shipped key text, so this also proves the OWNER token parses and checks out.
        let owner_key = "DPF-OWNER-SWIFTCEO-T2JG";
        let tier = LicenseManager::validate_key(owner_key, &[])
            .expect("the owner key must validate against the shipped check-code algorithm");
        assert_eq!(tier, LicenseTier::Owner, "OWNER must map to the Owner class");
        assert!(
            features_for_tier(&LicenseTier::Owner).iter().any(|f| f == "admin_panel"),
            "the Owner class must hold admin_panel"
        );

        // No sellable tier may hold it, now or after any future edit to a tier table.
        for t in LicenseTier::all() {
            assert!(
                !features_for_tier(&t).iter().any(|f| f == "admin_panel"),
                "{t:?} is a SOLD tier and must never grant admin_panel"
            );
        }

        // ...and the shipped JSON must agree, since that file (not the code) is authoritative for
        // a tier's feature list at runtime.
        if let Ok(raw) = std::fs::read_to_string("feature_tiers.json") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(tiers) = v.get("tiers").and_then(|t| t.as_object()) {
                    for (slug, t) in tiers {
                        let has = t
                            .get("features")
                            .and_then(|f| f.as_array())
                            .map(|a| a.iter().any(|x| x.as_str() == Some("admin_panel")))
                            .unwrap_or(false);
                        assert!(
                            !has,
                            "feature_tiers.json sells admin_panel in '{slug}' — \
                             a customer could generate licence keys"
                        );
                    }
                }
            }
        }

        // A customer key must NOT validate as the owner.
        for k in ["DPF-PERSONAL-AB12CD34-ZZ00", "DPF-ENTERPRISE-SWIFTCEO-T2JG"] {
            if let Ok(t) = LicenseManager::validate_key(k, &[]) {
                assert_ne!(t, LicenseTier::Owner, "{k} must not be treated as an owner key");
            }
        }
    }
}
