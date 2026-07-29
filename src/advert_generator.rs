//! Advert Generator — AI-Generated via LLM Router
//!
//! All ad copy, visual concepts, layout specs, and conversion scores
//! are produced by the LLM router. Nothing hardcoded.
//!
//! The LLM uses a structured system prompt that guarantees JSON output
//! conforming to the Advert data model. Parsed with serde_json.

use crate::adverts::{
    Advert, AdvertStatus, AspectRatio, BackgroundStyle, BrandIdentity,
    Campaign, CampaignStatus, ColorScheme, CopyFramework, CopyVariation,
    GenerationConfig, LayoutSpec, ProductPlacement, RatioLayoutSpec,
};
use crate::llm_router::{GenerationRequest, LLMProfile, LLMRouter};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;

const ADVERTS_SYSTEM_PROMPT: &str = r#"You are the Adverts Module inside Digital Product Factory, an AI-powered desktop application built to generate high-converting advertising creatives, product photography scenes, copy, and multi-format promotional layouts for e-commerce, POD (Print on Demand), and digital products.

## Core Functionality & Objectives
- Generate advertising assets for 7 formats: Square (1:1, 1080x1080), Story (9:16, 1080x1920), Landscape (16:9, 1200x628), Banner (728x90), Medium Rectangle (300x250), Skyscraper (160x600), and POD Custom Shape (hexagon, circle badge, transparent cutout).
- Create AI product photography descriptions with background scenes (e.g., rustic wood, minimalist platform, neon diner, natural sunlight)
- Apply proven conversion frameworks: PAS (Problem-Agitate-Solve), AIDA (Attention-Interest-Desire-Action), BAB (Before-After-Bridge)
- Provide a projected Conversion Score (0-100) for each concept with reasoning

## Input Provided
- Product name, category, and target audience
- Brand identity (name, tagline, voice tone, hex colors, font family)
- Selected aspect ratios and copy frameworks
- Number of variations to generate

## Output Format
Return ONLY valid JSON. No markdown, no code fences. The JSON structure must be:
{
  "adverts": [
    {
      "aspect_ratio": "square_1_1" | "story_9_16" | "landscape_16_9" | "banner" | "medium_rectangle" | "skyscraper" | "pod_shape",
      "headline": "string",
      "subheadline": "string",
      "body_copy": "string (multi-line with \n for line breaks)",
      "call_to_action": "string",
      "copy_framework": "PAS" | "AIDA" | "BAB",
      "conversion_score": 0-100,
      "score_reasoning": "string (why this score, what's working, what could improve)",
      "visual_concept": {
        "concept_name": "string",
        "background_description": "string (detailed scene prompt)",
        "color_scheme": "Light" | "Dark" | "Vibrant" | "Monochrome" | "Default",
        "background_style": "Gradient" | "Solid" | "Image" | "Pattern"
      },
      "layout_spec": {
        "dimensions": "1080x1080" | "1080x1920" | "1200x628",
        "headline_position": "string",
        "product_scale": "string",
        "cta_badge": "string",
        "platform_recommendation": "string"
      }
    }
  ],
  "brand_identity": {
    "extracted_tone": "string",
    "recommended_palette": ["hex1", "hex2", "hex3"],
    "target_platforms": ["string"]
  }
}

