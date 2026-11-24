//! The engine behind the door.
//!
//! Every tool here calls the SAME module function the GUI calls. Nothing is re-implemented for the
//! headless path:
//!
//! | tool                    | the call the GUI makes                          |
//! |-------------------------|-------------------------------------------------|
//! | `dpf_list_products`     | `Database::load_ideas` (what `Pipeline::load` uses) |
//! | `dpf_get_product`       | `Database::load_ideas`                          |
//! | `dpf_create_product`    | `Database::save_idea` (what `Pipeline::add_idea` uses) |
//! | `dpf_list_templates`    | `TemplateRegistry::new()` (the Create screen's registry) |
//! | `dpf_generate_content`  | `ProductGenerator::generate` (the Create screen's generator) |
//! | `dpf_strategy_brief`    | `strategy::run_async` (the Strategy panel's one call) |
//!
//! What is deliberately NOT here: publishing/listing, licence activation, payment or checkout,
//! sale recording, and anything that delivers a paid product. Those stay off the agent's surface
//! until the payment hold (`t_90a8f547`) lifts, and the README says so.

use crate::config::AppConfig;
use crate::database::Database;
use crate::llm_router::{LLMRouter, Provider};
use crate::pipeline::{PipelineStage, Priority, ProductIdea};
use crate::product_generator::ProductGenerator;
use crate::strategy::{self, StrategyRequest};
use crate::templates::TemplateRegistry;
use chrono::Utc;
use mcp_door::tool::BoxFut;
use mcp_door::{Ctx, Registry, ToolError, ToolSpec};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct Engine {
    db: Arc<Database>,
    generator: ProductGenerator,
    router: LLMRouter,
    config: AppConfig,
}

impl Engine {
    pub fn new(
        db: Arc<Database>,
        runtime: Arc<tokio::runtime::Runtime>,
        config: AppConfig,
    ) -> Self {
        let mut generator = ProductGenerator::new(&db, runtime);
        generator.set_api_keys(
            config.openai_key.clone(),
            config.anthropic_key.clone(),
            config.google_key.clone(),
            config.deepseek_key.clone(),
            config.moonshot_key.clone(),
        );
        let router = LLMRouter::new(
            config.openai_key.clone(),
            config.anthropic_key.clone(),
            config.google_key.clone(),
            config.deepseek_key.clone(),
            config.moonshot_key.clone(),
        );
        Self {
            db,
            generator,
            router,
            config,
        }
    }

