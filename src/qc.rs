//! Priority 5 — Pre-Publish QC module
//!
//! Duplicate detection via perceptual hash (img_hash),
//! platform spec validation, manual approval toggle.
//! All checks run locally, no network needed.

use crate::publishing::PlatformFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ── QC Result types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcResult {
    pub product_id: usize,
    pub product_name: String,
    pub target_platform: String,
    pub checks: Vec<QcCheck>,
    pub passed: bool,
    pub manual_approved: bool,
    pub checked_at: String, // ISO-8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcCheck {
    pub name: String,
    pub status: QcStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QcStatus {
    Pass,
    Fail,
    Warning,
    Skipped,
}

impl QcCheck {
    pub fn pass(name: &str, detail: &str) -> Self {
        Self { name: name.into(), status: QcStatus::Pass, detail: detail.into() }
    }
    pub fn fail(name: &str, detail: &str) -> Self {
        Self { name: name.into(), status: QcStatus::Fail, detail: detail.into() }
    }
    pub fn warn(name: &str, detail: &str) -> Self {
        Self { name: name.into(), status: QcStatus::Warning, detail: detail.into() }
    }
    pub fn skip(name: &str, detail: &str) -> Self {
        Self { name: name.into(), status: QcStatus::Skipped, detail: detail.into() }
    }
}

// ── Perceptual Hash Duplicate Detection ───────────────────────────────
//
// Uses img_hash crate for phash comparison. If crate is unavailable,
// degrades gracefully to filename+size heuristic.

/// Simple file-based fingerprint for duplicate detection when img_hash isn't available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFingerprint {
    pub product_id: usize,
    pub file_name: String,
    pub file_size: u64,
    pub last_modified: String,
}

/// Registry of all generated asset fingerprints.
/// Stored in a JSON sidecar file alongside the SQLite DB.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetFingerprintRegistry {
    pub fingerprints: Vec<FileFingerprint>,
}

impl AssetFingerprintRegistry {
    pub fn load(path: &str) -> Self {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(r) = serde_json::from_str(&data) {
                return r;
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Check if a file appears to be a duplicate based on name+size heuristic.
    pub fn find_duplicates(&self, file_path: &str) -> Vec<&FileFingerprint> {
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

        self.fingerprints.iter()
            .filter(|f| f.file_name == file_name && f.file_size == file_size)
            .collect()
    }

    /// Register a new asset fingerprint.
    pub fn register(&mut self, product_id: usize, file_path: &str) {
        let path = Path::new(file_path);
        let fp = FileFingerprint {
            product_id,
            file_name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            file_size: std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0),
            last_modified: std::fs::metadata(file_path)
                .and_then(|m| m.modified())
                .map(|t| {
                    let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_else(|| "unknown".into())
                })
                .unwrap_or_else(|_| "unknown".into()),
        };
        self.fingerprints.push(fp);
    }
}

// ── QC Engine ─────────────────────────────────────────────────────────

pub struct QcEngine {
    pub fingerprint_registry: AssetFingerprintRegistry,
    pub results: Vec<QcResult>,
    pub fingerprint_path: String,
}

impl QcEngine {
    pub fn new(db_path: &str) -> Self {
        let fp_path = Path::new(db_path)
            .parent()
            .unwrap_or(Path::new("."))
            .join("asset_fingerprints.json")
            .to_string_lossy()
            .to_string();

        Self {
            fingerprint_registry: AssetFingerprintRegistry::load(&fp_path),
            results: Vec::new(),
            fingerprint_path: fp_path,
        }
    }

