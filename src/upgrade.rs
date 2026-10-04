//! Upgrade path — the "offer the upgrade" side of feature gating.
//!
//! Gating and selling are two different jobs and they live in two different places:
//!
//!   * The GATE decides which tabs are open, and lives in `feature_tiers.json` (features per tier).
//!   * The OFFER must be able to change without touching the gate, because sales pages move more
//!     often than feature lists do.
//!
//! So the upgrade target is a single URL resolved in this order:
//!
//!   1. `upgrade_url` in `feature_tiers.json` beside the binary  <- edit this file, no rebuild
//!   2. `DPF_UPGRADE_URL` environment variable                   <- useful for testing
//!   3. `PLACEHOLDER_URL` below                                   <- what ships today
//!
//! `PLACEHOLDER_URL` is deliberately NOT a real store link. The store (Mintbird) is not wired up
//! yet, and shipping a link that goes nowhere is worse than shipping one that says so. The UI
//! renders the placeholder state honestly rather than pretending the button works.
//!
//! WHEN THE REAL CART URL ARRIVES: set `upgrade_url` in `feature_tiers.json`. No code change, no
//! rebuild — that is the entire point of putting it in the data file instead of in this module.

use std::sync::OnceLock;

/// Shipped until a real checkout URL exists.
///
/// A sentinel rather than an empty string, so the UI can tell "no URL configured yet" apart from
/// "the URL failed to load" and say something honest in each case.
pub const PLACEHOLDER_URL: &str = "https://swiftsoftware.net/upgrade-placeholder";

/// Where the placeholder sends the user: nowhere useful, on purpose.
pub fn is_placeholder(url: &str) -> bool {
    url.trim().is_empty() || url.trim() == PLACEHOLDER_URL
}

static RESOLVED: OnceLock<String> = OnceLock::new();

/// The upgrade URL, resolved once per process.
///
/// Read once and cached because it is on a UI path — re-reading the file every frame would be a
/// needless syscall per repaint.
pub fn upgrade_url() -> &'static str {
    RESOLVED.get_or_init(|| {
        // 1. the data file, so a sales-page change needs no rebuild
        if let Ok(raw) = std::fs::read_to_string("feature_tiers.json") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(u) = v.get("upgrade_url").and_then(|u| u.as_str()) {
                    let u = u.trim();
                    if !u.is_empty() {
                        return u.to_string();
                    }
                }
            }
        }

        // 2. env override, for testing without editing the shipped file
        if let Ok(u) = std::env::var("DPF_UPGRADE_URL") {
            let u = u.trim().to_string();
            if !u.is_empty() {
                return u;
            }
        }

        // 3. the shipped placeholder
        PLACEHOLDER_URL.to_string()
    })
}

/// Text for the upgrade button, given the tier the user would be moving to.
///
/// Deliberately contains no price: a licence may be sold one-time or as a subscription, so the
/// price belongs on the sales page, which this button leads to.
pub fn cta_label(target_tier: Option<&str>) -> String {
    match target_tier {
        Some(t) if !t.is_empty() => format!("Upgrade to {t} →"),
        _ => "See upgrade options →".to_string(),
    }
}

/// The honest one-liner shown under the button.
pub fn cta_note() -> &'static str {
    if is_placeholder(upgrade_url()) {
        "Upgrade page is not connected yet — this button will open the store once it is live."
    } else {
        "Opens the upgrade page in your browser."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_is_detected() {
        assert!(is_placeholder(PLACEHOLDER_URL));
        assert!(is_placeholder(""));
        assert!(is_placeholder("   "));
        assert!(!is_placeholder("https://example.com/cart"));
    }

    #[test]
    fn cta_label_has_no_price_in_it() {
        let l = cta_label(Some("Team"));
        assert!(l.contains("Team"));
        assert!(!l.contains('$'), "the CTA must not name a price: {l}");
        // and it still reads sensibly with no tier context
        assert!(!cta_label(None).is_empty());
    }

    /// The shipped data file must carry an `upgrade_url` key, so the sales link is editable
    /// without a rebuild. It may hold the placeholder value — that is expected until the store is
    /// live — but the KEY must exist or nobody will know where to put the real URL.
    #[test]
    fn tiers_file_exposes_an_upgrade_url_key() {
        let raw = std::fs::read_to_string("feature_tiers.json")
            .expect("feature_tiers.json must ship beside the binary");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");
        assert!(
            v.get("upgrade_url").is_some(),
            "feature_tiers.json must expose a top-level 'upgrade_url' so the sales link can be \
             changed without a rebuild"
        );
    }

    #[test]
    fn cta_note_tells_the_truth_about_the_placeholder() {
        // The shipped build has no real URL, so the note must say so rather than promising a page.
        let note = cta_note();
        assert!(!note.is_empty());
        if is_placeholder(upgrade_url()) {
            assert!(note.contains("not connected"), "placeholder note must admit it: {note}");
        }
    }
}
