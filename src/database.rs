//! Fast SQLite database with connection pooling

use rusqlite::{Connection, params, Result as SqlResult};
use std::sync::{Arc, Mutex};
use crate::pipeline::{ProductIdea, PipelineStage, Priority};
use crate::scheduler::ScheduledTask;
use crate::license_manager::License;

pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

/// The per-user application data directory for this platform.
///
/// Deliberately dependency-free: a handful of environment variables cover every platform we ship
/// to, and adding a crate for this would be more surface area than the job needs. Falls back to the
/// current directory only if the environment gives us nothing at all, which keeps the app working
/// in an unusual setup rather than refusing to start.
fn data_dir() -> std::path::PathBuf {
    const APP: &str = "DigitalProductFactory";

    // Windows: %APPDATA%\DigitalProductFactory
    if cfg!(windows) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return std::path::PathBuf::from(appdata).join(APP);
            }
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            if !profile.trim().is_empty() {
                return std::path::PathBuf::from(profile).join("AppData").join("Roaming").join(APP);
            }
        }
        return std::path::PathBuf::from(".");
    }

    // macOS: ~/Library/Application Support/DigitalProductFactory
    if cfg!(target_os = "macos") {
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return std::path::PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join(APP);
            }
        }
        return std::path::PathBuf::from(".");
    }

    // Linux/BSD: $XDG_DATA_HOME/DigitalProductFactory, else ~/.local/share/DigitalProductFactory
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.trim().is_empty() {
            return std::path::PathBuf::from(xdg).join(APP);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return std::path::PathBuf::from(home).join(".local").join("share").join(APP);
        }
    }
    std::path::PathBuf::from(".")
}

impl Database {
    /// Where the database lives.
    ///
    /// This used to be the bare relative path "dpf_data.db", which means the file landed in
    /// whatever directory the app happened to be launched from. Three things followed from that,
    /// all bad for a shipped product:
    ///
    ///   1. Moving or replacing the executable lost every idea, product, sale and asset — because
    ///      "delete the folder and unzip the new version" is exactly how people update.
    ///   2. Launching from a different folder silently opened a DIFFERENT, empty database, so the
    ///      user's work appeared to have vanished.
    ///   3. A shortcut with a different "Start in" directory changed which data you saw.
    ///
    /// The data now lives in the per-user application data directory for the platform, which is
    /// independent of where the program file sits. Updating is therefore just replacing the
    /// executable.
    pub fn data_file_path() -> std::path::PathBuf {
        let dir = data_dir();
        let _ = std::fs::create_dir_all(&dir);
        dir.join("dpf_data.db")
    }

    /// Import a database left in the current directory by an older version, so upgrading does not
    /// look like data loss. The original is left in place (never deleted) and the import happens
    /// only when the new location has no database yet.
    fn migrate_legacy_cwd_database(target: &std::path::Path) {
        let legacy = std::path::Path::new("dpf_data.db");
        if target.exists() || !legacy.exists() {
            return;
        }
        // canonicalise so we never copy a file onto itself
        let a = std::fs::canonicalize(legacy).ok();
        let b = std::fs::canonicalize(target).ok();
        if a.is_some() && a == b {
            return;
        }
        match std::fs::copy(legacy, target) {
            Ok(bytes) => crate::log_line(&format!(
                "data: imported existing database from the program folder ({} bytes) into {}",
                bytes,
                target.display()
            )),
            Err(e) => crate::log_line(&format!(
                "data: could NOT import the database in the program folder: {e}"
            )),
        }
    }

    pub fn new() -> SqlResult<Self> {
        let path = Self::data_file_path();
        Self::migrate_legacy_cwd_database(&path);
        crate::log_line(&format!("data: database at {}", path.display()));

        let conn = Connection::open(&path)?;
        
        // Enable WAL mode for better concurrency.
        // NOTE: "PRAGMA journal_mode=..." always returns the resulting mode
        // as a row, even though it's a "set" statement — rusqlite's
        // `.execute()` rejects any statement that returns rows, so this one
        // must go through `query_row` instead (unlike the other PRAGMAs
        // below, which don't return anything).
        conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get::<_, String>(0))?;
        conn.execute("PRAGMA synchronous=NORMAL;", [])?;
        conn.execute("PRAGMA cache_size=10000;", [])?;
        conn.execute("PRAGMA temp_store=memory;", [])?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.init_tables()?;
        