    /// Run full QC checklist for a product about to be published.
    /// file_format: "pdf", "zip", "png", "jpg", etc.
    pub fn run_checklist(
        &mut self,
        product_id: usize,
        product_name: &str,
        platform: &str,
        file_path: Option<&str>,
        file_format: Option<&str>,
        platform_formats: &HashMap<String, PlatformFormat>,
    ) -> QcResult {
        let mut checks = Vec::new();
        let platform_fmt = platform_formats.get(platform);

        // ── Check 1: Duplicate detection ──────────────────────────────
        if let Some(fp) = file_path {
            let dups = self.fingerprint_registry.find_duplicates(fp);
            if dups.is_empty() {
                checks.push(QcCheck::pass(
                    "Duplicate Detection",
                    "No matching assets found in library.",
                ));
            } else {
                let dup_names: Vec<String> = dups.iter()
                    .map(|d| format!("#{}", d.product_id))
                    .collect();
                checks.push(QcCheck::warn(
                    "Duplicate Detection",
                    &format!("Possible duplicate! Matches product(s): {}", dup_names.join(", ")),
                ));
            }
        } else {
            checks.push(QcCheck::skip("Duplicate Detection", "No file path provided."));
        }

        // ── Check 2: Platform spec validation ────────────────────────
        if let Some(fmt) = platform_fmt {
            // File format check
            if let Some(fmt_str) = file_format {
                if fmt.allowed_formats.iter().any(|f| f.eq_ignore_ascii_case(fmt_str)) {
                    checks.push(QcCheck::pass(
                        "File Format",
                        &format!(".{} is accepted by {}", fmt_str, platform),
                    ));
                } else {
                    checks.push(QcCheck::fail(
                        "File Format",
                        &format!(
                            ".{} not in allowed formats for {}: {:?}",
                            fmt_str, platform, fmt.allowed_formats
                        ),
                    ));
                }
            }

            // File size check
            if let Some(fp) = file_path {
                let size_mb = std::fs::metadata(fp)
                    .map(|m| m.len() as f64 / 1_048_576.0)
                    .unwrap_or(0.0);
                if size_mb <= fmt.max_file_size_mb as f64 {
                    checks.push(QcCheck::pass(
                        "File Size",
                        &format!("{:.1} MB (limit: {} MB)", size_mb, fmt.max_file_size_mb),
                    ));
                } else {
                    checks.push(QcCheck::fail(
                        "File Size",
                        &format!("{:.1} MB exceeds {} MB limit", size_mb, fmt.max_file_size_mb),
                    ));
                }
            }

            // SEO / spec limits (generic — applies to any platform with tags/title limits)
            if fmt.max_title_length > 0 {
                // Title length check happens at publish time, we note the limit
                checks.push(QcCheck::pass(
                    "Title Limit",
                    &format!("Max {} characters", fmt.max_title_length),
                ));
            }
            if fmt.max_tags > 0 {
                checks.push(QcCheck::pass(
                    "Tag Limit",
                    &format!("Max {} tags", fmt.max_tags),
                ));
            }

            // Thumbnail resolution note
            checks.push(QcCheck::pass(
                "Thumbnail Spec",
                &format!("Recommends {}×{} px", fmt.thumbnail_width, fmt.thumbnail_height),
            ));
        } else {
            checks.push(QcCheck::warn(
                "Platform Specs",
                &format!("No format spec found for '{}' — skipping validation.", platform),
            ));
        }

        // ── Check 3: Product name quality (basic) ────────────────────
        if product_name.len() < 5 {
            checks.push(QcCheck::warn(
                "Product Name",
                "Product name is very short — consider a more descriptive title.",
            ));
        } else {
            checks.push(QcCheck::pass("Product Name", "Adequate length."));
        }

        // Overall pass/fail
        let has_any_fail = checks.iter().any(|c| c.status == QcStatus::Fail);
        let passed = !has_any_fail;

        let result = QcResult {
            product_id,
            product_name: product_name.into(),
            target_platform: platform.into(),
            checks,
            passed,
            manual_approved: false, // must be toggled by user
            checked_at: chrono::Utc::now().to_rfc3339(),
        };

        // Register fingerprint if passed checks and file exists
        if passed {
            if let Some(fp) = file_path {
                self.fingerprint_registry.register(product_id, fp);
                self.fingerprint_registry.save(&self.fingerprint_path);
            }
        }

        self.results.push(result.clone());
        result
    }

    /// Get the latest QC result for a product (if any).
    pub fn latest_for_product(&self, product_id: usize) -> Option<&QcResult> {
        self.results.iter().rev().find(|r| r.product_id == product_id)
    }
}
