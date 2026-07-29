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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openai_key: String::new(),
            anthropic_key: String::new(),
            google_key: String::new(),
            deepseek_key: String::new(),
            moonshot_key: String::new(),
            auto_save: true,
            dark_mode: true,
            sidebar_expanded: true,
            default_view: "dashboard".to_string(),
            max_concurrent_tasks: 4,
            cache_size_mb: 100,
            max_searches_per_hour: 20,
            max_products_per_day: 10,
            max_publish_per_hour: 5,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishingConfig {
    pub platform_settings_path: String,
}

impl PublishingConfig {
    pub fn default_path() -> String {
        "platform_formats.json".to_string()
    }
}
