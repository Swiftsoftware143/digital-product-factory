//! License management

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub tier: LicenseTier,
    pub max_devices: usize,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: LicenseStatus,
    pub activated_devices: Vec<Device>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseTier {
    Personal,
    Team,
    Agency,
    Enterprise,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LicenseStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub activated_at: DateTime<Utc>,
}

pub struct LicenseManager {
    db: Arc<Database>,
    current_license: Option<License>,
}

impl LicenseManager {
    pub fn new(db: &Arc<Database>) -> Self {
        // Try to load existing license
        let current_license = None; // TODO: Load from db
        
        Self {
            db: db.clone(),
            current_license,
        }
    }
    
    pub fn is_licensed(&self) -> bool {
        self.current_license.is_some()
    }
    
    pub fn current_license(&self) -> Option<&License> {
        self.current_license.as_ref()
    }
    
    pub fn activate(&mut self, key: &str) -> Result<(), String> {
        // TODO: Validate and activate license
        Ok(())
    }
    
    pub fn deactivate(&mut self) {
        self.current_license = None;
    }
}
