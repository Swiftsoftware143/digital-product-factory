//! Where the headless door gets the user's settings — and therefore their BYOK keys.
//!
//! The GUI never stores keys in a file of its own: `DpfApp::save` hands the whole `AppConfig` to
//! eframe, which writes `app.ron` into the per-platform application-data directory. A headless
//! door that invented a SECOND settings file would mean the user configures their key twice (or,
//! worse, believes they configured it once). So the order of resolution is:
//!
//!   1. `--settings PATH` (or `DPF_SETTINGS`) — an explicit eframe `app.ron`, or a bare RON/JSON
//!      `AppConfig`, for anyone who wants to keep the agent's settings separate;
//!   2. the app's own `app.ron`, discovered the same way eframe writes it (`EFRAME_STORAGE_DIR`,
//!      else the platform data directory);
//!   3. environment variables — `DPF_OPENAI_KEY`, `DPF_ANTHROPIC_KEY`, `DPF_GOOGLE_KEY`,
//!      `DPF_DEEPSEEK_KEY`, `DPF_MOONSHOT_KEY` — which always OVERRIDE a file value, so a user
//!      can run the door with a key that is not on disk at all.
//!
//! Nothing here ever logs, prints or returns a key value. `source` is a path/description only.

use crate::config::AppConfig;
use std::path::{Path, PathBuf};

pub struct LoadedSettings {
    pub config: AppConfig,
    /// Human-readable provenance, printed once at startup (never contains a secret).
    pub source: String,
    /// How many of the five AI provider keys are non-empty.
    pub providers_configured: usize,
}

/// Resolve the settings for a headless run.
pub fn load(explicit: Option<&Path>) -> LoadedSettings {
    let explicit = explicit
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::var_os("DPF_SETTINGS").map(PathBuf::from));

    let mut source = "built-in defaults (no settings file found)".to_string();
    let mut config = AppConfig::default();

    if let Some(path) = explicit {
        match read_settings_file(&path) {
            Ok(cfg) => {
                config = cfg;
                source = format!("--settings {}", path.display());
            }
            Err(e) => {
                // An explicitly named file that cannot be read is a configuration mistake the
                // user must see, not something to paper over with a default.
                crate::log_line(&format!("mcp: settings error: {e}"));
                eprintln!("dpf: {e}");
            }
        }
    } else if let Some(path) = discover_app_settings() {
        match read_settings_file(&path) {
            Ok(cfg) => {
                config = cfg;
                source = format!("app settings {}", path.display());
            }
            Err(e) => crate::log_line(&format!("mcp: ignoring unreadable app settings: {e}")),
        }
    }

    let overridden = apply_env_overrides(&mut config);
    if overridden > 0 {
        source.push_str(&format!(" + {overridden} env override(s)"));
    }

    let providers_configured = [
        &config.openai_key,
        &config.anthropic_key,
        &config.google_key,
        &config.deepseek_key,
        &config.moonshot_key,
    ]
    .iter()
    .filter(|k| !k.trim().is_empty())
    .count();

    LoadedSettings {
        config,
        source,
        providers_configured,
    }
}

/// One file, three accepted shapes: eframe's wrapper map, a bare RON `AppConfig`, or JSON.
fn read_settings_file(path: &Path) -> Result<AppConfig, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read settings file {}: {e}", path.display()))?;

    // eframe writes `{ "app": "(openai_key:\"\",…)" }` — a map of key -> RON-encoded value.
    if let Ok(map) = ron::from_str::<std::collections::HashMap<String, String>>(&text) {
        if let Some(inner) = map.get(eframe::APP_KEY).or_else(|| map.values().next()) {
            if let Ok(cfg) = ron::from_str::<AppConfig>(inner) {
                return Ok(cfg);
            }
        }
    }
    if let Ok(cfg) = ron::from_str::<AppConfig>(&text) {
        return Ok(cfg);
    }
    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&text) {
        return Ok(cfg);
    }
    Err(format!(
        "{} is not an eframe app.ron, a RON AppConfig, or a JSON AppConfig",
        path.display()
    ))
}

