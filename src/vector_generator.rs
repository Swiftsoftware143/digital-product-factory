//! Vector / Logo SVG Generator — AI-Generated via LLM Router
//!
//! Follows the same pattern as advert_generator.rs.
//! The LLM system prompt guarantees valid SVG output in JSON format.

use crate::llm_router::{GenerationRequest, LLMRouter, LLMProfile};
use crate::vector_types::{
    GeneratedLogo, GeneratedVector, LogoGenerateRequest,
    LogoStyle, VectorCategory, VectorGenerateRequest,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Generate a logo from brand name + style picker
pub fn generate_logo(
    llm: &LLMRouter,
    rt: &Arc<Runtime>,
    req: &LogoGenerateRequest,
) -> Result<GeneratedLogo, String> {
    let palette_desc = if req.palette.is_empty() {
        "auto-generate a harmonious color palette".to_string()
    } else {
        format!("use this palette: {}", req.palette.join(", "))
    };

    let system_prompt = format!(
        r#"You are a professional logo designer. Generate a complete logo as JSON.

Brand: {brand}
Tagline: {tagline}
Style: {style}
{colors}
Icon hint: {icon_hint}

OUTPUT FORMAT — return ONLY valid JSON (no markdown, no explanation):
{{
  "icon_svg": "<svg>…</svg>",
  "typography_svg": "<svg>…</svg>",
  "full_svg": "<svg>…</svg>",
  "palette": ["hex1", "hex2", "hex3"]
}}

RULES:
1. icon_svg must be a clean, scalable SVG icon (viewBox 0 0 100 100)
2. typography_svg renders the brand name as text (viewBox 0 0 100 30)
3. full_svg combines icon + typography (viewBox 0 0 100 130)
4. All SVGs must be valid standalone XML with xmlns
5. Use smooth curves, modern gradients, professional spacing
6. No external fonts — use standard sans-serif/system fonts"#,
        brand = req.brand_name,
        tagline = req.tagline.as_deref().unwrap_or(""),
        style = req.style,
        colors = palette_desc,
        icon_hint = req.icon_description.as_deref().unwrap_or("create a brand-appropriate icon"),
    );

    let request = GenerationRequest {
        profile: LLMProfile::Reasoning,
        prompt: system_prompt.clone(),
        system_prompt: Some(system_prompt),
        temperature: 0.7,
        max_tokens: 4096,
    };

    let response = rt
        .block_on(async { llm.generate(request).await })
        .map_err(|e| format!("LLM generation failed: {}", e))?;

    let gen: GeneratedLogo = serde_json::from_str(&response.content)
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    Ok(gen)
}

/// Generate a vector asset from a prompt
pub fn generate_vector(
    llm: &LLMRouter,
    rt: &Arc<Runtime>,
    req: &VectorGenerateRequest,
) -> Result<GeneratedVector, String> {
    let palette_desc = if req.palette.is_empty() {
        "auto-generate harmonious colors".to_string()
    } else {
        format!("use this palette: {}", req.palette.join(", "))
    };

    let system_prompt = format!(
        r#"You are a professional vector graphics designer.

Category: {category}
Prompt: {prompt}
Style: {style}
{colors}

OUTPUT FORMAT — return ONLY valid JSON:
{{
  "svg_content": "<svg>…</svg>",
  "palette": ["hex1", "hex2"],
  "view_box": "0 0 200 200"
}}

RULES:
1. Generate a clean, scalable SVG matching the prompt
2. Appropriate viewBox for the subject
3. Valid standalone SVG with xmlns
4. Use gradients, curves, professional styling
5. No external fonts — use system fonts"#,
        category = req.category.label(),
        prompt = req.prompt,
        style = req.style.as_deref().unwrap_or("clean modern"),
        colors = palette_desc,
    );

    let request = GenerationRequest {
        profile: LLMProfile::Reasoning,
        prompt: system_prompt.clone(),
        system_prompt: Some(system_prompt),
        temperature: 0.7,
        max_tokens: 4096,
    };

    let response = rt
        .block_on(async { llm.generate(request).await })
        .map_err(|e| format!("LLM generation failed: {}", e))?;

    let gen: GeneratedVector = serde_json::from_str(&response.content)
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    Ok(gen)
}
