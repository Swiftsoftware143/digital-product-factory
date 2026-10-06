//! Adverts View — Campaign Manager Tab
//!
//! Main tab for the Adverts & Campaign Suite. Allows users to:
//! - Create new campaigns
//! - Generate AI ad concepts
//! - Browse saved campaigns
//! - Navigate to composer/preview/export

use egui::*;
use crate::adverts::{
    AdvertStatus, AspectRatio, Campaign, CampaignStatus,
    CopyFramework, GenerationConfig,
};
use crate::advert_generator::AdvertGenerator;
use crate::llm_router::LLMRouter;
use crate::advert_export::AdvertExporter;
use crate::app::DpfApp;

// ── Campaign Manager State ────────────────────────────────────────────



/// Which sub-view the Adverts tab is showing.
///
/// The Composer and Preview buttons existed but were wired to nothing — `advert_composer::show`
/// and `advert_preview::show` were complete views that no caller ever reached, which is also why
/// every helper only they used showed up as dead code. This mode is the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdvertMode {
    #[default]
    Suite,
    Composer,
    Preview,
}

#[derive(Default)]
pub struct AdvertsManager {
    /// Currently loaded campaign (if any)
    pub campaign: Option<Campaign>,
    /// Saved campaigns list
    pub campaigns: Vec<Campaign>,
    /// Selected advert index for composer
    pub selected_advert: Option<usize>,
    /// Selected advert index for preview
    pub selected_preview: Option<usize>,

    // Generation form state
    pub campaign_name: String,
    pub target_audience: String,
    pub platform: String,
    pub brand_name: String,
    pub tagline: String,
    pub voice_tone: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub font_family: String,
    pub product_name: String,
    pub selected_aspect_ratios: Vec<bool>,
    pub selected_frameworks: Vec<bool>,
    pub num_variations: usize,
    pub generating: bool,
    pub export_path: String,
    /// Which sub-view is showing. Defaults to the suite.
    pub mode: AdvertMode,
    /// Last export/action message, shown as a toast. Cleared once shown.
    pub notice: Option<String>,
}

impl AdvertsManager {
    pub fn new() -> Self {
        Self {
            selected_aspect_ratios: vec![true, false, true, false, false, false, false],
            selected_frameworks: vec![true, true, false],
            num_variations: 2,
            export_path: "exports/".to_string(),
            mode: AdvertMode::Suite,
            ..Default::default()
        }
    }

    /// Build generation config from UI state
    pub fn build_config(&self) -> GenerationConfig {
        let brand_identity = crate::adverts::BrandIdentity {
            brand_name: self.brand_name.clone(),
            tagline: self.tagline.clone(),
            voice_tone: if self.voice_tone.is_empty() { "Professional".into() } else { self.voice_tone.clone() },
            primary_color: if self.primary_color.is_empty() { "#1a1a2e".into() } else { self.primary_color.clone() },
            secondary_color: if self.secondary_color.is_empty() { "#16213e".into() } else { self.secondary_color.clone() },
            accent_color: if self.accent_color.is_empty() { "#e94560".into() } else { self.accent_color.clone() },
            font_family: if self.font_family.is_empty() { "Inter".into() } else { self.font_family.clone() },
            logo_description: String::new(),
        };

        let mut aspect_ratios = Vec::new();
        let ratio_options = AspectRatio::all();
        for (i, &ar) in ratio_options.iter().enumerate() {
            if i < self.selected_aspect_ratios.len() && self.selected_aspect_ratios[i] {
                aspect_ratios.push(ar);
            }
        }

        let framework_options = [CopyFramework::Pas, CopyFramework::Aida, CopyFramework::Bab];
        let mut frameworks = Vec::new();
        for (i, &fw) in framework_options.iter().enumerate() {
            if i < self.selected_frameworks.len() && self.selected_frameworks[i] {
                frameworks.push(fw);
            }
        }

        GenerationConfig {
            brand_identity,
            aspect_ratios,
            copy_frameworks: frameworks,
            num_variations: self.num_variations.max(1),
            include_visuals: true,
        }
    }
}

// ── Main View ─────────────────────────────────────────────────────────