/// The settings file the GUI itself writes, discovered without guessing.
fn discover_app_settings() -> Option<PathBuf> {
    const FILE: &str = "app.ron";

    if let Some(dir) = std::env::var_os("EFRAME_STORAGE_DIR") {
        let p = PathBuf::from(dir).join(FILE);
        if p.is_file() {
            return Some(p);
        }
    }

    for dir in candidate_dirs() {
        let p = dir.join(FILE);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// eframe (via `directories`) lowercases and strips separators from the app name for its storage
/// directory. Older builds of this app used a differently-cased name, so both are checked — a
/// user must not have to know which build wrote their file.
fn candidate_dirs() -> Vec<PathBuf> {
    let names = [
        "digitalproductfactory",
        "DigitalProductFactory",
        "Digital Product Factory",
    ];
    let mut bases: Vec<PathBuf> = Vec::new();

    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            bases.push(PathBuf::from(appdata));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            bases.push(
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support"),
            );
        }
    } else {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            bases.push(PathBuf::from(xdg));
        }
        if let Some(home) = std::env::var_os("HOME") {
            bases.push(PathBuf::from(home).join(".local").join("share"));
        }
    }

    let mut out = Vec::new();
    for base in bases {
        for name in names {
            out.push(base.join(name));
        }
    }
    out
}

/// Env vars win over the file. Returns how many keys were overridden.
fn apply_env_overrides(config: &mut AppConfig) -> usize {
    let pairs: [(&str, &mut String); 5] = [
        ("DPF_OPENAI_KEY", &mut config.openai_key),
        ("DPF_ANTHROPIC_KEY", &mut config.anthropic_key),
        ("DPF_GOOGLE_KEY", &mut config.google_key),
        ("DPF_DEEPSEEK_KEY", &mut config.deepseek_key),
        ("DPF_MOONSHOT_KEY", &mut config.moonshot_key),
    ];
    let mut n = 0;
    for (name, slot) in pairs {
        if let Ok(v) = std::env::var(name) {
            if !v.trim().is_empty() {
                *slot = v;
                n += 1;
            }
        }
    }
    if let Ok(v) = std::env::var("DPF_STRATEGY_PROVIDER") {
        if !v.trim().is_empty() {
            config.strategy_provider = v;
        }
    }
    if let Ok(v) = std::env::var("DPF_STRATEGY_MODEL") {
        if !v.trim().is_empty() {
            config.strategy_model = v;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eframe_wrapper_is_understood() {
        let cfg = AppConfig {
            deepseek_key: "ds-test-key-not-real".into(),
            ..Default::default()
        };
        let inner = ron::to_string(&cfg).expect("ron encode");
        let text = format!("{{ \"{}\": {inner:?} }}", eframe::APP_KEY);
        // The wrapper stores the value as a RON string.
        let parsed = read_settings_file_from_str(&text).expect("wrapper parses");
        assert_eq!(parsed.deepseek_key, "ds-test-key-not-real");
    }

    #[test]
    fn a_bare_ron_config_is_understood() {
        let cfg = AppConfig {
            openai_key: "oai-test-key-not-real".into(),
            ..Default::default()
        };
        let text = ron::to_string(&cfg).expect("ron encode");
        let parsed = read_settings_file_from_str(&text).expect("bare ron parses");
        assert_eq!(parsed.openai_key, "oai-test-key-not-real");
    }

    #[test]
    fn junk_is_refused_with_a_message() {
        let err = read_settings_file_from_str("not a config at all").expect_err("must refuse");
        assert!(err.contains("not an eframe"));
    }

    /// Test-only seam: same parsing as `read_settings_file`, without touching the filesystem.
    fn read_settings_file_from_str(text: &str) -> Result<AppConfig, String> {
        if let Ok(map) = ron::from_str::<std::collections::HashMap<String, String>>(text) {
            if let Some(inner) = map.get(eframe::APP_KEY).or_else(|| map.values().next()) {
                if let Ok(cfg) = ron::from_str::<AppConfig>(inner) {
                    return Ok(cfg);
                }
            }
        }
        if let Ok(cfg) = ron::from_str::<AppConfig>(text) {
            return Ok(cfg);
        }
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(text) {
            return Ok(cfg);
        }
        Err("not an eframe app.ron, a RON AppConfig, or a JSON AppConfig".to_string())
    }
}
