//! Priority 3 ??? Asset Library & Version Management
//!
//! Local asset library view for browsing all generated products
//! independent of pipeline stage. Version history per product with
//! rollback. Optional cloud backup (off by default).

use crate::database::Database;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ?????? Asset Record ??????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: usize,
    pub product_id: usize,
    pub product_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub file_format: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVersion {
    pub id: usize,
    pub asset_id: usize,
    pub version: u32,
    pub file_path: String,
    pub file_size: u64,
    pub created_at: DateTime<Utc>,
    pub change_notes: String,
}

// ?????? Asset Library ???????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????

pub struct AssetLibrary {
    pub assets: Vec<Asset>,
    pub versions: HashMap<usize, Vec<AssetVersion>>,
    pub search_query: String,
    pub tag_filter: Vec<String>,
    pub selected_asset: Option<usize>,
    pub selected_version: Option<u32>,
    pub dirty: bool,
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLibrary {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            versions: HashMap::new(),
            search_query: String::new(),
            tag_filter: Vec::new(),
            selected_asset: None,
            selected_version: None,
            dirty: false,
        }
    }

    /// Load assets from the products table in DB
    pub fn load_from_db(&mut self, db: &Database) {
        // The guard is scoped to the query and dropped before load_versions() is called.
        //
        // It used to hold `conn` for the whole body and then call self.load_versions(db), which
        // locks the SAME std::sync::Mutex. That mutex is NOT reentrant, so the second lock blocked
        // forever: DpfApp::new never returned, the first frame was never rendered, and the window
        // stayed black on every machine regardless of GPU or renderer. This was the actual cause of
        // the black screen — not the graphics back end.
        let assets: Vec<Asset> = {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, product_type, created_at, status, file_path, metadata
                 FROM products ORDER BY created_at DESC"
            ).unwrap();

            stmt.query_map([], |row| {
                let id_val: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let created: String = row.get(3)?;
                let file_path: String = row.get::<_, Option<String>>(5).unwrap_or(None).unwrap_or_default();
                let metadata: String = row.get::<_, Option<String>>(6).unwrap_or(None).unwrap_or_default();

                let tags: Vec<String> = serde_json::from_str(&metadata).unwrap_or_default();
                let file_size = if !file_path.is_empty() {
                    std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0)
                } else { 0 };
                let file_format = Path::new(&file_path)
                    .extension().and_then(|e| e.to_str())
                    .unwrap_or("unknown").to_string();
                let created_at = chrono::DateTime::parse_from_rfc3339(&created)
                    .unwrap_or_default().with_timezone(&chrono::Utc);

                Ok(Asset {
                    id: id_val as usize,
                    product_id: id_val as usize,
                    product_name: name,
                    file_path,
                    file_size,
                    file_format,
                    version: 1,
                    tags,
                    created_at,
                    updated_at: created_at,
                    notes: String::new(),
                })
            }).unwrap().filter_map(|r| r.ok()).collect()
        }; // <-- conn released here

        self.assets = assets;
        self.load_versions(db);
    }

    /// Load version history from asset_versions table
    fn load_versions(&mut self, db: &Database) {
        let conn = db.conn.lock().unwrap();
        let table_exists: bool = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='asset_versions'"
        ).and_then(|mut s| s.exists(params![])).unwrap_or(false);

        if !table_exists { return; }

        let mut stmt = conn.prepare(
            "SELECT id, asset_id, version, file_path, file_size, created_at, change_notes
             FROM asset_versions ORDER BY version DESC"
        ).unwrap();

        let versions: Vec<AssetVersion> = stmt.query_map([], |row| {
            Ok(AssetVersion {
                id: row.get::<_, i64>(0)? as usize,
                asset_id: row.get::<_, i64>(1)? as usize,
                version: row.get::<_, i32>(2)? as u32,
                file_path: row.get(3)?,
                file_size: row.get::<_, i64>(4)? as u64,
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<_, String>(5)?
                ).unwrap_or_default().with_timezone(&chrono::Utc),
                change_notes: row.get(6)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect();

        for v in versions {
            self.versions.entry(v.asset_id).or_default().push(v);
        }
    }

    /// Register a new version when a product is regenerated
    pub fn register_version(&mut self, db: &Database, product_id: usize, file_path: &str, notes: &str) {
        let version_num = self.versions.get(&product_id)
            .map(|v| v.iter().map(|v| v.version).max().unwrap_or(0) + 1)
            .unwrap_or(1);

        let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

        let ver = AssetVersion {
            id: 0,
            asset_id: product_id,
            version: version_num,
            file_path: file_path.into(),
            file_size,
            created_at: Utc::now(),
            change_notes: notes.into(),
        };

        let conn = db.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO asset_versions (asset_id, version, file_path, file_size, created_at, change_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                ver.asset_id as i64,
                ver.version as i32,
                ver.file_path,
                ver.file_size as i64,
                ver.created_at.to_rfc3339(),
                ver.change_notes,
            ],
        );

        self.versions.entry(product_id).or_default().push(ver);
        self.dirty = true;
    }

    /// Get all versions for an asset
    pub fn versions_for(&self, product_id: usize) -> Vec<&AssetVersion> {
        self.versions.get(&product_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Search & filter assets
    pub fn filtered_assets(&self) -> Vec<&Asset> {
        self.assets.iter()
            .filter(|a| {
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !a.product_name.to_lowercase().contains(&q) &&
                       !a.tags.iter().any(|t| t.to_lowercase().contains(&q)) &&
                       !a.file_format.to_lowercase().contains(&q) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// All unique tags across assets
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.assets.iter()
            .flat_map(|a| a.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Revert an asset to a previous version (copies old file over current)
    pub fn rollback_to(&self, product_id: usize, version: u32) -> Result<String, String> {
        let versions = self.versions.get(&product_id)
            .ok_or_else(|| "No version history".to_string())?;

        let ver = versions.iter()
            .find(|v| v.version == version)
            .ok_or_else(|| format!("Version {} not found", version))?;

        if !Path::new(&ver.file_path).exists() {
            return Err("Version file not found".into());
        }

        if let Some(asset) = self.assets.iter().find(|a| a.product_id == product_id) {
            let rollback_path = asset.file_path.replace(
                &format!(".{}", asset.file_format),
                &format!("_v{}.{}", version, asset.file_format)
            );
            std::fs::copy(&ver.file_path, &rollback_path)
                .map_err(|e| format!("Copy failed: {}", e))?;
            Ok(rollback_path)
        } else {
            Err("Asset not found".into())
        }
    }
}

// ?????? Cloud Backup Config (stub, off by default) ????????????????????????????????????????????????????????????????????????

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudBackupConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub last_sync: Option<String>,
}

impl Default for CloudBackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "https://s3.us-east-1.backblazeb2.com".into(),
            bucket: "dpf-assets".into(),
            region: "us-east-1".into(),
            access_key: String::new(),
            secret_key: String::new(),
            last_sync: None,
        }
    }
}

pub fn load_cloud_config(path: &str) -> CloudBackupConfig {
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str(&data) {
            return cfg;
        }
    }
    CloudBackupConfig::default()
}

pub fn save_cloud_config(path: &str, cfg: &CloudBackupConfig) {
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