    /// Build the tool table. Startup validation (mcp-door) refuses a door with a bad name,
    /// a short description or a non-object schema, so this cannot ship half-annotated.
    pub fn registry(self: &Arc<Self>) -> Registry {
        let mut reg = Registry::new();

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::read(
                "dpf_list_products",
                "List the user's products, newest activity first. A product in DPF is a pipeline \
                 project: title, stage, priority, values and notes. Optional `stage` filters to \
                 one pipeline column.",
            )
            .title("List products")
            .schema(json!({
                "type": "object",
                "properties": {
                    "stage": {
                        "type": "string",
                        "enum": ["idea","research","creating","review","listed","selling","archived"],
                        "description": "Only products in this pipeline stage."
                    },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200 },
                    "offset": { "type": "integer", "minimum": 0 }
                },
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.list_products(&ctx, &args) })
            },
        ));

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::read(
                "dpf_get_product",
                "Read one product in full by its id: every stored field including the notes, which \
                 is where generated content is kept.",
            )
            .title("Read a product")
            .schema(json!({
                "type": "object",
                "properties": { "id": { "type": "integer", "minimum": 0 } },
                "required": ["id"],
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.get_product(&ctx, &args) })
            },
        ));

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::write(
                "dpf_create_product",
                "Create a product (pipeline project) from a title and optional details, then read \
                 it back from the database. Writes a real row; it does not publish or charge \
                 anything. Answers with the stored record and its new id.",
            )
            .title("Create a product")
            .schema(json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Product name." },
                    "description": { "type": "string" },
                    "product_type": { "type": "string", "description": "e.g. the template id, or a free label." },
                    "stage": {
                        "type": "string",
                        "enum": ["idea","research","creating","review","listed","selling","archived"],
                        "description": "Defaults to idea."
                    },
                    "priority": { "type": "string", "enum": ["low","medium","high","urgent"] },
                    "estimated_value": { "type": "number" },
                    "tags": { "type": "array", "items": { "type": "string" } },
                    "platform": { "type": "array", "items": { "type": "string" } },
                    "notes": { "type": "string" }
                },
                "required": ["title"],
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.create_product(&ctx, &args) })
            },
        ));

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::read(
                "dpf_list_templates",
                "List the built-in product templates the Create screen offers, with each template's \
                 id and parameters. Pass a template id to `dpf_generate_content`.",
            )
            .title("List templates")
            .schema(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Filter by name, description or tag." },
                    "category": { "type": "string", "description": "Filter by category name, e.g. Planner." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200 }
                },
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.list_templates(&ctx, &args) })
            },
        ));

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::write(
                "dpf_generate_content",
                "Generate product content with the USER'S OWN AI key (BYOK) through the same \
                 generator the Create screen uses. Spends their provider credit, so it is marked \
                 as a write and a host will ask first. Set `save_as_product` to store the result \
                 as a pipeline product you can read back with dpf_get_product.",
            )
            .title("Generate product content")
            .schema(json!({
                "type": "object",
                "properties": {
                    "template_id": { "type": "string", "description": "From dpf_list_templates." },
                    "params": { "type": "object", "description": "Values for the template's parameters." },
                    "prompt": { "type": "string", "description": "Optional prompt that replaces the template's own wording." },
                    "save_as_product": { "type": "boolean", "description": "Save the result as a product (default false)." },
                    "title": { "type": "string", "description": "Title for the saved product; defaults to the generated name." }
                },
                "required": ["template_id"],
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.generate_content(&ctx, &args).await })
            },
        ));

        let eng = Arc::clone(self);
        reg.register(mcp_door::tool_fn(
            ToolSpec::write(
                "dpf_strategy_brief",
                "Produce the deep-thinking strategy brief for a niche (what to build, pricing, and \
                 one explicit do-not-build) with the user's own key. One model call, spent from \
                 their credit, so it is marked as a write.",
            )
            .title("Strategy brief")
            .schema(json!({
                "type": "object",
                "properties": {
                    "niche": { "type": "string", "description": "The market to reason about." },
                    "existing_products": { "type": "string" },
                    "price_positioning": { "type": "string" },
                    "provider": { "type": "string", "description": "anthropic|deepseek|openai|google|moonshot; default auto." },
                    "model": { "type": "string", "description": "Any model id; default the provider's." }
                },
                "required": ["niche"],
                "additionalProperties": false
            })),
            move |ctx: Ctx, args: Value| -> BoxFut {
                let eng = Arc::clone(&eng);
                Box::pin(async move { eng.strategy_brief(&ctx, &args).await })
            },
        ));

        reg
    }

    fn db_err(e: rusqlite::Error) -> ToolError {
        ToolError::Internal(format!("sqlite: {e}"))
    }

    fn load_all(&self) -> Result<Vec<ProductIdea>, ToolError> {
        self.db.load_ideas().map_err(Self::db_err)
    }

    fn list_products(&self, ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        let stage = match args.get("stage").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => Some(parse_stage(s).ok_or_else(|| {
                ToolError::Invalid(format!(
                    "`stage` must be one of idea, research, creating, review, listed, selling, archived (got {s:?})"
                ))
            })?),
            _ => None,
        };
        let limit = ctx.limit(args.get("limit"), 25, 200) as usize;
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        let mut rows: Vec<ProductIdea> = self
            .load_all()?
            .into_iter()
            .filter(|p| stage.map(|s| p.stage == s).unwrap_or(true))
            .collect();
        let total = rows.len();
        let items: Vec<Value> = rows
            .drain(..)
            .skip(offset)
            .take(limit)
            .map(|p| product_json(&p))
            .collect();

        Ok(json!({
            "count": items.len(),
            "total": total,
            "offset": offset,
            "items": items
        }))
    }

    fn get_product(&self, _ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        let id = args
            .get("id")
            .and_then(|v| v.as_i64())
            .filter(|v| *v >= 0)
            .ok_or_else(|| {
                ToolError::Invalid("`id` is required and must be a non-negative integer".into())
            })? as usize;

        // `not_found` (never `forbidden`) for a missing id: a refusal must not confirm a row exists.
        self.load_all()?
            .into_iter()
            .find(|p| p.id == id)
            .map(|p| product_json(&p))
            .ok_or(ToolError::NotFound)
    }

    fn create_product(&self, _ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ToolError::Invalid("`title` is required".into()))?
            .to_string();

        let stage = match args.get("stage").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => parse_stage(s)
                .ok_or_else(|| ToolError::Invalid(format!("unknown `stage` {s:?}")))?,
            _ => PipelineStage::Idea,
        };
        let priority = match args.get("priority").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => parse_priority(s)
                .ok_or_else(|| ToolError::Invalid(format!("unknown `priority` {s:?}")))?,
            _ => Priority::Medium,
        };

        let now = Utc::now();
        let idea = ProductIdea {
            // Ask the database for a free id: `save_idea` is INSERT OR REPLACE, so guessing an id
            // would overwrite a user's product.
            id: self.db.next_idea_id().map_err(Self::db_err)?,
            title,
            description: str_arg(args, "description"),
            stage,
            product_type: str_arg(args, "product_type"),
            created_at: now,
            updated_at: now,
            priority,
            tags: str_list(args, "tags"),
            estimated_value: args
                .get("estimated_value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            actual_value: None,
            notes: str_arg(args, "notes"),
            platform: str_list(args, "platform"),
        };

        // The same call `Pipeline::add_idea` makes.
        self.db.save_idea(&idea).map_err(Self::db_err)?;

        // Read the row back out of SQLite — the answer is what is STORED, not what was intended.
        self.load_all()?
            .into_iter()
            .find(|p| p.id == idea.id)
            .map(|p| product_json(&p))
            .ok_or_else(|| {
                ToolError::Internal("product was written but could not be read back".into())
            })
    }

    fn list_templates(&self, ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        let registry = TemplateRegistry::new();
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let limit = ctx.limit(args.get("limit"), 50, 200) as usize;

        let mut templates: Vec<&crate::templates::Template> = registry.list();
        templates.sort_by_key(|t| std::cmp::Reverse(t.trending_score));

        let items: Vec<Value> = templates
            .into_iter()
            .filter(|t| {
                let matches_query = query.is_empty()
                    || t.name.to_ascii_lowercase().contains(&query)
                    || t.description.to_ascii_lowercase().contains(&query)
                    || t.tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&query));
                let matches_category =
                    category.is_empty() || t.category.name().to_ascii_lowercase() == category;
                matches_query && matches_category
            })
            .take(limit)
            .map(|t| {
                json!({
                    "id": t.id,
                    "name": t.name,
                    "description": t.description,
                    "category": t.category.name(),
                    "tags": t.tags,
                    "output_format": format!("{:?}", t.output_format).to_ascii_lowercase(),
                    "parameters": t.parameters.iter().map(|p| json!({
                        "name": p.name,
                        "description": p.description,
                        "type": format!("{:?}", p.param_type),
                        "required": p.required,
                        "default": p.default,
                    })).collect::<Vec<_>>()
                })
            })
            .collect();

        Ok(json!({ "count": items.len(), "items": items }))
    }

    async fn generate_content(&self, ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        ctx.require_write()?;

        let template_id = args
            .get("template_id")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ToolError::Invalid("`template_id` is required".into()))?
            .to_string();
        let params = args.get("params").cloned().unwrap_or_else(|| json!({}));
        if !params.is_object() {
            return Err(ToolError::Invalid("`params` must be an object".into()));
        }
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());
        let save_as = args
            .get("save_as_product")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if self.providers_configured() == 0 {
            return Err(ToolError::Unavailable(
                "no AI provider key is configured. Add one in DPF Settings (or set \
                 DPF_OPENAI_KEY / DPF_ANTHROPIC_KEY / DPF_DEEPSEEK_KEY / DPF_GOOGLE_KEY / \
                 DPF_MOONSHOT_KEY) — generation uses your own key."
                    .into(),
            ));
        }

        // The same call the Create screen makes (`generate_blocking_with_prompt` awaits this).
        let product = self
            .generator
            .generate(&template_id, params, prompt)
            .await
            .map_err(ToolError::Unavailable)?;

        let saved = if save_as {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| product.name.clone());

            let now = Utc::now();
            let idea = ProductIdea {
                id: self.db.next_idea_id().map_err(Self::db_err)?,
                title,
                description: format!(
                    "Generated from template '{}' via the MCP door.",
                    template_id
                ),
                // Same landing stage the scheduler uses for generated work.
                stage: PipelineStage::Review,
                product_type: template_id.clone(),
                created_at: now,
                updated_at: now,
                priority: Priority::Medium,
                tags: vec!["generated".to_string(), "mcp".to_string()],
                estimated_value: 0.0,
                actual_value: None,
                // The content itself, so a saved product is readable in full afterwards.
                notes: format!(
                    "{} · {} tokens · {} ms\n\n{}",
                    product.metadata.model_used,
                    product.metadata.tokens_used,
                    product.metadata.generation_time_ms,
                    product.content
                ),
                platform: Vec::new(),
            };
            self.db.save_idea(&idea).map_err(Self::db_err)?;
            self.load_all()?
                .into_iter()
                .find(|p| p.id == idea.id)
                .map(|p| product_json(&p))
        } else {
            None
        };

        Ok(json!({
            "name": product.name,
            "template_id": product.template_id,
            "format": format!("{:?}", product.format).to_ascii_lowercase(),
            "content": product.content,
            "model_used": product.metadata.model_used,
            "tokens_used": product.metadata.tokens_used,
            "generation_time_ms": product.metadata.generation_time_ms,
            "saved_product": saved,
        }))
    }

    async fn strategy_brief(&self, ctx: &Ctx, args: &Value) -> Result<Value, ToolError> {
        ctx.require_write()?;

        let niche = args
            .get("niche")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ToolError::Invalid("`niche` is required".into()))?
            .to_string();

        if self.providers_configured() == 0 {
            return Err(ToolError::Unavailable(
                "strategy needs an AI provider key. Add one in DPF Settings (or set \
                 DPF_ANTHROPIC_KEY / DPF_DEEPSEEK_KEY / … ) — the brief uses your own key."
                    .into(),
            ));
        }

        let provider = match args.get("provider").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => Some(
                Provider::from_config_str(s).ok_or_else(|| {
                    ToolError::Invalid(format!(
                        "unknown `provider` {s:?}; use anthropic, deepseek, openai, google or moonshot"
                    ))
                })?,
            ),
            // The user's saved choice, exactly like the Strategy panel.
            _ => Provider::from_config_str(&self.config.strategy_provider),
        };
        let model = args
            .get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let m = self.config.strategy_model.trim();
                if m.is_empty() {
                    None
                } else {
                    Some(m.to_string())
                }
            });

        let request = StrategyRequest {
            niche,
            existing_products: str_arg(args, "existing_products"),
            price_positioning: str_arg(args, "price_positioning"),
        };

        // The same one-call brief the Strategy panel runs.
        let brief = strategy::run_async(&self.router, provider, model.as_deref(), &request)
            .await
            .map_err(ToolError::Unavailable)?;

        Ok(json!({
            "provider": brief.provider_label,
            "model": brief.model,
            "brief": brief.content,
        }))
    }

    fn providers_configured(&self) -> usize {
        self.router.available_providers().len()
    }
}

