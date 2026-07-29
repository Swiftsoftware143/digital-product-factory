//! Product generation engine with AI integration

use crate::database::Database;
use crate::llm_router::{LLMRouter, LLMProfile, GenerationRequest};
use crate::templates::{Template, TemplateRegistry, OutputFormat};
use std::sync::Arc;
use tokio::runtime::Runtime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GeneratedProduct {
    pub id: usize,
    pub name: String,
    pub template_id: String,
    pub content: String,
    pub format: OutputFormat,
    pub created_at: chrono::DateTime<Utc>,
    pub metadata: ProductMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProductMetadata {
    pub model_used: String,
    pub tokens_used: u32,
    pub generation_time_ms: u64,
    pub parameters: serde_json::Value,
}

pub struct ProductGenerator {
    db: Arc<Database>,
    llm_router: Option<LLMRouter>,
    template_registry: TemplateRegistry,
    runtime: Arc<Runtime>,
}

impl ProductGenerator {
    pub fn new(db: &Arc<Database>, runtime: Arc<Runtime>) -> Self {
        Self {
            db: db.clone(),
            llm_router: None,
            template_registry: TemplateRegistry::new(),
            runtime,
        }
    }

    pub fn set_api_keys(&mut self, openai: String, anthropic: String, google: String, deepseek: String, moonshot: String) {
        self.llm_router = Some(LLMRouter::new(openai, anthropic, google, deepseek, moonshot));
    }

    pub fn generate_blocking(
        &self,
        template_id: &str,
        params: serde_json::Value,
    ) -> Result<GeneratedProduct, String> {
        let router = self.llm_router.as_ref()
            .ok_or("API keys not configured")?;

        let template = self.template_registry.get(template_id)
            .ok_or(format!("Template '{}' not found", template_id))?;

        // Build prompt from template
        let prompt = self.build_prompt(template, &params)?;

        // Auto-select best LLM profile
        let profile = LLMProfile::Creative; // Could auto-select based on template

        let request = GenerationRequest {
            profile,
            prompt,
            system_prompt: Some("You are an expert digital product creator. Generate high-quality, professional content.".to_string()),
            temperature: 0.7,
            max_tokens: 4000,
        };

        let start = std::time::Instant::now();

        // Run async generation in blocking context
        let response = self.runtime.block_on(async {
            router.generate(request).await
        })?;

        let generation_time = start.elapsed().as_millis() as u64;

        let product = GeneratedProduct {
            id: 0, // Will be set by database
            name: self.extract_product_name(&response.content, template),
            template_id: template_id.to_string(),
            content: response.content,
            format: template.output_format,
            created_at: Utc::now(),
            metadata: ProductMetadata {
                model_used: response.model,
                tokens_used: response.tokens_used,
                generation_time_ms: generation_time,
                parameters: params,
            },
        };

        Ok(product)
    }

    fn build_prompt(&self, template: &Template, params: &serde_json::Value) -> Result<String, String> {
        let mut prompt = template.prompt_template.clone();

        // Replace parameters in template
        if let Some(obj) = params.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{}}}", key);
                let fallback = value.to_string();
                let replacement = value.as_str().unwrap_or(&fallback);
                prompt = prompt.replace(&placeholder, replacement);
            }
        }

        // Check for unreplaced placeholders
        if prompt.contains('{') && prompt.contains('}') {
            // Fill with defaults or error
            for param in &template.parameters {
                let placeholder = format!("{{{}}}", param.name);
                if prompt.contains(&placeholder) {
                    if let Some(default) = &param.default {
                        prompt = prompt.replace(&placeholder, default);
                    } else if param.required {
                        return Err(format!("Required parameter '{}' not provided", param.name));
                    }
                }
            }
        }

        Ok(prompt)
    }

    fn extract_product_name(&self, content: &str, template: &Template) -> String {
        // Try to extract title from first line or use template name
        let first_line = content.lines().next().unwrap_or("");
        if first_line.starts_with("# ") {
            first_line.strip_prefix("# ").unwrap_or("").trim().to_string()
        } else if first_line.starts_with("Title:") {
            first_line.strip_prefix("Title:").unwrap_or("").trim().to_string()
        } else {
            format!("{} - {}", template.name, chrono::Local::now().format("%Y-%m-%d"))
        }
    }

    pub fn get_template_registry(&self) -> &TemplateRegistry {
        &self.template_registry
    }

    pub fn preview_template(&self, template_id: &str, params: &serde_json::Value) -> Result<String, String> {
        let template = self.template_registry.get(template_id)
            .ok_or("Template not found")?;

        self.build_prompt(template, params)
    }
}
