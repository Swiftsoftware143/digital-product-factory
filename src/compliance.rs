//! Priority 4 — Compliance & Licensing
//!
//! AI disclosure clause generator that adapts to per-platform requirements.
//! Trademark/IP denylist scanning before generation.
//! License terms tracking per AI tool/plan.

use serde::{Deserialize, Serialize};

// ── AI Disclosure Generator ──────────────────────────────────────────

/// Per-platform AI disclosure rules (loaded from JSON, updateable without rebuild)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDisclosureRule {
    pub platform: String,
    pub requires_disclosure: bool,
    pub required_label: Option<String>,
    pub disclosure_text_template: String,
    pub location: String,
    pub last_updated: String,
}

impl AiDisclosureRule {
    pub fn defaults() -> Vec<Self> {
        vec![
            AiDisclosureRule {
                platform: "etsy".into(),
                requires_disclosure: true,
                required_label: Some("AI Generated".into()),
                disclosure_text_template: "This product was created with the assistance of artificial intelligence tools. Final design and quality control performed by [seller_name].".into(),
                location: "description".into(),
                last_updated: "2026-07-01".into(),
            },
            AiDisclosureRule {
                platform: "gumroad".into(),
                requires_disclosure: true,
                required_label: None,
                disclosure_text_template: "Created with AI assistance. See license terms for specific usage rights.".into(),
                location: "description".into(),
                last_updated: "2026-07-01".into(),
            },
            AiDisclosureRule {
                platform: "shopify".into(),
                requires_disclosure: false,
                required_label: None,
                disclosure_text_template: "This product incorporates AI-generated content.".into(),
                location: "description".into(),
                last_updated: "2026-07-01".into(),
            },
            AiDisclosureRule {
                platform: "payhip".into(),
                requires_disclosure: false,
                required_label: None,
                disclosure_text_template: "AI-assisted creation.".into(),
                location: "description".into(),
                last_updated: "2026-07-01".into(),
            },
        ]
    }

    pub fn load(path: &str) -> Vec<Self> {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(rules) = serde_json::from_str(&data) {
                return rules;
            }
        }
        Self::defaults()
    }

    pub fn save(path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(&Self::defaults()) {
            let _ = std::fs::write(path, json);
        }
    }
}

// ── Trademark/IP Denylist ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenylistEntry {
    pub term: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DenylistScanner {
    pub entries: Vec<DenylistEntry>,
    pub custom_terms: Vec<String>,
}

impl Default for DenylistScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl DenylistScanner {
    pub fn new() -> Self {
        Self {
            entries: Self::default_entries(),
            custom_terms: Vec::new(),
        }
    }