fn str_arg(args: &Value, key: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn str_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_stage(s: &str) -> Option<PipelineStage> {
    match s.trim().to_ascii_lowercase().as_str() {
        "idea" => Some(PipelineStage::Idea),
        "research" => Some(PipelineStage::Research),
        "creating" => Some(PipelineStage::Creating),
        "review" => Some(PipelineStage::Review),
        "listed" => Some(PipelineStage::Listed),
        "selling" => Some(PipelineStage::Selling),
        "archived" => Some(PipelineStage::Archived),
        _ => None,
    }
}

fn parse_priority(s: &str) -> Option<Priority> {
    match s.trim().to_ascii_lowercase().as_str() {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "urgent" => Some(Priority::Urgent),
        _ => None,
    }
}

fn stage_name(stage: PipelineStage) -> &'static str {
    match stage {
        PipelineStage::Idea => "idea",
        PipelineStage::Research => "research",
        PipelineStage::Creating => "creating",
        PipelineStage::Review => "review",
        PipelineStage::Listed => "listed",
        PipelineStage::Selling => "selling",
        PipelineStage::Archived => "archived",
    }
}

fn priority_name(p: Priority) -> &'static str {
    match p {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Urgent => "urgent",
    }
}

fn product_json(p: &ProductIdea) -> Value {
    json!({
        "id": p.id,
        "title": p.title,
        "description": p.description,
        "stage": stage_name(p.stage),
        "product_type": p.product_type,
        "created_at": p.created_at.to_rfc3339(),
        "updated_at": p.updated_at.to_rfc3339(),
        "priority": priority_name(p.priority),
        "tags": p.tags,
        "estimated_value": p.estimated_value,
        "actual_value": p.actual_value,
        "notes": p.notes,
        "platform": p.platform,
    })
}