## Rules
- Always output all requested aspect ratios with tailored positioning
- Maintain brand colors and typography in every layout
- Prioritize readability, high visual contrast, clear product placement, bold CTAs
- Conversion score must include reasoning
- Visual concepts must be unique across variations - no duplicates
- Use the provided brand voice_tone consistently in all copy"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmAdvertEntry {
    aspect_ratio: String,
    headline: String,
    subheadline: String,
    body_copy: String,
    call_to_action: String,
    copy_framework: String,
    conversion_score: u8,
    score_reasoning: String,
    visual_concept: LlmVisualConcept,
    layout_spec: LlmLayoutSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmVisualConcept {
    concept_name: String,
    background_description: String,
    color_scheme: String,
    background_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmLayoutSpec {
    dimensions: String,
    headline_position: String,
    product_scale: String,
    cta_badge: String,
    platform_recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmBrandIdentity {
    extracted_tone: String,
    recommended_palette: Vec<String>,
    target_platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmAdvertResponse {
    adverts: Vec<LlmAdvertEntry>,
    brand_identity: LlmBrandIdentity,
}

pub struct AdvertGenerator {
    llm_router: Option<LLMRouter>,
}

impl AdvertGenerator {
    pub fn new() -> Self {
        Self { llm_router: None }
    }

    pub fn set_api_keys(
        &mut self,
        openai: String,
        anthropic: String,
        google: String,
        deepseek: String,
        moonshot: String,
    ) {
        self.llm_router = Some(LLMRouter::new(
            openai, anthropic, google, deepseek, moonshot,
        ));
    }

    pub fn generate_campaign(
        &self,
        config: &GenerationConfig,
        campaign_name: &str,
        product_name: &str,
        target_audience: &str,
        platform: &str,
        router: Option<&LLMRouter>,
        runtime: &Arc<Runtime>,
    ) -> Result<Campaign, String> {
        let llm = router.or(self.llm_router.as_ref())
            .ok_or("LLM router not configured - set API keys first")?;

        let frameworks_str = config
            .copy_frameworks
            .iter()
            .map(|f| f.label())
            .collect::<Vec<_>>()
            .join(", ");
        let ratios_str = config
            .aspect_ratios
            .iter()
            .map(|a| a.label())
            .collect::<Vec<_>>()
            .join(", ");

        let brand = &config.brand_identity;
        let user_prompt = format!(
            "Generate {} advert variation(s) for:\n\
             Product name: {}\n\
             Target audience: {}\n\
             Platform: {}\n\
             Brand name: {}\n\
             Tagline: {}\n\
             Voice tone: {}\n\
             Colors: primary={}, secondary={}, accent={}\n\
             Font: {}\n\
             Copy frameworks requested: {}\n\
             Aspect ratios requested: {}\n\n\
             Output exactly {} advert(s), distributing across the requested aspect ratios \
             and copy frameworks. Each advert must be unique.",
            config.num_variations,
            product_name,
            target_audience,
            platform,
            brand.brand_name,
            brand.tagline,
            brand.voice_tone,
            brand.primary_color,
            brand.secondary_color,
            brand.accent_color,
            brand.font_family,
            frameworks_str,
            ratios_str,
            config.num_variations,
        );

        let request = GenerationRequest {
            profile: LLMProfile::Creative,
            prompt: user_prompt,
            system_prompt: Some(ADVERTS_SYSTEM_PROMPT.to_string()),
            temperature: 0.8,
            max_tokens: 4096,
        };

        let response = runtime
            .block_on(async { llm.generate(request).await })
            .map_err(|e| format!("LLM generation failed: {}", e))?;

        let parsed: LlmAdvertResponse = serde_json::from_str(&response.content)
            .map_err(|e| format!("Failed to parse LLM response as JSON: {}\nRaw: {}", e, response.content))?;

        let mut adverts: Vec<Advert> = Vec::new();
        for (i, entry) in parsed.adverts.into_iter().enumerate() {
            let aspect_ratio = parse_aspect_ratio(&entry.aspect_ratio);
            let framework = parse_framework(&entry.copy_framework);
            let color_scheme = parse_color_scheme(&entry.visual_concept.color_scheme);
            let background_style = parse_background_style(&entry.visual_concept.background_style);
            let ratio_spec = RatioLayoutSpec::for_ratio(aspect_ratio);

            let advert = Advert {
                id: i + 1,
                campaign_id: 0,
                name: format!(
                    "{} - {} - V{}",
                    campaign_name,
                    aspect_ratio.label(),
                    i + 1
                ),
                aspect_ratio,
                headline: entry.headline,
                subheadline: entry.subheadline,
                body_copy: entry.body_copy,
                call_to_action: entry.call_to_action,
                visual_description: format!(
                    "{}\n\nBackground: {}\nConcept: {}",
                    entry.visual_concept.concept_name,
                    entry.visual_concept.background_description,
                    entry.score_reasoning,
                ),
                brand_identity: config.brand_identity.clone(),
                conversion_score: entry.conversion_score,
                copy_framework: framework,
                product_placement: ProductPlacement {
                    product_name: product_name.to_string(),
                    ..Default::default()
                },
                layout_spec: LayoutSpec {
                    color_scheme,
                    background_style,
                    ratio_specs: vec![ratio_spec],
                    ..Default::default()
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                status: AdvertStatus::Generated,
            };
            adverts.push(advert);
        }

        if adverts.is_empty() {
            return Err("LLM returned zero adverts".to_string());
        }

        Ok(Campaign {
            id: 1,
            name: campaign_name.to_string(),
            description: format!("AI-generated campaign for {}", product_name),
            product_name: product_name.to_string(),
            target_audience: target_audience.to_string(),
            platform: platform.to_string(),
            adverts,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: CampaignStatus::Draft,
        })
    }

    pub fn score_advert(&self, advert: &Advert) -> u8 {
        advert.conversion_score
    }
}

fn parse_aspect_ratio(s: &str) -> AspectRatio {
    match s {
        "story_9_16" | "9:16" | "9:16 Story" | "story" => AspectRatio::Story,
        "landscape_16_9" | "16:9" | "16:9 Landscape" | "landscape" => AspectRatio::Landscape,
        "banner" | "728x90" | "728x90 Banner" => AspectRatio::Banner,
        "medium_rectangle" | "300x250" | "300x250 Medium Rectangle" => AspectRatio::MediumRectangle,
        "skyscraper" | "160x600" | "160x600 Skyscraper" => AspectRatio::Skyscraper,
        "pod_shape" | "POD Custom Shape" => AspectRatio::PodShape,
        _ => AspectRatio::Square,
    }
}

fn parse_framework(s: &str) -> CopyFramework {
    match s.to_uppercase().as_str() {
        "AIDA" | "AIDA (ATTENTION-INTEREST-DESIRE-ACTION)" => CopyFramework::Aida,
        "BAB" | "BAB (BEFORE-AFTER-BRIDGE)" => CopyFramework::Bab,
        _ => CopyFramework::Pas,
    }
}

fn parse_color_scheme(s: &str) -> ColorScheme {
    match s {
        "Light" => ColorScheme::Light,
        "Dark" => ColorScheme::Dark,
        "Vibrant" => ColorScheme::Vibrant,
        "Monochrome" => ColorScheme::Monochrome,
        _ => ColorScheme::Default,
    }
}

fn parse_background_style(s: &str) -> BackgroundStyle {
    match s {
        "Solid" => BackgroundStyle::Solid,
        "Gradient" => BackgroundStyle::Gradient,
        "Image" => BackgroundStyle::Image,
        _ => BackgroundStyle::Pattern,
    }
}