        Ok(db)
    }
    
    fn init_tables(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        // Initialize analytics + publishing tables
        crate::db_ext::init_business_tables(&conn)?;

        // Initialize variant tables
        self.create_variants_table_inner(&conn)?;

        // Pipeline ideas table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ideas (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                stage TEXT NOT NULL,
                product_type TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                priority TEXT NOT NULL,
                tags TEXT,
                estimated_value REAL,
                actual_value REAL,
                notes TEXT,
                platform TEXT
            )",
            [],
        )?;
        
        // Scheduled tasks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS scheduled_tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                task_type TEXT NOT NULL,
                schedule TEXT NOT NULL,
                next_run TEXT,
                last_run TEXT,
                status TEXT NOT NULL,
                data TEXT
            )",
            [],
        )?;
        
        // License table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS licenses (
                key TEXT PRIMARY KEY,
                tier TEXT NOT NULL,
                max_devices INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                status TEXT NOT NULL,
                activated_devices TEXT,
                metadata TEXT
            )",
            [],
        )?;
        
        // Products table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS products (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                product_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL,
                file_path TEXT,
                metadata TEXT
            )",
            [],
        )?;
        
        // Create indexes for fast queries
        conn.execute("CREATE INDEX IF NOT EXISTS idx_ideas_stage ON ideas(stage);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_ideas_priority ON ideas(priority);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_status ON scheduled_tasks(status);", [])?;

        // Variants table (product variants with versioning)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS variants (
                id INTEGER PRIMARY KEY,
                product_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                format TEXT NOT NULL,
                price REAL NOT NULL DEFAULT 0.0,
                status TEXT NOT NULL DEFAULT 'Draft',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                current_version INTEGER NOT NULL DEFAULT 0,
                notes TEXT DEFAULT ''
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS variant_versions (
                id INTEGER PRIMARY KEY,
                variant_id INTEGER NOT NULL,
                version_number INTEGER NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                content_type TEXT NOT NULL DEFAULT 'text',
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL,
                file_size_bytes INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_variants_product ON variants(product_id);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_varversions_variant ON variant_versions(variant_id);", [])?;

        // Advert module: campaign and advert tables
        conn.execute(
           "CREATE TABLE IF NOT EXISTS campaigns (
               id INTEGER PRIMARY KEY,
               name TEXT NOT NULL,
               description TEXT NOT NULL DEFAULT '',
               product_name TEXT NOT NULL DEFAULT '',
               target_audience TEXT NOT NULL DEFAULT '',
               platform TEXT NOT NULL DEFAULT 'facebook',
               status TEXT NOT NULL DEFAULT 'Draft',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
           )",
           [],
        )?;
        
        conn.execute(
           "CREATE TABLE IF NOT EXISTS adverts (
               id INTEGER PRIMARY KEY,
               campaign_id INTEGER NOT NULL,
               product_id INTEGER NOT NULL,
               product_name TEXT NOT NULL,
               format TEXT NOT NULL,
               copy_json TEXT NOT NULL,
               background_json TEXT NOT NULL,
               conversion_score INTEGER NOT NULL DEFAULT 0,
               status TEXT NOT NULL DEFAULT 'Draft',
               created_at TEXT NOT NULL,
               FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
           )",
           [],
        )?;
        

        
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    /// Create the variants + variant_versions tables (might be called separately for standalone init)
    fn create_variants_table_inner(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS variants (
                id INTEGER PRIMARY KEY,
                product_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                format TEXT NOT NULL DEFAULT '',
                price REAL NOT NULL DEFAULT 0.0,
                status TEXT NOT NULL DEFAULT 'Draft',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                current_version INTEGER NOT NULL DEFAULT 0,
                notes TEXT DEFAULT ''
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS variant_versions (
                id INTEGER PRIMARY KEY,
                variant_id INTEGER NOT NULL,
                version_number INTEGER NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                content_type TEXT NOT NULL DEFAULT 'text',
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL,
                file_size_bytes INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_variants_product ON variants(product_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_varversions_variant ON variant_versions(variant_id)",
            [],
        )?;

        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    /// Create variant tables (public accessor for migration scenarios)
    pub fn create_variants_table(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        self.create_variants_table_inner(&conn)
    }
    
    // Pipeline operations
    pub fn load_ideas(&self) -> SqlResult<Vec<ProductIdea>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, stage, product_type, 
                    created_at, updated_at, priority, tags, 
                    estimated_value, actual_value, notes, platform 
             FROM ideas ORDER BY updated_at DESC"
        )?;
        
        let ideas = stmt.query_map([], |row| {
            Ok(ProductIdea {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                stage: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(PipelineStage::Idea),
                product_type: row.get(4)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                priority: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or(Priority::Medium),
                tags: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                estimated_value: row.get(9)?,
                actual_value: row.get(10)?,
                notes: row.get(11)?,
                platform: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
            })
        })?;
        
        ideas.collect()
    }
    
    pub fn save_idea(&self, idea: &ProductIdea) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO ideas 
             (id, title, description, stage, product_type, created_at, updated_at, 
              priority, tags, estimated_value, actual_value, notes, platform)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                idea.id,
                idea.title,
                idea.description,
                serde_json::to_string(&idea.stage).unwrap(),
                idea.product_type,
                idea.created_at.to_rfc3339(),
                idea.updated_at.to_rfc3339(),
                serde_json::to_string(&idea.priority).unwrap(),
                serde_json::to_string(&idea.tags).unwrap(),
                idea.estimated_value,
                idea.actual_value,
                idea.notes,
                serde_json::to_string(&idea.platform).unwrap(),
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }
    
    pub fn delete_idea(&self, id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ideas WHERE id = ?1", params![id])?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }
    
    // License operations

    /// Load the stored licence (there is at most one). Used at startup so activation
    /// survives a restart.
    pub fn load_active_license(&self) -> SqlResult<Option<License>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, tier, max_devices, created_at, expires_at,
                    status, activated_devices, metadata
             FROM licenses LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(License {
                key: row.get(0)?,
                tier: serde_json::from_str(&row.get::<_, String>(1)?)
                    .unwrap_or(crate::license_manager::LicenseTier::Personal),
                max_devices: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                expires_at: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                status: serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or(crate::license_manager::LicenseStatus::Active),
                activated_devices: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                metadata: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_license(&self, key: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM licenses WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn save_license(&self, license: &License) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO licenses 
             (key, tier, max_devices, created_at, expires_at, status, activated_devices, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                license.key,
                serde_json::to_string(&license.tier).unwrap(),
                license.max_devices,
                license.created_at.to_rfc3339(),
                license.expires_at.map(|d| d.to_rfc3339()),
                serde_json::to_string(&license.status).unwrap(),
                serde_json::to_string(&license.activated_devices).unwrap(),
                license.metadata.clone(),
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }
    
    pub fn get_license(&self, key: &str) -> SqlResult<Option<License>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, tier, max_devices, created_at, expires_at, 
                    status, activated_devices, metadata 
             FROM licenses WHERE key = ?1"
        )?;
        
        let mut rows = stmt.query(params![key])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(License {
                key: row.get(0)?,
                tier: serde_json::from_str(&row.get::<_, String>(1)?).unwrap(),
                max_devices: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                expires_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                status: serde_json::from_str(&row.get::<_, String>(5)?).unwrap(),
                activated_devices: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                metadata: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    // Scheduled tasks operations
    pub fn load_scheduled_tasks(&self) -> SqlResult<Vec<crate::scheduler::ScheduledTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, task_type, schedule, next_run, last_run, status, data, enabled 
             FROM scheduled_tasks ORDER BY next_run"
        )?;
        
        let tasks = stmt.query_map([], |row| {
            Ok(crate::scheduler::ScheduledTask {
                id: row.get(0)?,
                name: row.get(1)?,
                task_type: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                schedule: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                next_run: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                last_run: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc)),
                status: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or(crate::scheduler::TaskStatus::Pending),
                data: row.get(7)?,
                enabled: row.get(8)?,
            })
        })?;
        
        tasks.collect()
    }
    
    pub fn save_scheduled_task(&self, task: &crate::scheduler::ScheduledTask) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO scheduled_tasks 
             (id, name, task_type, schedule, next_run, last_run, status, data, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                task.id,
                task.name,
                serde_json::to_string(&task.task_type).unwrap(),
                serde_json::to_string(&task.schedule).unwrap(),
                task.next_run.map(|d| d.to_rfc3339()),
                task.last_run.map(|d| d.to_rfc3339()),
                serde_json::to_string(&task.status).unwrap(),
                task.data,
                task.enabled,
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    // ── Variant Operations ─────────────────────────────────────────────

    pub fn load_variants(&self) -> SqlResult<Vec<crate::product_variants::Variant>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, product_id, name, format, price, status, created_at, updated_at, current_version, notes
             FROM variants ORDER BY created_at DESC"
        )?;

        let variants = stmt.query_map([], |row| {
            Ok(crate::product_variants::Variant {
                id: row.get(0)?,
                product_id: row.get(1)?,
                name: row.get(2)?,
                format: row.get(3)?,
                price: row.get(4)?,
                status: serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or(crate::product_variants::VariantStatus::Draft),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                current_version: row.get(8)?,
                notes: row.get(9)?,
            })
        })?;

        variants.collect()
    }

    pub fn save_variant(&self, variant: &crate::product_variants::Variant) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO variants (id, product_id, name, format, price, status, created_at, updated_at, current_version, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                variant.id,
                variant.product_id,
                variant.name,
                variant.format,
                variant.price,
                serde_json::to_string(&variant.status).unwrap(),
                variant.created_at.to_rfc3339(),
                variant.updated_at.to_rfc3339(),
                variant.current_version,
                variant.notes,
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    pub fn delete_variant(&self, id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM variants WHERE id = ?1", params![id])?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    pub fn load_variant_versions(&self) -> SqlResult<Vec<crate::product_variants::VariantVersion>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, variant_id, version_number, content, content_type, metadata, created_at, file_size_bytes
             FROM variant_versions ORDER BY variant_id, version_number DESC"
        )?;

        let versions = stmt.query_map([], |row| {
            Ok(crate::product_variants::VariantVersion {
                id: row.get(0)?,
                variant_id: row.get(1)?,
                version_number: row.get(2)?,
                content: row.get(3)?,
                content_type: row.get(4)?,
                metadata: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                file_size_bytes: row.get(7)?,
            })
        })?;

        versions.collect()
    }

    pub fn save_variant_version(&self, version: &crate::product_variants::VariantVersion) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO variant_versions (id, variant_id, version_number, content, content_type, metadata, created_at, file_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                version.id,
                version.variant_id,
                version.version_number,
                version.content,
                version.content_type,
                version.metadata,
                version.created_at.to_rfc3339(),
                version.file_size_bytes,
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    pub fn load_variant_versions_for(&self, variant_id: usize) -> SqlResult<Vec<crate::product_variants::VariantVersion>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, variant_id, version_number, content, content_type, metadata, created_at, file_size_bytes
             FROM variant_versions WHERE variant_id = ?1 ORDER BY version_number DESC"
        )?;

        let versions = stmt.query_map(params![variant_id], |row| {
            Ok(crate::product_variants::VariantVersion {
                id: row.get(0)?,
                variant_id: row.get(1)?,
                version_number: row.get(2)?,
                content: row.get(3)?,
                content_type: row.get(4)?,
                metadata: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                file_size_bytes: row.get(7)?,
            })
        })?;

        versions.collect()
    }

    pub fn delete_variant_versions(&self, variant_id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM variant_versions WHERE variant_id = ?1", params![variant_id])?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    // ── Sales Records ────────────────────────────────────────────────

    // ── Clients ──────────────────────────────────────────────────────────
    // Backs the `client_management` feature sold on the Pro + Enterprise tiers.

    pub fn load_clients(&self) -> SqlResult<Vec<crate::client_manager::Client>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, company, status, notes, created_at
             FROM clients ORDER BY name COLLATE NOCASE ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(crate::client_manager::Client {
                id: row.get::<_, i64>(0)? as usize,
                name: row.get(1)?,
                email: row.get(2)?,
                company: row.get(3)?,
                status: crate::client_manager::ClientStatus::from_label(&row.get::<_, String>(4)?),
                notes: row.get(5)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        rows.collect()
    }

    pub fn save_client(&self, client: &crate::client_manager::Client) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO clients (id, name, email, company, status, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                client.id as i64,
                client.name,
                client.email,
                client.company,
                client.status.label(),
                client.notes,
                client.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_client(&self, id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clients WHERE id = ?1", params![id as i64])?;
        Ok(())
    }

    pub fn load_sales_records(&self) -> SqlResult<Vec<crate::analytics::SalesRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, product_id, product_name, platform, units_sold, revenue, fee, net_revenue, sale_date, recorded_at, notes
             FROM sales_records ORDER BY sale_date DESC"
        )?;

        let records = stmt.query_map([], |row| {
            Ok(crate::analytics::SalesRecord {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                platform: row.get(3)?,
                units_sold: row.get::<_, i32>(4)? as u32,
                revenue: row.get(5)?,
                fee: row.get(6)?,
                net_revenue: row.get(7)?,
                sale_date: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                recorded_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                notes: row.get(10)?,
            })
        })?;

        records.collect()
    }

    pub fn save_sales_record(&self, record: &crate::analytics::SalesRecord) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sales_records (id, product_id, product_name, platform, units_sold, revenue, fee, net_revenue, sale_date, recorded_at, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                record.id,
                record.product_id,
                record.product_name,
                record.platform,
                record.units_sold,
                record.revenue,
                record.fee,
                record.net_revenue,
                record.sale_date.to_rfc3339(),
                record.recorded_at.to_rfc3339(),
                record.notes,
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    pub fn delete_sales_record(&self, id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sales_records WHERE id = ?1", params![id])?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    // ── Publish Logs ─────────────────────────────────────────────────

    pub fn load_publish_logs(&self) -> SqlResult<Vec<crate::publishing::PublishLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, product_id, product_name, platform, listing_url, listing_id, status, error_message, published_at
             FROM publish_logs ORDER BY published_at DESC"
        )?;

        let logs = stmt.query_map([], |row| {
            Ok(crate::publishing::PublishLog {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                platform: row.get(3)?,
                listing_url: row.get(4)?,
                listing_id: row.get(5)?,
                status: serde_json::from_str(&row.get::<_, String>(6)?)
                    .unwrap_or(crate::publishing::PublishStatus::Published),
                error_message: row.get(7)?,
                published_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
            })
        })?;

        logs.collect()
    }

    pub fn save_publish_log(&self, log: &crate::publishing::PublishLog) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO publish_logs (id, product_id, product_name, platform, listing_url, listing_id, status, error_message, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                log.id,
                log.product_id,
                log.product_name,
                log.platform,
                log.listing_url,
                log.listing_id,
                serde_json::to_string(&log.status).unwrap(),
                log.error_message,
                log.published_at.to_rfc3339(),
            ],
        )?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }

    pub fn delete_publish_log(&self, id: usize) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM publish_logs WHERE id = ?1", params![id])?;
        // Logo & Vector Generator tables
        let logo_sql = r#"CREATE TABLE IF NOT EXISTS logos (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brand_name TEXT NOT NULL,
            tagline TEXT DEFAULT '',
            style TEXT NOT NULL,
            palette TEXT DEFAULT '[]',
            icon_svg TEXT DEFAULT '',
            typography_svg TEXT DEFAULT '',
            full_svg TEXT DEFAULT '',
            favicon_enabled INTEGER DEFAULT 0,
            favicon_package_json TEXT,
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(logo_sql, [])?;

        let vector_sql = r#"CREATE TABLE IF NOT EXISTS vector_assets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            prompt TEXT DEFAULT '',
            svg_content TEXT DEFAULT '',
            palette TEXT DEFAULT '[]',
            view_box TEXT DEFAULT '0 0 200 200',
            export_formats TEXT DEFAULT '[]',
            status TEXT DEFAULT 'draft',
            created_at TEXT DEFAULT (datetime('now'))
        )"#;
        conn.execute(vector_sql, [])?;

        Ok(())
    }
}
