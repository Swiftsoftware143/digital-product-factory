//! App configuration and settings

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // API Keys
    pub openai_key: String,
    pub anthropic_key: String,
    pub google_key: String,
    pub deepseek_key: String,
    pub moonshot_key: String,

    /// Outbound automation webhooks. `serde(default)` so a settings file written before this
    /// field existed still loads instead of failing.
    #[serde(default)]
    pub webhook: crate::webhooks::WebhookConfig,

    // Preferences
    pub auto_save: bool,
    pub dark_mode: bool,
    pub sidebar_expanded: bool,
    pub default_view: String,

    // Performance
    pub max_concurrent_tasks: usize,
    pub cache_size_mb: usize,

    // Safety limits
    pub max_searches_per_hour: u32,
    pub max_products_per_day: u32,
    pub max_publish_per_hour: u32,

    // Strategy (deep-thinking layer). User-selectable BY DESIGN — the app is never locked
    // to one provider or one model id. Model ids get retired by providers; a user must be
    // able to move to a new one without waiting for an app update.
    //
    //   strategy_provider: "" = auto-pick the strongest provider the user has a key for.
    //                      Otherwise one of: anthropic | deepseek | openai | google | moonshot
    //   strategy_model:    "" = that provider's sensible default. Otherwise ANY model id the
    //                      user types (free text, so a new/renamed model is never a blocker).
    //
    // `serde(default)` keeps existing saved configs loadable — without it, adding these
    // fields would fail to deserialize every settings file already on disk.
    #[serde(default)]
    pub strategy_provider: String,
    #[serde(default)]
    pub strategy_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openai_key: String::new(),
            anthropic_key: String::new(),
            google_key: String::new(),
            deepseek_key: String::new(),
            moonshot_key: String::new(),
            webhook: crate::webhooks::WebhookConfig::default(),
            auto_save: true,
            dark_mode: true,
            sidebar_expanded: true,
            default_view: "dashboard".to_string(),
            max_concurrent_tasks: 4,
            cache_size_mb: 100,
            max_searches_per_hour: 20,
            max_products_per_day: 10,
            max_publish_per_hour: 5,
            // Empty = auto-select the strongest provider/model the user has a key for.
            // An explicit choice here always wins.
            strategy_provider: String::new(),
            strategy_model: String::new(),
        }
    }
}


