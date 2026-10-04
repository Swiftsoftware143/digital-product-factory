//! Research -> Strategy: the deep-thinking layer.
//!
//! DPF generates product content cheaply and well. What it did not do is help a creator decide
//! WHAT to make — and that is where solo creators actually lose money. Their failure mode is not
//! bad copy, it is building something nobody buys. Selection is a reasoning problem, so it is the
//! one place in this app where a strong model earns its cost.
//!
//! Deliberate design choices, all of which are load-bearing:
//!
//!   * ONE call per brief. No chat loop. A strategy chat burns the user's money in a loop.
//!   * The user's own key (BYOK), whichever provider they already have.
//!   * The provider AND model are user-selectable and never locked: providers retire model ids,
//!     so the model is free text with suggestions, not a fixed enum.
//!   * Low temperature. A brief is judgement, not prose.
//!   * The output is asked to include an explicit "do NOT build this" — the anti-recommendation
//!     is often worth more than the recommendations, and it is what makes the advice feel real
//!     rather than flattering.

use crate::llm_router::{LLMRouter, Provider};

/// What the user asks for.
#[derive(Debug, Clone, Default)]
pub struct StrategyRequest {
    pub niche: String,
    /// What they already sell (optional). Sharpens the advice considerably when present.
    pub existing_products: String,
    /// Rough budget/positioning hint (optional), e.g. "under $50 impulse buy".
    pub price_positioning: String,
}

/// The brief that comes back, plus which model produced it (shown to the user for honesty).
#[derive(Debug, Clone)]
pub struct StrategyBrief {
    pub content: String,
    pub provider_label: String,
    pub model: String,
}

/// The system prompt. Written to force a *decisive* answer rather than a survey.
pub const STRATEGY_SYSTEM_PROMPT: &str = "\
You are a product strategist for a solo digital-product creator. You are decisive, concrete and \
commercially honest. You do not flatter. If the niche is weak, say so plainly and say why.

Rules you must follow:
- Use ONLY the niche and details given. Do not invent market statistics, revenue figures, or \
  'studies' you cannot source. Reason from first principles about what buyers in this niche want.
- Be specific. 'People who need organisation' is useless; 'freelancers who invoice monthly and \
  lose track of receipts' is useful.
- Always include exactly ONE explicit DO NOT BUILD item: something tempting in this niche that \
  would fail, and the reason. This is mandatory and is often the most valuable part.
- Never promise outcomes. Give reasoning the creator can check.

Return EXACTLY this structure in markdown, nothing before or after:

## The buyer
One short paragraph: who this is, and the moment they decide to spend money.

## Build these
Three numbered product ideas. For each:
- **Name** — a working title
- **What it is** — one or two sentences
- **Why it should sell** — the specific pain, and why they buy now
- **Who it competes with** — the realistic alternative (including 'a free YouTube video')

## Pricing and bundling
A concrete recommendation with reasoning: what to charge and whether to bundle. Reason about \
volume vs price-per-unit. Offer a one-time and a recurring option if both make sense. Use the \
price positioning given if provided.

## DO NOT build this
Exactly one idea that would fail, and the specific reason.

## Confidence
Low / Medium / High, plus the ONE assumption that would most change this answer if it were wrong.";

/// UI state for the Strategy panel, held on the app.
///
/// `provider_choice` / `model_choice` are the USER'S selection and are what actually get saved.
/// They default to empty, meaning "auto — pick the strongest provider I have a key for".
#[derive(Debug, Clone, Default)]
pub struct StrategyState {
    pub request: StrategyRequest,
    /// Saved provider choice ("", or a provider config id).
    pub provider_choice: String,
    /// Saved model id. Free text — may be empty (provider default) or any id the user types.
    pub model_choice: String,
    /// Last brief produced, if any.
    pub last_brief: Option<StrategyBrief>,
    /// Last error, shown verbatim so a failure is never silent.
    pub last_error: Option<String>,
    /// Which providers the user actually has keys for, refreshed when the panel draws.
    pub available: Vec<Provider>,
}

