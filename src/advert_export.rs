//! Advert Export — JSON + Image Export
//!
//! Exports adverts as structured JSON with all metadata, copy, and layout specs.
//! Future: PNG/JPEG/SVG rendering of ad previews.

use crate::adverts::{Advert, AdvertExport, ExportFormat};
use chrono::Utc;

pub struct AdvertExporter;

impl AdvertExporter {
    pub fn new() -> Self {
        Self
    }

    /// Export a single advert as a JSON string
    pub fn export_json(&self, advert: &Advert) -> Result<String, String> {
        let export = AdvertExport {
            advert: advert.clone(),
            export_format: ExportFormat::Json,
            exported_at: Utc::now(),
            file_path: None,
        };

        serde_json::to_string_pretty(&export)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    /// Export a batch of adverts as a JSON array
    pub fn export_json_batch(&self, adverts: &[Advert]) -> Result<String, String> {
        let exports: Vec<AdvertExport> = adverts
            .iter()
            .map(|a| AdvertExport {
                advert: a.clone(),
                export_format: ExportFormat::Json,
                exported_at: Utc::now(),
                file_path: None,
            })
            .collect();

        serde_json::to_string_pretty(&exports)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    /// Write a single advert JSON export to disk
    pub fn write_json_file(&self, advert: &Advert, path: &str) -> Result<(), String> {
        let json = self.export_json(advert)?;
        let safe_name = advert.name.replace([' ', '/', '\\', ':'], "_");
        let file_path = format!("{}/{}.json", path.trim_end_matches('/'), safe_name);
        std::fs::write(&file_path, &json)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Write batch export to a single JSON file
    pub fn write_json_batch_file(&self, adverts: &[Advert], file_path: &str) -> Result<(), String> {
        let json = self.export_json_batch(adverts)?;
        std::fs::write(file_path, &json)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Export as a human-readable campaign summary (markdown)
    pub fn export_campaign_summary(&self, campaign_name: &str, adverts: &[Advert]) -> String {
        let mut md = format!("# Campaign: {}\n\n", campaign_name);
        md.push_str(&format!("Total adverts: {}\n\n", adverts.len()));

        for advert in adverts {
            md.push_str(&format!("## {} ({})\n", advert.name, advert.aspect_ratio.label()));
            md.push_str(&format!("- **Conversion Score:** {}/100\n", advert.conversion_score));
            md.push_str(&format!("- **Headline:** {}\n", advert.headline));
            md.push_str(&format!("- **Subheadline:** {}\n", advert.subheadline));
            md.push_str(&format!("- **CTA:** {}\n", advert.call_to_action));
            md.push_str(&format!("- **Visual:** {}\n\n", advert.visual_description));
        }

        md
    }
}
