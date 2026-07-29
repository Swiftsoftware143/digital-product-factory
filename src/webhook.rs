//! Priority 6 — Automation Webhook
//!
//! Optional local HTTP listener (behind a config flag, off by default).
//! When enabled, listens on localhost:PORT for POST requests with JSON payloads.
//! Can trigger headless generation runs from n8n / Make.com / other automation.
//! Payload schema documented in README.
//!
//! Graceful failure when port is busy or when disabled — no crash.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

// ── Request / Response schemas ────────────────────────────────────────

/// POST /generate — trigger a headless generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// Template ID from the preset/template registry
    pub template_id: String,
    /// Main prompt / description for the product
    pub prompt: String,
    /// Optional: target platform(s)
    pub platforms: Option<Vec<String>>,
    /// Optional: price override
    pub price: Option<f64>,
    /// Optional: tags
    pub tags: Option<Vec<String>>,
    /// Optional: webhook URL to POST result to
    pub callback_url: Option<String>,
}

/// POST /generate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub success: bool,
    pub product_id: Option<usize>,
    pub message: String,
    pub error: Option<String>,
}

/// GET /status — health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub generation_count: u32,
}

/// GET /schema — return available endpoints and payload schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaResponse {
    pub endpoints: Vec<EndpointDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub method: String,
    pub path: String,
    pub description: String,
    pub request_schema: serde_json::Value,
    pub response_schema: serde_json::Value,
}

// ── Webhook State ─────────────────────────────────────────────────────

pub struct WebhookState {
    pub enabled: bool,
    pub port: u16,
    pub generation_count: Arc<std::sync::atomic::AtomicU32>,
    pub running: Arc<AtomicBool>,
    pub last_error: Arc<Mutex<Option<String>>>,
    pub start_time: std::time::Instant,
}

impl WebhookState {
    pub fn new(enabled: bool, port: u16) -> Self {
        Self {
            enabled,
            port,
            generation_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn record_generation(&self) {
        self.generation_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

// ── Payload Schema for README documentation ───────────────────────────

/// Returns the documented request schema as JSON Value
pub fn request_schema() -> serde_json::Value {
    serde_json::json!({
        "template_id": "string (required) — ID of the template/preset to use",
        "prompt": "string (required) — Main description or prompt for generation",
        "platforms": ["string (optional) — Target marketplaces like 'etsy', 'gumroad'"],
        "price": "number (optional) — Price override",
        "tags": ["string (optional) — Product tags"],
        "callback_url": "string (optional) — URL to POST result to when done"
    })
}

/// Returns endpoint documentation
pub fn generate_endpoint_docs() -> Vec<EndpointDoc> {
    let req_schema = request_schema();
    let resp_schema = serde_json::json!({
        "success": "boolean",
        "product_id": "number | null",
        "message": "string",
        "error": "string | null"
    });

    vec![
        EndpointDoc {
            method: "POST".into(),
            path: "/generate".into(),
            description: "Trigger a headless product generation. Returns immediately with a product_id; generation runs in background.".into(),
            request_schema: req_schema.clone(),
            response_schema: resp_schema,
        },
        EndpointDoc {
            method: "GET".into(),
            path: "/status".into(),
            description: "Health check endpoint — returns server status, version, uptime.".into(),
            request_schema: serde_json::Value::Null,
            response_schema: serde_json::json!({
                "status": "string",
                "version": "string",
                "uptime_seconds": "number",
                "generation_count": "number"
            }),
        },
        EndpointDoc {
            method: "GET".into(),
            path: "/schema".into(),
            description: "Returns this documentation — endpoint list and payload schemas.".into(),
            request_schema: serde_json::Value::Null,
            response_schema: serde_json::json!({
                "endpoints": [
                    { "method": "string", "path": "string", "description": "string",
                      "request_schema": "object", "response_schema": "object" }
                ]
            }),
        },
    ]
}