    fn default_entries() -> Vec<DenylistEntry> {
        vec![
            DenylistEntry { term: "disney".into(), category: "franchise".into(), description: "The Walt Disney Company — extensive IP protections".into() },
            DenylistEntry { term: "marvel".into(), category: "franchise".into(), description: "Marvel Entertainment — trademarked characters".into() },
            DenylistEntry { term: "star wars".into(), category: "franchise".into(), description: "Star Wars — Lucasfilm IP".into() },
            DenylistEntry { term: "nike".into(), category: "brand".into(), description: "Nike — registered trademark".into() },
            DenylistEntry { term: "adidas".into(), category: "brand".into(), description: "Adidas — registered trademark".into() },
            DenylistEntry { term: "gucci".into(), category: "brand".into(), description: "Gucci — registered trademark".into() },
            DenylistEntry { term: "louis vuitton".into(), category: "brand".into(), description: "Louis Vuitton — registered trademark".into() },
            DenylistEntry { term: "harry potter".into(), category: "character".into(), description: "Harry Potter — Warner Bros IP".into() },
            DenylistEntry { term: "mickey mouse".into(), category: "character".into(), description: "Mickey Mouse — Disney trademark".into() },
            DenylistEntry { term: "pokemon".into(), category: "franchise".into(), description: "Pokémon — Nintendo/The Pokémon Company IP".into() },
            DenylistEntry { term: "nintendo".into(), category: "brand".into(), description: "Nintendo — registered trademark".into() },
            DenylistEntry { term: "coca-cola".into(), category: "brand".into(), description: "Coca-Cola — registered trademark".into() },
            DenylistEntry { term: "coca cola".into(), category: "brand".into(), description: "Coca-Cola — registered trademark".into() },
            DenylistEntry { term: "apple".into(), category: "brand".into(), description: "Apple Inc. — registered trademark (context-dependent)".into() },
            DenylistEntry { term: "mcdonald".into(), category: "brand".into(), description: "McDonald's — registered trademark".into() },
            DenylistEntry { term: "barbie".into(), category: "character".into(), description: "Barbie — Mattel trademark".into() },
            DenylistEntry { term: "lego".into(), category: "brand".into(), description: "LEGO — registered trademark".into() },
            DenylistEntry { term: "pixar".into(), category: "franchise".into(), description: "Pixar — Disney IP".into() },
            DenylistEntry { term: "dc comics".into(), category: "franchise".into(), description: "DC Comics — Warner Bros IP".into() },
            DenylistEntry { term: "batman".into(), category: "character".into(), description: "Batman — DC Comics / Warner Bros trademark".into() },
            DenylistEntry { term: "superman".into(), category: "character".into(), description: "Superman — DC Comics / Warner Bros trademark".into() },
            DenylistEntry { term: "spider-man".into(), category: "character".into(), description: "Spider-Man — Marvel IP".into() },
            DenylistEntry { term: "spiderman".into(), category: "character".into(), description: "Spider-Man — Marvel IP".into() },
            DenylistEntry { term: "hello kitty".into(), category: "character".into(), description: "Hello Kitty — Sanrio trademark".into() },
        ]
    }

    /// Scan a prompt for protected terms. Returns list of formatted warning strings.
    pub fn scan(&self, prompt: &str) -> Vec<String> {
        let lower = prompt.to_lowercase();
        let mut results = Vec::new();

        for entry in &self.entries {
            if lower.contains(&entry.term) {
                results.push(format!("• {} — {} ({})", entry.term, entry.description, entry.category));
            }
        }
        for term in &self.custom_terms {
            if lower.contains(&term.to_lowercase()) {
                results.push(format!("• {} — User-added protected term (custom)", term));
            }
        }

        results.sort();
        results.dedup();
        results
    }

    /// Load custom denylist from JSON file
    pub fn load_denylist(path: &str) -> Vec<String> {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(terms) = serde_json::from_str::<Vec<String>>(&data) {
                return terms;
            }
        }
        Vec::new()
    }
}

// ── License Terms Tracker ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolLicense {
    pub tool_name: String,
    pub plan_tier: String,
    pub commercial_use_allowed: bool,
    pub ownership: String,
    pub attribution_required: bool,
    pub restrictions: Vec<String>,
}

impl AiToolLicense {
    pub fn defaults() -> Vec<Self> {
        vec![
            AiToolLicense {
                tool_name: "OpenAI".into(),
                plan_tier: "GPT-4 / ChatGPT Plus".into(),
                commercial_use_allowed: true,
                ownership: "user_owns_output".into(),
                attribution_required: false,
                restrictions: vec!["Cannot use to compete with OpenAI".into()],
            },
            AiToolLicense {
                tool_name: "Anthropic".into(),
                plan_tier: "Claude Pro / Claude API".into(),
                commercial_use_allowed: true,
                ownership: "user_owns_output".into(),
                attribution_required: false,
                restrictions: vec![],
            },
            AiToolLicense {
                tool_name: "Gemini".into(),
                plan_tier: "Gemini Advanced / API".into(),
                commercial_use_allowed: true,
                ownership: "user_owns_output".into(),
                attribution_required: false,
                restrictions: vec!["Google may use data for service improvement".into()],
            },
        ]
    }
}
