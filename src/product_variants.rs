//! Product Variants & Versioning System
//!
//! A product can have multiple variants (different formats, prices, etc.)
//! Each variant tracks version history for rollback.
//!
//! Pattern: Pluggable adapter, independently testable, local-only SQLite store.
//!
//! Tier: Personal+ (core feature available to all tiers)

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ── Data Models ───────────────────────────────────────────────────────

/// A variant of a product — different format, price, content version, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub id: usize,
    pub product_id: usize,          // FK to pipeline idea id
    pub name: String,                // e.g. "Daily Planner - PDF", "Logo Pack - Full Color"
    pub format: String,              // "pdf", "docx", "xlsx", "zip", "png", "jpg", "txt", "html", "markdown", "json"
    pub price: f64,                  // variant-specific price (overrides product estimate)
    pub status: VariantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_version: u32,        // points to the active VariantVersion
    pub notes: String,
}

/// Status of a variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum VariantStatus {
#[default]
    Draft,       // Being created, not yet ready
    Active,      // Live / ready
    Deprecated,  // Old, superseded by another variant
    Archived,    // No longer in use
}

impl VariantStatus {
    pub fn name(&self) -> &'static str {
        match self {
            VariantStatus::Draft => "📝 Draft",
            VariantStatus::Active => "✅ Active",
            VariantStatus::Deprecated => "⏳ Deprecated",
            VariantStatus::Archived => "📦 Archived",
        }
    }
}

/// A specific version of a variant — immutable snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantVersion {
    pub id: usize,
    pub variant_id: usize,           // FK to variant id
    pub version_number: u32,         // 1, 2, 3, ...
    pub content: String,             // The actual content (text, file path, or serialized data)
    pub content_type: String,        // "text", "filepath", "serialized"
    pub metadata: String,            // JSON blob for extra info (file size, AI model used, params)
    pub created_at: DateTime<Utc>,
    pub file_size_bytes: u64,        // size hint
}

// ── Manager ───────────────────────────────────────────────────────────

pub struct VariantManager {
    pub variants: Vec<Variant>,
    pub versions: Vec<VariantVersion>,
    pub selected_variant: Option<usize>,
    pub show_add_variant_dialog: bool,
    pub show_version_history: Option<usize>, // variant_id to show history for
    pub show_view_version: Option<(usize, u32)>, // (variant_id, version_number) to view
    // Draft fields for the add-variant form
    pub new_variant_name: String,
    pub new_variant_format: String,
    pub new_variant_price: String,
}

impl VariantManager {
    pub fn new(db: &Arc<Database>) -> Self {
        let variants = db.load_variants().unwrap_or_default();
        let versions = db.load_variant_versions().unwrap_or_default();

        Self {
            variants,
            versions,
            selected_variant: None,
            show_add_variant_dialog: false,
            show_version_history: None,
            show_view_version: None,
            new_variant_name: String::new(),
            new_variant_format: String::new(),
            new_variant_price: String::new(),
        }
    }

    /// Get all variants for a given product
    pub fn get_variants_for_product(&self, product_id: usize) -> Vec<&Variant> {
        self.variants
            .iter()
            .filter(|v| v.product_id == product_id)
            .collect()
    }

    /// Get an active (non-archived) variants for a product
    pub fn get_active_variants_for_product(&self, product_id: usize) -> Vec<&Variant> {
        self.variants
            .iter()
            .filter(|v| v.product_id == product_id && v.status != VariantStatus::Archived)
            .collect()
    }

    /// Create a new variant for a product
    pub fn create_variant(
        &mut self,
        db: &Database,
        product_id: usize,
        name: String,
        format: String,
        price: f64,
    ) -> usize {
        let id = self.variants.len() + 1; // simple auto-increment
        let now = Utc::now();

        let variant = Variant {
            id,
            product_id,
            name,
            format,
            price,
            status: VariantStatus::Draft,
            created_at: now,
            updated_at: now,
            current_version: 0,
            notes: String::new(),
        };

        db.save_variant(&variant).ok();
        self.variants.push(variant);
        id
    }

    /// Update a variant's metadata (name, format, price, status)
    pub fn update_variant(
        &mut self,
        db: &Database,
        id: usize,
        name: Option<String>,
        format: Option<String>,
        price: Option<f64>,
        status: Option<VariantStatus>,
        notes: Option<String>,
    ) {
        if let Some(variant) = self.variants.iter_mut().find(|v| v.id == id) {
            if let Some(n) = name { variant.name = n; }
            if let Some(f) = format { variant.format = f; }
            if let Some(p) = price { variant.price = p; }
            if let Some(s) = status { variant.status = s; }
            if let Some(n) = notes { variant.notes = n; }
            variant.updated_at = Utc::now();
            db.save_variant(variant).ok();
        }
    }

    /// Add a new version snapshot to a variant
    pub fn add_version(
        &mut self,
        db: &Database,
        variant_id: usize,
        content: String,
        content_type: String,
        metadata: String,
        file_size_bytes: u64,
    ) -> u32 {
        let variant = if let Some(v) = self.variants.iter_mut().find(|v| v.id == variant_id) {
            v
        } else {
            return 0;
        };

        let version_number = variant.current_version + 1;
        let version_id = self.versions.len() + 1;

        let version = VariantVersion {
            id: version_id,
            variant_id,
            version_number,
            content,
            content_type,
            metadata,
            created_at: Utc::now(),
            file_size_bytes,
        };

        // Update variant's current version
        variant.current_version = version_number;
        variant.updated_at = Utc::now();
        variant.status = VariantStatus::Active;

        db.save_variant(variant).ok();
        db.save_variant_version(&version).ok();
        self.versions.push(version);

        version_number
    }

    /// Get all versions for a variant, sorted newest first
    pub fn get_versions(&self, variant_id: usize) -> Vec<&VariantVersion> {
        let mut versions: Vec<_> = self.versions
            .iter()
            .filter(|v| v.variant_id == variant_id)
            .collect();
        versions.sort_by(|a, b| b.version_number.cmp(&a.version_number));
        versions
    }

    /// Get the current (latest) version for a variant
    pub fn get_current_version(&self, variant_id: usize) -> Option<&VariantVersion> {
        let variant = self.variants.iter().find(|v| v.id == variant_id)?;
        self.versions
            .iter()
            .find(|v| v.variant_id == variant_id && v.version_number == variant.current_version)
    }

    /// Rollback a variant to a previous version
    pub fn rollback_to_version(
        &mut self,
        db: &Database,
        variant_id: usize,
        target_version_number: u32,
    ) -> Option<u32> {
        // Find the target version
        let target = self.versions
            .iter()
            .find(|v| v.variant_id == variant_id && v.version_number == target_version_number)?;

        // Update the variant's pointer
        if let Some(variant) = self.variants.iter_mut().find(|v| v.id == variant_id) {
            variant.current_version = target_version_number;
            variant.updated_at = Utc::now();
            db.save_variant(variant).ok();
            Some(target_version_number)
        } else {
            None
        }
    }

    /// Delete a variant and all its versions
    pub fn delete_variant(&mut self, db: &Database, id: usize) {
        // Remove versions
        self.versions.retain(|v| v.variant_id != id);
        db.delete_variant_versions(id).ok();

        // Remove variant
        self.variants.retain(|v| v.id != id);
        db.delete_variant(id).ok();
    }
}
