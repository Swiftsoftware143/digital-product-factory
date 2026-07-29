//! Domain types for Logo Generator & Vector Generator modules.
//! Mirrors the pattern in adverts.rs

use serde::{Deserialize, Serialize};

// ── Logo ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogoStyle {
    Minimal,
    Modern,
    Vintage,
    Playful,
    Corporate,
    Tech,
    HandDrawn,
}

impl LogoStyle {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Minimal, Self::Modern, Self::Vintage, Self::Playful,
            Self::Corporate, Self::Tech, Self::HandDrawn,
        ]
    }
    pub fn label(&self) -> &str {
        match self {
            Self::Minimal => "Minimal",
            Self::Modern => "Modern",
            Self::Vintage => "Vintage",
            Self::Playful => "Playful",
            Self::Corporate => "Corporate",
            Self::Tech => "Tech",
            Self::HandDrawn => "Hand Drawn",
        }
    }
}

impl std::fmt::Display for LogoStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logo {
    pub id: String,
    pub name: String,
    pub style: LogoStyle,
    pub brand_name: String,
    pub tagline: String,
    pub icon_svg: String,
    pub typography_svg: String,
    pub full_svg: String,
    pub palette: Vec<String>,
    pub favicon_enabled: bool,
    pub favicon_package: Option<FaviconPackage>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconPackage {
    pub enabled: bool,
    pub sizes: Vec<(u32, u32)>,
    pub apple_touch_icon: Option<String>,
    pub ico_file: Option<Vec<u8>>,
    pub webmanifest_json: Option<String>,
}

// ── Vector Asset ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VectorCategory {
    Icon,
    Illustration,
    Badge,
    Pattern,
    Decorative,
    Infographic,
    UiElement,
}

impl VectorCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Icon, Self::Illustration, Self::Badge, Self::Pattern,
            Self::Decorative, Self::Infographic, Self::UiElement,
        ]
    }
    pub fn label(&self) -> &str {
        match self {
            Self::Icon => "Icon",
            Self::Illustration => "Illustration",
            Self::Badge => "Badge",
            Self::Pattern => "Pattern",
            Self::Decorative => "Decorative",
            Self::Infographic => "Infographic",
            Self::UiElement => "UI Element",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorAsset {
    pub id: String,
    pub name: String,
    pub category: VectorCategory,
    pub prompt: String,
    pub svg_content: String,
    pub palette: Vec<String>,
    pub view_box: String,
    pub export_formats: Vec<String>,
    pub status: String,
    pub created_at: String,
}

// ── Generation Requests ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoGenerateRequest {
    pub brand_name: String,
    pub tagline: Option<String>,
    pub style: LogoStyle,
    pub palette: Vec<String>,
    pub icon_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorGenerateRequest {
    pub category: VectorCategory,
    pub prompt: String,
    pub style: Option<String>,
    pub palette: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedLogo {
    pub icon_svg: String,
    pub typography_svg: String,
    pub full_svg: String,
    pub palette: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedVector {
    pub svg_content: String,
    pub palette: Vec<String>,
    pub view_box: String,
}
