//! Adverts & Campaign Suite — Core Domain Types
//!
//! Defines the data model for ad creatives, campaigns, copy variations,
//! aspect ratios, conversion scores, and AI-generated concepts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Aspect Ratio Definitions ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatio {
    Square,            // 1:1   — 1080x1080
    Story,             // 9:16  — 1080x1920
    Landscape,         // 16:9  — 1200x628
    Banner,            // 728x90
    MediumRectangle,   // 300x250
    Skyscraper,        // 160x600
    PodShape,          // Custom POD placement (hexagon, circle badge, etc.)
}

impl AspectRatio {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            AspectRatio::Square => (1080, 1080),
            AspectRatio::Story => (1080, 1920),
            AspectRatio::Landscape => (1200, 628),
            AspectRatio::Banner => (728, 90),
            AspectRatio::MediumRectangle => (300, 250),
            AspectRatio::Skyscraper => (160, 600),
            AspectRatio::PodShape => (1080, 1080),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AspectRatio::Square => "1:1 Square",
            AspectRatio::Story => "9:16 Story",
            AspectRatio::Landscape => "16:9 Landscape",
            AspectRatio::Banner => "728x90 Banner",
            AspectRatio::MediumRectangle => "300x250 Medium Rectangle",
            AspectRatio::Skyscraper => "160x600 Skyscraper",
            AspectRatio::PodShape => "POD Custom Shape",
        }
    }

    pub fn all() -> Vec<AspectRatio> {
        vec![
            AspectRatio::Square,
            AspectRatio::Story,
            AspectRatio::Landscape,
            AspectRatio::Banner,
            AspectRatio::MediumRectangle,
            AspectRatio::Skyscraper,
            AspectRatio::PodShape,
        ]
    }
}

// ── Copywriting Frameworks ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyFramework {
    Pas,  // Problem → Agitate → Solve
    Aida, // Attention → Interest → Desire → Action
    Bab,  // Before → After → Bridge
}

impl CopyFramework {
    pub fn label(&self) -> &'static str {
        match self {
            CopyFramework::Pas => "PAS (Problem-Agitate-Solve)",
            CopyFramework::Aida => "AIDA (Attention-Interest-Desire-Action)",
            CopyFramework::Bab => "BAB (Before-After-Bridge)",
        }
    }

    pub fn all() -> Vec<CopyFramework> {
        vec![CopyFramework::Pas, CopyFramework::Aida, CopyFramework::Bab]
    }
}

// ── Advert Model ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advert {
    pub id: usize,
    pub campaign_id: usize,
    pub name: String,
    pub aspect_ratio: AspectRatio,
    pub headline: String,
    pub subheadline: String,
    pub body_copy: String,
    pub call_to_action: String,
    pub visual_description: String,
    pub brand_identity: BrandIdentity,
    pub conversion_score: u8,           // 0–100
    pub copy_framework: CopyFramework,
    pub product_placement: ProductPlacement,
    pub layout_spec: LayoutSpec,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: AdvertStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvertStatus {
    Draft,
    Generated,
    Approved,
    Exported,
}

// ── Brand Identity ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandIdentity {
    pub brand_name: String,
    pub tagline: String,
    pub voice_tone: String,             // e.g. "Professional", "Playful", "Luxury"
    pub primary_color: String,          // hex
    pub secondary_color: String,        // hex
    pub accent_color: String,           // hex
    pub font_family: String,            // e.g. "Inter", "Playfair Display"
    pub logo_description: String,
}

impl Default for BrandIdentity {
    fn default() -> Self {
        Self {
            brand_name: String::new(),
            tagline: String::new(),
            voice_tone: "Professional".into(),
            primary_color: "#1a1a2e".into(),
            secondary_color: "#16213e".into(),
            accent_color: "#e94560".into(),
            font_family: "Inter".into(),
            logo_description: String::new(),
        }
    }
}

// ── Product Placement ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPlacement {
    pub product_name: String,
    pub mockup_path: Option<String>,
    pub scale_percent: f32,             // 0.0–100.0, size relative to canvas
    pub position_x: f32,                // 0.0–100.0, percent from left
    pub position_y: f32,                // 0.0–100.0, percent from top
    pub rotation_degrees: f32,
    pub shadow_enabled: bool,
}

impl Default for ProductPlacement {
    fn default() -> Self {
        Self {
            product_name: String::new(),
            mockup_path: None,
            scale_percent: 60.0,
            position_x: 50.0,
            position_y: 50.0,
            rotation_degrees: 0.0,
            shadow_enabled: true,
        }
    }
}

// ── Layout Spec ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSpec {
    pub headline_position: TextPosition,
    pub subheadline_position: TextPosition,
    pub cta_position: TextPosition,
    pub background_style: BackgroundStyle,
    pub color_scheme: ColorScheme,
    pub ratio_specs: Vec<RatioLayoutSpec>,
}