/// Build the user prompt from the request.
pub fn build_prompt(req: &StrategyRequest) -> String {
    let mut p = format!("Niche / market: {}\n", req.niche.trim());

    let existing = req.existing_products.trim();
    if existing.is_empty() {
        p.push_str("Existing products: none yet.\n");
    } else {
        p.push_str(&format!("Existing products: {}\n", existing));
    }

    let positioning = req.price_positioning.trim();
    if positioning.is_empty() {
        p.push_str("Price positioning: not specified.\n");
    } else {
        p.push_str(&format!("Price positioning: {}\n", positioning));
    }

    p.push_str("\nWrite the strategy brief now, in the exact structure specified.");
    p
}

/// Resolve which provider will actually be used, for display BEFORE spending anything.
///
/// `preferred` is the user's saved choice; `None` means auto. Mirrors the router's own resolution
/// so the panel can tell the user the truth before they press the button.
pub fn resolve_provider(router: &LLMRouter, preferred: Option<Provider>) -> Option<Provider> {
    match preferred {
        Some(p) if router.has_key(p) => Some(p),
        Some(_) => None, // chosen, but no key for it — the router will return an honest error
        None => router.best_strategic_provider(),
    }
}

/// Resolve the model id that will be used, for the same pre-spend display.
pub fn resolve_model(provider: Provider, chosen: Option<&str>) -> String {
    match chosen {
        Some(m) if !m.trim().is_empty() => m.trim().to_string(),
        _ => crate::llm_router::LLMProfile::strategic_model(provider).to_string(),
    }
}