pub fn show(app: &mut DpfApp, ctx: &Context) {
    // The Composer and Preview sub-views are separate view functions. Route to them here so the
    // tab's buttons actually go somewhere.
    match app.adverts_manager.mode {
        AdvertMode::Composer => return crate::ui::advert_composer::show(app, ctx),
        AdvertMode::Preview => return crate::ui::advert_preview::show(app, ctx),
        AdvertMode::Suite => {}
    }

    // Destructure app for split borrows
    let (mgr, config, runtime) = (
        &mut app.adverts_manager,
        &app.config,
        &app.runtime,
    );

    CentralPanel::default().show(ctx, |ui| {

        ui.horizontal(|ui| {
            ui.heading("📢 Adverts & Campaign Suite");
            // `generating` was set but never read, so a campaign could be mid-generation with
            // nothing on screen saying so — the UI just looked frozen.
            if mgr.generating {
                ui.colored_label(Color32::YELLOW, "⏳ Generating campaign…");
            }
            if let Some(ref campaign) = mgr.campaign {
                ui.label(format!("— Active: {}", campaign.name));
            }
        });
        ui.separator();

        // The toast helper existed unused; a completed export should acknowledge itself rather
        // than silently writing a file the user has to go looking for.
        if let Some(msg) = mgr.notice.take() {
            crate::ui::components::toast(ctx, &msg, 3.0);
        }

        // If no campaign loaded, show generation form + saved campaigns
        // Otherwise show campaign actions
        if mgr.campaign.is_some() {
            show_campaign_actions(ui, mgr);
        }

        show_generation_form(ui, mgr, config, runtime);

        // Saved campaigns list
        if !mgr.campaigns.is_empty() {
            ui.separator();
            ui.heading("Saved Campaigns");
            let mut delete_idx: Option<usize> = None;
            ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for (i, campaign) in mgr.campaigns.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} — {} adverts — {:?}",
                            campaign.name,
                            campaign.adverts.len(),
                            campaign.status
                        ));
                        if ui.button("Load").clicked() {
                            mgr.campaign = Some(campaign.clone());
                        }
                        if ui.button("🗑 Delete").clicked() {
                            delete_idx = Some(i);
                        }
                    });
                }
            });
            if let Some(idx) = delete_idx {
                mgr.campaigns.remove(idx);
            }
        }
    });
}

/// Show actions available when a campaign is loaded
fn show_campaign_actions(ui: &mut Ui, mgr: &mut AdvertsManager) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "Campaign: {} | Adverts: {}",
            mgr.campaign.as_ref().map(|c| c.name.as_str()).unwrap_or(""),
            mgr.campaign.as_ref().map(|c| c.adverts.len()).unwrap_or(0),
        ));

        if ui.button("📝 Edit (Composer)").clicked() {
            mgr.mode = AdvertMode::Composer;
        }
        if ui.button("👁️ Preview").clicked() {
            mgr.mode = AdvertMode::Preview;
        }
        if ui.button("📥 Export All").clicked() {
            if let Some(ref campaign) = mgr.campaign {
                let exporter = AdvertExporter::new();
                let safe_name: String = campaign.name.chars().map(|c| if c == ' ' { '_' } else { c }).collect();
                let path = format!("{}_campaign.json", safe_name);
                std::fs::create_dir_all(&mgr.export_path).ok();
                match exporter.write_json_batch_file(&campaign.adverts, &path) {
                    Ok(_) => tracing::info!("Exported campaign to {}", path),
                    Err(e) => tracing::error!("Export failed: {}", e),
                }
            }
        }
        // Campaign summary as Markdown (wired: export_campaign_summary had 0 call sites).
        // The exporter could already render a readable summary of every advert in a campaign;
        // there was no way to ask for it.
        if ui.button("📄 Export Summary").clicked() {
            if let Some(ref campaign) = mgr.campaign {
                let md = AdvertExporter::new()
                    .export_campaign_summary(&campaign.name, &campaign.adverts);
                let safe: String = campaign.name.chars()
                    .map(|c| if c == ' ' { '_' } else { c }).collect();
                let path = format!("{}_summary.md", safe);
                match std::fs::write(&path, md) {
                    Ok(_) => mgr.notice = Some(format!("Summary written to {}", path)),
                    Err(e) => mgr.notice = Some(format!("Summary export failed: {}", e)),
                }
            }
        }
        if ui.button("💾 Save Campaign").clicked() {
            if let Some(ref campaign) = mgr.campaign.clone() {
                // Save to list (replace if exists)
                if let Some(existing) = mgr.campaigns.iter_mut().find(|c| c.id == campaign.id) {
                    *existing = campaign.clone();
                } else {
                    mgr.campaigns.push(campaign.clone());
                }
            }
        }
        if ui.button("✖ Close Campaign").clicked() {
            mgr.campaign = None;
            mgr.selected_advert = None;
            mgr.selected_preview = None;
        }
    });

    ui.separator();

    // Show advert quick stats
    if let Some(ref campaign) = mgr.campaign {
        ui.horizontal(|ui| {
            for advert in &campaign.adverts {
                ui.group(|ui| {
                    ui.set_min_width(120.0);
                    ui.label(&advert.name);
                    ui.label(format!("Score: {}/100", advert.conversion_score));
                    ui.label(format!("Status: {:?}", advert.status));
                });
            }
        });
    }
}