impl Default for LayoutSpec {
    fn default() -> Self {
        Self {
            headline_position: TextPosition::Top,
            subheadline_position: TextPosition::Middle,
            cta_position: TextPosition::Bottom,
            background_style: BackgroundStyle::Gradient,
            color_scheme: ColorScheme::Default,
            ratio_specs: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextPosition {
    Top,
    Middle,
    Bottom,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundStyle {
    Solid,
    Gradient,
    Image,
    Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Light,
    Dark,
    Vibrant,
    Monochrome,
}

// ── Ratio Layout Spec ───────────────────────────────────────────────

/// Per-ratio layout configuration with platform-specific positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioLayoutSpec {
    pub dimensions: String,
    pub headline_position: String,
    pub subheadline_position: String,
    pub cta_position: String,
    pub cta_badge: String,
    pub product_scale: String,
    pub product_position: String,
    pub layout_description: String,
}

impl RatioLayoutSpec {
    pub fn for_ratio(ratio: AspectRatio) -> Self {
        match ratio {
            AspectRatio::Square => Self {
                dimensions: "1080x1080".into(),
                headline_position: "top-center".into(),
                subheadline_position: "center".into(),
                cta_position: "bottom-right".into(),
                cta_badge: "bottom-right".into(),
                product_scale: "60%".into(),
                product_position: "center".into(),
                layout_description: "Balanced square layout with centered product, headline at top, CTA badge at bottom-right. Optimized for Instagram Feed and Facebook Feed.".into(),
            },
            AspectRatio::Story => Self {
                dimensions: "1080x1920".into(),
                headline_position: "top-third".into(),
                subheadline_position: "middle".into(),
                cta_position: "bottom-center-swipe".into(),
                cta_badge: "bottom-center-swipe".into(),
                product_scale: "75%".into(),
                product_position: "center-upper".into(),
                layout_description: "Full-screen vertical layout optimized for mobile: product occupies upper 75%, headline at top-third, swipe-CTA at bottom. Designed for TikTok, IG Reels, and Stories.".into(),
            },
            AspectRatio::Landscape => Self {
                dimensions: "1200x628".into(),
                headline_position: "left-half".into(),
                subheadline_position: "left-below-headline".into(),
                cta_position: "bottom-left".into(),
                cta_badge: "bottom-left".into(),
                product_scale: "50% right-half".into(),
                product_position: "right-half".into(),
                layout_description: "Widescreen layout: headline and copy on left half, product mockup on right half. Optimized for Facebook Link Ads, website banners, and YouTube thumbnails.".into(),
            },
            AspectRatio::Banner => Self {
                dimensions: "728x90".into(),
                headline_position: "left".into(),
                subheadline_position: "left-inline".into(),
                cta_position: "right".into(),
                cta_badge: "right".into(),
                product_scale: "40% right".into(),
                product_position: "right".into(),
                layout_description: "Wide horizontal banner: headline and CTA on the left, compact product thumbnail on the right. Optimized for website headers and display ad networks.".into(),
            },
            AspectRatio::MediumRectangle => Self {
                dimensions: "300x250".into(),
                headline_position: "top".into(),
                subheadline_position: "middle".into(),
                cta_position: "bottom".into(),
                cta_badge: "bottom".into(),
                product_scale: "50% center".into(),
                product_position: "center".into(),
                layout_description: "Compact square-adjacent format: headline at top, product centered, CTA strip at bottom. Optimized for in-content display ads and Google Display Network.".into(),
            },
            AspectRatio::Skyscraper => Self {
                dimensions: "160x600".into(),
                headline_position: "top".into(),
                subheadline_position: "upper-middle".into(),
                cta_position: "bottom".into(),
                cta_badge: "bottom".into(),
                product_scale: "45% middle".into(),
                product_position: "middle".into(),
                layout_description: "Tall vertical skyscraper: headline stacked at top, product in middle, CTA at bottom. Optimized for sidebar display ads.".into(),
            },
            AspectRatio::PodShape => Self {
                dimensions: "1080x1080".into(),
                headline_position: "top".into(),
                subheadline_position: "below-headline".into(),
                cta_position: "bottom".into(),
                cta_badge: "bottom".into(),
                product_scale: "70% center".into(),
                product_position: "center".into(),
                layout_description: "Custom POD shape (hexagon, circle badge, transparent cutout). Product isolated on transparent or shaped background. Optimized for print-on-demand marketplaces and mockups.".into(),
            },
        }
    }
}

// ── Copy Variation ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyVariation {
    pub id: usize,
    pub headline: String,
    pub subheadline: String,
    pub body_copy: String,
    pub call_to_action: String,
    pub framework: CopyFramework,
    pub conversion_score: u8,
}

// ── AI-Generated Concept ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: usize,
    pub name: String,
    pub visual_concept: String,
    pub copy_variations: Vec<CopyVariation>,
    pub color_scheme: ColorScheme,
    pub background_style: BackgroundStyle,
    pub conversion_score: u8,
    pub notes: String,
}

// ── Campaign ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub product_name: String,
    pub target_audience: String,
    pub platform: String,               // "facebook", "instagram", "google", "print", etc.
    pub adverts: Vec<Advert>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: CampaignStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignStatus {
    Draft,
    Active,
    Paused,
    Archived,
}

// ── AI Generation Config ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub brand_identity: BrandIdentity,
    pub aspect_ratios: Vec<AspectRatio>,
    pub copy_frameworks: Vec<CopyFramework>,
    pub num_variations: usize,
    pub include_visuals: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            brand_identity: BrandIdentity::default(),
            aspect_ratios: vec![AspectRatio::Square],
            copy_frameworks: vec![CopyFramework::Pas, CopyFramework::Bab],
            num_variations: 2,
            include_visuals: true,
        }
    }
}

// ── Export Format ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvertExport {
    pub advert: Advert,
    pub export_format: ExportFormat,
    pub exported_at: DateTime<Utc>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Png,
    Jpeg,
    Svg,
}
