//! Multi-LLM router for AI generation

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct LLMRouter {
    client: Client,
    openai_key: String,
    anthropic_key: String,
    google_key: String,
    deepseek_key: String,
    moonshot_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LLMProfile {
    Creative,      // GPT-4o for creative writing
    Structured,    // Claude for structured data
    Professional,  // GPT for professional tone
    Technical,     // Gemini for technical content
    Visual,        // GPT-4o Vision for image analysis
    Fast,          // small/cheap model for quick tasks
    Logic,         // DeepSeek for logical tasks
    Reasoning,     // DeepSeek for reasoning tasks
    Chinese,       // Moonshot for Chinese content
    Strategic,     // deep-thinking layer: product/market judgement (see docs)
}

impl LLMProfile {
    pub fn model(&self) -> &'static str {
        match self {
            LLMProfile::Creative => "gpt-4o",
            LLMProfile::Structured => "claude-sonnet-4-5",
            LLMProfile::Professional => "gpt-4o",
            LLMProfile::Technical => "gemini-2.0-flash",
            LLMProfile::Visual => "gpt-4o",
            LLMProfile::Fast => "gpt-4o-mini",
            LLMProfile::Logic | LLMProfile::Reasoning => "deepseek-chat",
            LLMProfile::Chinese => "moonshot-v1-8k",
            // Strategic resolves its model from the CHOSEN provider at call time
            // (see `strategic_model`) because the user may hold any one of five keys.
            // This value is only a last-resort default.
            LLMProfile::Strategic => "claude-sonnet-4-5",
        }
    }

    pub fn provider(&self) -> Provider {
        match self {
            LLMProfile::Creative | LLMProfile::Professional | LLMProfile::Visual | LLMProfile::Fast => Provider::OpenAI,
            LLMProfile::Structured => Provider::Anthropic,
            LLMProfile::Technical => Provider::Google,
            LLMProfile::Logic | LLMProfile::Reasoning => Provider::DeepSeek,
            LLMProfile::Chinese => Provider::Moonshot,
            // Preference order only. The router picks the strongest provider the user
            // actually has a key for; see `best_strategic_provider`.
            LLMProfile::Strategic => Provider::Anthropic,
        }
    }

    /// Model to use for a strategic brief on a given provider.
    ///
    /// Strategic work is judgement, not volume, so each provider gets its STRONGEST
    /// general model rather than a cheap one.
    pub fn strategic_model(provider: Provider) -> &'static str {
        match provider {
            Provider::Anthropic => "claude-sonnet-4-5",
            Provider::DeepSeek => "deepseek-reasoner",
            Provider::OpenAI => "gpt-4o",
            Provider::Google => "gemini-2.0-flash",
            Provider::Moonshot => "moonshot-v1-32k",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    DeepSeek,
    Moonshot,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic (Claude)",
            Provider::Google => "Google (Gemini)",
            Provider::DeepSeek => "DeepSeek",
            Provider::Moonshot => "Moonshot",
        }
    }

    /// Stable id used in the saved config (lowercase, no spaces).
    pub fn config_id(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Google => "google",
            Provider::DeepSeek => "deepseek",
            Provider::Moonshot => "moonshot",
        }
    }

    /// Parse the saved config value. Unknown or empty means "auto" (None) rather than an
    /// error, so a config written by a future/older version can never hard-fail the app.
    pub fn from_config_str(s: &str) -> Option<Provider> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Provider::OpenAI),
            "anthropic" | "claude" => Some(Provider::Anthropic),
            "google" | "gemini" => Some(Provider::Google),
            "deepseek" => Some(Provider::DeepSeek),
            "moonshot" => Some(Provider::Moonshot),
            _ => None,
        }
    }

    /// Suggested model ids shown in the picker. Purely a convenience list — the model field
    /// accepts any id the user types, so a retired or brand-new model is never a blocker.
    pub fn suggested_models(&self) -> &'static [&'static str] {
        match self {
            Provider::Anthropic => &["claude-sonnet-4-5", "claude-opus-4-1", "claude-haiku-4-5"],
            Provider::DeepSeek => &["deepseek-reasoner", "deepseek-chat"],
            Provider::OpenAI => &["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
            Provider::Google => &["gemini-2.0-flash", "gemini-1.5-pro"],
            Provider::Moonshot => &["moonshot-v1-32k", "moonshot-v1-8k"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub profile: LLMProfile,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: u32,
}

impl LLMRouter {
    pub fn new(openai_key: String, anthropic_key: String, google_key: String, deepseek_key: String, moonshot_key: String) -> Self {
        Self {
            client: Client::new(),
            openai_key,
            anthropic_key,
            google_key,
            deepseek_key,
            moonshot_key,
        }
    }

    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, String> {
        // The provider and model are normally fixed by the profile. Strategic is the one
        // exception: it is the deep-thinking layer and must work with WHICHEVER key the user
        // holds, so both are resolved from the keys actually configured.
        let provider = match request.profile {
            LLMProfile::Strategic => self.best_strategic_provider().ok_or_else(|| {
                "Strategy needs an AI provider key. Add one in Settings, then try again.".to_string()
            })?,
            _ => request.profile.provider(),
        };
        let model: &str = match request.profile {
            LLMProfile::Strategic => LLMProfile::strategic_model(provider),
            _ => request.profile.model(),
        };

        match provider {
            Provider::OpenAI => self.call_openai(request, model).await,
            Provider::Anthropic => self.call_anthropic(request, model).await,
            Provider::Google => self.call_google(request, model).await,
            Provider::DeepSeek => self.call_deepseek(request, model).await,
            Provider::Moonshot => self.call_moonshot(request, model).await,
        }
    }

    /// Does the user hold a key for this provider?
    pub fn has_key(&self, p: Provider) -> bool {
        let k = match p {
            Provider::OpenAI => &self.openai_key,
            Provider::Anthropic => &self.anthropic_key,
            Provider::Google => &self.google_key,
            Provider::DeepSeek => &self.deepseek_key,
            Provider::Moonshot => &self.moonshot_key,
        };
        !k.trim().is_empty()
    }

    /// Providers the user can actually use, strongest-for-reasoning first.
    pub fn available_providers(&self) -> Vec<Provider> {
        [Provider::Anthropic, Provider::DeepSeek, Provider::OpenAI, Provider::Google, Provider::Moonshot]
            .into_iter()
            .filter(|p| self.has_key(*p))
            .collect()
    }

    /// Best provider for a strategic brief, or None if no key is configured.
    pub fn best_strategic_provider(&self) -> Option<Provider> {
        self.available_providers().into_iter().next()
    }

    /// Run a strategic product/market brief.
    ///
    /// `preferred` and `model` come straight from the user's settings and ALWAYS win if usable.
    /// Empty/Nones mean auto-select. The app is deliberately never locked to one model id:
    /// providers retire model strings, and the user must be able to move to a new one without
    /// waiting for an app update — hence `model` is free text, not an enum.
    pub async fn generate_strategy(
        &self,
        preferred: Option<Provider>,
        model: Option<&str>,
        system_prompt: String,
        prompt: String,
        max_tokens: u32,
    ) -> Result<GenerationResponse, String> {
        let provider = match preferred {
            // An explicit choice wins — but a choice without a key is an honest error, not a
            // silent fallback to a different provider (that would spend the wrong money).
            Some(p) if self.has_key(p) => p,
            Some(p) => {
                return Err(format!(
                    "No API key is configured for {}. Add one in Settings, or set Strategy to Auto.",
                    p.label()
                ))
            }
            None => self.best_strategic_provider().ok_or_else(|| {
                "Strategy needs an AI provider key. Add one in Settings, then try again.".to_string()
            })?,
        };

        let model: &str = match model {
            Some(m) if !m.trim().is_empty() => m.trim(),
            _ => LLMProfile::strategic_model(provider),
        };

        // Low temperature: a brief is judgement, not prose. Deterministic > creative here.
        let request = GenerationRequest {
            profile: LLMProfile::Strategic,
            prompt,
            system_prompt: Some(system_prompt),
            temperature: 0.4,
            max_tokens,
        };

        match provider {
            Provider::OpenAI => self.call_openai(request, model).await,
            Provider::Anthropic => self.call_anthropic(request, model).await,
            Provider::Google => self.call_google(request, model).await,
            Provider::DeepSeek => self.call_deepseek(request, model).await,
            Provider::Moonshot => self.call_moonshot(request, model).await,
        }
    }

    async fn call_openai(&self, request: GenerationRequest, model: &str) -> Result<GenerationResponse, String> {
        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            "You are a helpful assistant for creating digital products.".to_string()
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": request.prompt}
                ],
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
            }))
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        let tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(GenerationResponse {
            content,
            model: model.to_string(),
            tokens_used: tokens,
        })
    }

    async fn call_anthropic(&self, request: GenerationRequest, model: &str) -> Result<GenerationResponse, String> {
        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            "You are a helpful assistant for creating digital products.".to_string()
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.anthropic_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": model,
                "max_tokens": request.max_tokens,
                "system": system_prompt,
                "messages": [
                    {"role": "user", "content": request.prompt}
                ],
                "temperature": request.temperature,
            }))
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        let tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32 +
                     json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(GenerationResponse {
            content,
            model: model.to_string(),
            tokens_used: tokens,
        })
    }

    async fn call_google(&self, request: GenerationRequest, model: &str) -> Result<GenerationResponse, String> {
        // Google Gemini API implementation
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model,
            self.google_key
        );

        let response = self.client
            .post(&url)
            .json(&json!({
                "contents": [{
                    "parts": [{"text": request.prompt}]
                }],
                "generationConfig": {
                    "temperature": request.temperature,
                    "maxOutputTokens": request.max_tokens,
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Google API request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Google response: {}", e))?;

        let content = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        Ok(GenerationResponse {
            content,
            model: model.to_string(),
            tokens_used: 0, // Google doesn't always return token counts
        })
    }

    async fn call_deepseek(&self, request: GenerationRequest, model: &str) -> Result<GenerationResponse, String> {
        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            "You are a helpful assistant for creating digital products.".to_string()
        });

        let response = self.client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.deepseek_key))
            .json(&json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": request.prompt}
                ],
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
            }))
            .send()
            .await
            .map_err(|e| format!("DeepSeek request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse DeepSeek response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        let tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(GenerationResponse {
            content,
            model: model.to_string(),
            tokens_used: tokens,
        })
    }

    async fn call_moonshot(&self, request: GenerationRequest, model: &str) -> Result<GenerationResponse, String> {
        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            "You are a helpful assistant for creating digital products.".to_string()
        });

        let response = self.client
            .post("https://api.moonshot.cn/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.moonshot_key))
            .json(&json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": request.prompt}
                ],
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
            }))
            .send()
            .await
            .map_err(|e| format!("Moonshot request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Moonshot response: {}", e))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        let tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(GenerationResponse {
            content,
            model: model.to_string(),
            tokens_used: tokens,
        })
    }

    /// Auto-select best profile based on task
    pub fn auto_select_profile(task: &str) -> LLMProfile {
        let task_lower = task.to_lowercase();

        if task_lower.contains("creative") || task_lower.contains("write") || task_lower.contains("story") {
            LLMProfile::Creative
        } else if task_lower.contains("structure") || task_lower.contains("data") || task_lower.contains("json") {
            LLMProfile::Structured
        } else if task_lower.contains("technical") || task_lower.contains("code") {
            LLMProfile::Technical
        } else if task_lower.contains("professional") || task_lower.contains("business") {
            LLMProfile::Professional
        } else if task_lower.contains("image") || task_lower.contains("visual") {
            LLMProfile::Visual
        } else if task_lower.contains("logic") || task_lower.contains("reasoning") || task_lower.contains("analysis") {
            LLMProfile::Logic
        } else if task_lower.contains("chinese") || task_lower.contains("cn") {
            LLMProfile::Chinese
        } else {
            LLMProfile::Fast
        }
    }
}