/// Run the brief. Exactly one model call.
///
/// `runtime` is the app's own Tokio runtime — the caller passes it in, because creating a nested
/// runtime inside an existing one panics, and the app already owns one (`DpfApp::runtime`).
pub fn run(
    router: &LLMRouter,
    runtime: &tokio::runtime::Runtime,
    preferred: Option<Provider>,
    model: Option<&str>,
    req: &StrategyRequest,
) -> Result<StrategyBrief, String> {
    let niche = req.niche.trim();
    if niche.is_empty() {
        return Err("Enter a niche or market to research first.".to_string());
    }

    let provider = resolve_provider(router, preferred).ok_or_else(|| {
        "Strategy needs an AI provider key. Add one in Settings, then try again.".to_string()
    })?;
    let model_id = resolve_model(provider, model);

    // One call. No retry loop: a repeated failure here is a real error the user should see,
    // not something to spin on and spend their money twice.
    let response = runtime.block_on(router.generate_strategy(
        Some(provider),
        Some(&model_id),
        STRATEGY_SYSTEM_PROMPT.to_string(),
        build_prompt(req),
        4000,
    ))?;

    Ok(StrategyBrief {
        content: response.content,
        provider_label: provider.label().to_string(),
        model: response.model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_router::LLMProfile;

    /// The whole point of the selection feature: a user-supplied model id is used verbatim,
    /// so a retired or brand-new model never requires an app update.
    #[test]
    fn an_explicit_model_always_wins() {
        let got = resolve_model(Provider::Anthropic, Some("claude-something-not-invented-yet"));
        assert_eq!(got, "claude-something-not-invented-yet");

        // Provider-agnostic: a brand-new DeepSeek id works the same way.
        let got = resolve_model(Provider::DeepSeek, Some("deepseek-future-9000"));
        assert_eq!(got, "deepseek-future-9000");
    }

    /// Empty / whitespace means "provider default", not "blank model id".
    #[test]
    fn blank_model_falls_back_to_the_provider_default() {
        for blank in [None, Some(""), Some("   ")] {
            let got = resolve_model(Provider::Anthropic, blank);
            assert_eq!(got, crate::llm_router::LLMProfile::strategic_model(Provider::Anthropic));
            assert!(!got.trim().is_empty());
        }
    }

    /// Every provider must have a non-empty default and at least one suggestion — otherwise a
    /// user holding only that key would hit a dead end.
    #[test]
    fn every_provider_is_usable_and_selectable() {
        let all = [
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Google,
            Provider::DeepSeek,
            Provider::Moonshot,
        ];
        for p in all {
            assert!(!LLMProfile::strategic_model(p).trim().is_empty(), "{p:?} has no default model");
            assert!(!p.label().trim().is_empty(), "{p:?} has no label");
            assert!(!p.suggested_models().is_empty(), "{p:?} has no suggestions");
            assert!(!p.config_id().trim().is_empty(), "{p:?} has no config id");
        }
    }

    /// The config id round-trips, so a saved choice survives a restart.
    #[test]
    fn provider_choice_round_trips_through_config() {
        for p in [
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Google,
            Provider::DeepSeek,
            Provider::Moonshot,
        ] {
            assert_eq!(Provider::from_config_str(p.config_id()), Some(p), "{p:?} lost on save/load");
        }
        // "" is Auto; junk must not hard-fail a config written by another version.
        assert_eq!(Provider::from_config_str(""), None);
        assert_eq!(Provider::from_config_str("   "), None);
        assert_eq!(Provider::from_config_str("some-future-provider"), None);
        // Case/alias tolerance.
        assert_eq!(Provider::from_config_str("Claude"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_config_str("GEMINI"), Some(Provider::Google));
    }

    /// A brief cannot be produced without a niche, and the error says what to do.
    #[test]
    fn empty_niche_is_refused_with_a_usable_message() {
        let router = LLMRouter::new(
            "k".into(), "k".into(), "k".into(), "k".into(), "k".into(),
        );
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let err = run(&router, &rt, None, None, &StrategyRequest::default())
            .expect_err("empty niche must not run");
        assert!(err.to_lowercase().contains("niche"), "unhelpful error: {err}");
    }

    /// With no keys at all, the failure is honest and actionable — not a panic, not a hang.
    #[test]
    fn no_keys_gives_an_actionable_error() {
        let router = LLMRouter::new(
            String::new(), String::new(), String::new(), String::new(), String::new(),
        );
        assert!(router.available_providers().is_empty());
        assert!(router.best_strategic_provider().is_none());

        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let req = StrategyRequest { niche: "ADHD planners".into(), ..Default::default() };
        let err = run(&router, &rt, None, None, &req).expect_err("no keys must refuse");
        assert!(err.to_lowercase().contains("key"), "unhelpful error: {err}");
    }

    /// Choosing a provider you have no key for is an explicit error, NOT a silent fallback to a
    /// different provider — silently spending a different key would be the wrong money.
    #[test]
    fn chosen_provider_without_a_key_is_refused_not_silently_swapped() {
        // Only DeepSeek is configured...
        let router = LLMRouter::new(
            String::new(), String::new(), String::new(), "ds".into(), String::new(),
        );
        // ...but the user explicitly chose Anthropic.
        let resolved = resolve_provider(&router, Some(Provider::Anthropic));
        assert_eq!(resolved, None, "must not silently substitute DeepSeek");

        // Auto, by contrast, correctly picks the one they do have.
        assert_eq!(resolve_provider(&router, None), Some(Provider::DeepSeek));
    }

    /// Preference order is strongest-reasoning-first, so Auto does the right thing.
    #[test]
    fn auto_prefers_the_strongest_key_present() {
        let router = LLMRouter::new(
            "oai".into(), "ant".into(), "goo".into(), "ds".into(), "moo".into(),
        );
        let order = router.available_providers();
        assert_eq!(order[0], Provider::Anthropic);
        assert_eq!(order[1], Provider::DeepSeek);
        assert_eq!(router.best_strategic_provider(), Some(Provider::Anthropic));
    }

    /// The prompt must carry the niche, and must ask for the mandatory anti-recommendation —
    /// that section is what makes the brief useful rather than flattering.
    #[test]
    fn prompt_carries_the_niche_and_demands_the_anti_recommendation() {
        let req = StrategyRequest {
            niche: "  ADHD planners  ".into(),
            existing_products: "a 2026 wall planner".into(),
            price_positioning: "under $50".into(),
        };
        let p = build_prompt(&req);
        assert!(p.contains("ADHD planners"), "niche missing");
        assert!(p.contains("2026 wall planner"), "existing products missing");
        assert!(p.contains("under $50"), "positioning missing");

        assert!(STRATEGY_SYSTEM_PROMPT.contains("DO NOT build this"));
        assert!(STRATEGY_SYSTEM_PROMPT.contains("Confidence"));
        // It must not invite invented statistics.
        assert!(STRATEGY_SYSTEM_PROMPT.to_lowercase().contains("do not invent"));
    }

    /// Missing optional fields must not produce a broken prompt.
    #[test]
    fn optional_fields_are_optional() {
        let req = StrategyRequest { niche: "cats".into(), ..Default::default() };
        let p = build_prompt(&req);
        assert!(p.contains("cats"));
        assert!(p.contains("none yet"));
        assert!(p.contains("not specified"));
    }
}