/// Show the generation form
fn show_generation_form(ui: &mut Ui, mgr: &mut AdvertsManager, app_config: &crate::config::AppConfig, runtime: &std::sync::Arc<tokio::runtime::Runtime>) {
    ui.heading("Generate New Campaign");
    ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
        Frame::group(ui.style()).show(ui, |ui| {
            // Basic info
            ui.horizontal(|ui| {
                ui.label("Campaign Name:");
                ui.add(TextEdit::singleline(&mut mgr.campaign_name).desired_width(200.0));
            });
            ui.horizontal(|ui| {
                ui.label("Product Name:");
                ui.add(TextEdit::singleline(&mut mgr.product_name).desired_width(200.0));
            });
            ui.horizontal(|ui| {
                ui.label("Target Audience:");
                ui.add(TextEdit::singleline(&mut mgr.target_audience).desired_width(200.0));
            });
            ui.horizontal(|ui| {
                ui.label("Platform:");
                ComboBox::new("platform_select", "")
                    .selected_text(&mgr.platform)
                    .show_ui(ui, |ui| {
                        let platforms = ["facebook", "instagram", "google", "print", "linkedin"];
                        for &p in &platforms {
                            if ui.selectable_label(mgr.platform == p, p).clicked() {
                                mgr.platform = p.to_string();
                            }
                        }
                    });
            });

            ui.separator();

            // Brand identity
            ui.label("Brand Identity");
            ui.horizontal(|ui| {
                ui.label("Brand Name:");
                ui.add(TextEdit::singleline(&mut mgr.brand_name));
            });
            ui.horizontal(|ui| {
                ui.label("Tagline:");
                ui.add(TextEdit::singleline(&mut mgr.tagline));
            });
            ui.horizontal(|ui| {
                ui.label("Voice Tone:");
                ComboBox::new("voice_tone", "")
                    .selected_text(&mgr.voice_tone)
                    .show_ui(ui, |ui| {
                        for tone in &["Professional", "Playful", "Luxury", "Casual", "Bold"] {
                            if ui.selectable_label(mgr.voice_tone == *tone, *tone).clicked() {
                                mgr.voice_tone = tone.to_string();
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Colors:");
                ui.add(TextEdit::singleline(&mut mgr.primary_color).desired_width(70.0));
                ui.label("Primary");
                ui.add(TextEdit::singleline(&mut mgr.secondary_color).desired_width(70.0));
                ui.label("Secondary");
                ui.add(TextEdit::singleline(&mut mgr.accent_color).desired_width(70.0));
                ui.label("Accent");
            });

            ui.separator();

            // Aspect ratios
            ui.label("Aspect Ratios:");
            ui.horizontal(|ui| {
                let labels = ["1:1 Square", "9:16 Story", "16:9 Landscape", "728x90 Banner", "300x250 Medium Rect", "160x600 Skyscraper", "POD Custom Shape"];
                for (i, label) in labels.iter().enumerate() {
                    let mut checked = mgr.selected_aspect_ratios.get(i).copied().unwrap_or(false);
                    if ui.checkbox(&mut checked, *label).changed() {
                        while mgr.selected_aspect_ratios.len() <= i {
                            mgr.selected_aspect_ratios.push(false);
                        }
                        mgr.selected_aspect_ratios[i] = checked;
                    }
                }
            });

            // Copy frameworks
            ui.label("Copy Frameworks:");
            ui.horizontal(|ui| {
                let labels = ["PAS", "AIDA", "BAB"];
                for (i, label) in labels.iter().enumerate() {
                    let mut checked = mgr.selected_frameworks.get(i).copied().unwrap_or(false);
                    if ui.checkbox(&mut checked, *label).changed() {
                        while mgr.selected_frameworks.len() <= i {
                            mgr.selected_frameworks.push(false);
                        }
                        mgr.selected_frameworks[i] = checked;
                    }
                }
            });

            // Variations
            ui.horizontal(|ui| {
                ui.label("Variations per ratio:");
                let mut nv = mgr.num_variations as f64;
                ui.add(Slider::new(&mut nv, 1.0..=5.0));
                mgr.num_variations = nv as usize;
            });

            ui.separator();

            // Generate button
            if ui.add_sized([200.0, 36.0], Button::new("🎨 Generate Campaign")).clicked() {
                let config = mgr.build_config();

                // Build LLM router from app API keys
                let router = if !app_config.openai_key.is_empty() {
                    Some(LLMRouter::new(
                        app_config.openai_key.clone(),
                        app_config.anthropic_key.clone(),
                        app_config.google_key.clone(),
                        app_config.deepseek_key.clone(),
                        app_config.moonshot_key.clone(),
                    ))
                } else {
                    None
                };

                let generator = AdvertGenerator::new();

                let campaign = generator.generate_campaign(
                    &config,
                    if mgr.campaign_name.is_empty() { "Untitled Campaign" } else { &mgr.campaign_name },
                    if mgr.product_name.is_empty() { "Product" } else { &mgr.product_name },
                    if mgr.target_audience.is_empty() { "General" } else { &mgr.target_audience },
                    if mgr.platform.is_empty() { "facebook" } else { &mgr.platform },
                    router.as_ref(),
                    runtime,
                );

                match campaign {
                    Ok(c) => {
                        mgr.campaign = Some(c);
                    }
                    Err(e) => {
                        tracing::error!("Campaign generation failed: {}", e);
                    }
                }
                mgr.selected_advert = None;
                mgr.selected_preview = None;
            }
        });
    });
}
