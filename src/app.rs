//! Main application state and UI

use crate::ui::adverts_view::AdvertsManager;
use crate::strategy::StrategyState;
use egui::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::{admin::AdminState,
    mockup_compositor::MockupCompositor,
    pipeline::{Pipeline, PipelineStage, ProductIdea},
    product_generator::ProductGenerator,
    product_variants::VariantManager,
    license_manager::LicenseManager,
    templates::TemplateRegistry,
    research::MarketResearch,
    scheduler::Scheduler,
    bundler::Bundler,
    exporter::Exporter,
    contract_generator::ContractGenerator,
    database::Database,
    config::AppConfig,
    presets::PresetRegistry,
    analytics::Analytics,
    publishing::PublishManager,
    qc::QcEngine,
    webhook::WebhookState,
    asset_library::AssetLibrary,
    client_manager::{Client, ClientManager},
    compliance::{DenylistScanner, AiDisclosureRule, AiToolLicense},
    ui::{sidebar, main_content, status_bar, analytics_view, publish_view, settings_dialog, license_dialog},
};

/// State for Logo Generator & Vector Generator modules
#[derive(Clone)]
pub struct VectorState {
    pub brand_name: String,
    pub tagline: String,
    pub selected_style: crate::vector_types::LogoStyle,
    pub palette_input: String,
    pub icon_description: String,
    pub favicon_enabled: bool,
    pub current_logo: Option<crate::vector_types::Logo>,
    pub saved_logos: Vec<crate::vector_types::Logo>,
    pub selected_logo_index: Option<usize>,

    pub vector_name: String,
    pub prompt: String,
    pub style_input: String,
    pub selected_category: crate::vector_types::VectorCategory,
    pub current_vector: Option<crate::vector_types::VectorAsset>,
    pub saved_vectors: Vec<crate::vector_types::VectorAsset>,
    pub selected_vector_index: Option<usize>,

    pub error: Option<String>,
}

impl Default for VectorState {
    fn default() -> Self {
        Self {
            brand_name: String::new(),
            tagline: String::new(),
            selected_style: crate::vector_types::LogoStyle::Modern,
            palette_input: String::new(),
            icon_description: String::new(),
            favicon_enabled: false,
            current_logo: None,
            saved_logos: Vec::new(),
            selected_logo_index: None,
            vector_name: String::new(),
            prompt: String::new(),
            style_input: String::new(),
            selected_category: crate::vector_types::VectorCategory::Icon,
            current_vector: None,
            saved_vectors: Vec::new(),
            selected_vector_index: None,
            error: None,
        }
    }
}

/// State for the Create Product module (template selection + parameter form +
/// prompt preview + generation result)
#[derive(Clone)]
pub struct CreateState {
    /// id of the template the user has selected (None = nothing selected yet)
    pub selected_template: Option<String>,
    /// active category filter, stored as the category's display name
    /// (None = show all)
    pub category_filter: Option<String>,
    /// current value per template parameter name, edited by the parameter form
    pub param_values: std::collections::HashMap<String, String>,
    /// result of the last `preview_template` call
    pub prompt_preview: Option<String>,
    /// status / error line shown to the user
    pub status: Option<String>,
    /// true while a synchronous generation call is in flight
    pub is_generating: bool,
    /// most recent successfully generated product
    pub last_product: Option<crate::product_generator::GeneratedProduct>,
}

impl Default for CreateState {
    fn default() -> Self {
        Self {
            selected_template: None,
            category_filter: None,
            param_values: std::collections::HashMap::new(),
            prompt_preview: None,
            status: None,
            is_generating: false,
            last_product: None,
        }
    }
}

pub struct DpfApp {
    pub db: Arc<Database>,
    pub runtime: Arc<Runtime>,
    pub config: AppConfig,
    pub pipeline: Pipeline,
    pub generator: ProductGenerator,
    pub license_manager: LicenseManager,
    pub template_registry: TemplateRegistry,
    pub research: MarketResearch,
    pub scheduler: Scheduler,
    pub bundler: Bundler,
    pub exporter: Exporter,
    pub contract_generator: ContractGenerator,
    pub preset_registry: PresetRegistry,
    pub analytics: Analytics,
    pub adverts_manager: AdvertsManager,
    pub vector_state: VectorState,
    pub create_state: CreateState,
    pub publish_manager: PublishManager,
    pub mockup_compositor: MockupCompositor,
    pub variant_manager: VariantManager,
    pub admin: AdminState,
    // -- NEW MODULES -------------------------------------------------
    pub qc_engine: QcEngine,
    pub webhook_state: WebhookState,
    pub asset_library: AssetLibrary,
    /// Client Management — the module behind the `client_management` feature slug
    /// (Agency + Enterprise tiers). See `client_manager`.
    pub clients: ClientManager,
    pub client_draft: Client,
    pub client_editing: Option<usize>,
    pub client_search: String,
    pub client_status_message: Option<(bool, String)>,
    /// Throttles the scheduler check; the UI loop runs far faster than once a second.
    pub last_scheduler_tick: std::time::Instant,
    pub denylist_scanner: DenylistScanner,
    pub disclosure_rules: Vec<AiDisclosureRule>,
    /// Deep-thinking layer state. The provider/model choice lives in `config` (so it persists);
    /// this holds the in-flight panel state and the last brief.
    pub strategy_state: StrategyState,
    /// Scheduler add-task dialog. The draft lives on the app so it survives the frame — a view is
    /// redrawn constantly and local state would be wiped between keystrokes.
    /// Contract flow state: which template is open, its answers, the chosen category filter, and
    /// the last result/error. On the app because a view is redrawn every frame and local state
    /// would be wiped between keystrokes.
    pub contract_selected: Option<String>,
    pub contract_answers: std::collections::HashMap<String, String>,
    pub contract_category: Option<crate::contract_generator::ContractCategory>,
    pub contract_result: Option<crate::contract_generator::GeneratedContract>,
    pub contract_error: Option<String>,
    pub scheduler_adding: bool,
    pub scheduler_draft_name: String,
    pub scheduler_draft_type: crate::ui::scheduler_view::TaskTypeChoice,
    pub scheduler_draft_query: String,
    pub scheduler_draft_freq: crate::ui::scheduler_view::FrequencyChoice,
    // -- UI State ----------------------------------------------------
    pub current_tab: Tab,
    pub sidebar_expanded: bool,
    pub search_query: String,
    pub selected_product: Option<usize>,
    pub show_settings: bool,
    pub show_license_dialog: bool,
    pub show_add_sale_dialog: bool,
    pub selected_preset_id: Option<String>,
    pub loaded_preset_id: Option<String>,
    pub selected_platform: Option<String>,
    pub new_api_key: String,
    /// Licence activation — must live on the app, not in the frame, or typing resets each frame.
    pub license_key_input: String,
    /// (is_error, message) shown under the activation form.
    pub license_message: Option<(bool, String)>,
    pub publish_target: String,
    pub publish_price: f64,
    pub pending_publish: Option<(String, String, f64)>,
    pub new_sale: analytics_view::NewSaleDraft,
    pub active_help_topic: Option<String>,
    pub last_frame_time: std::time::Instant,
    pub fps: f32,
    // -- QC UI state -------------------------------------------------
    pub qc_target_product_id: Option<usize>,
    pub qc_target_platform: String,
    pub qc_current_result: Option<crate::qc::QcResult>,
    pub qc_manual_approve: bool,
    // -- Asset Library UI state --------------------------------------
    pub asset_search: String,
    pub asset_selected_id: Option<usize>,
    pub asset_version_notes: String,
    // -- Compliance UI state -----------------------------------------
    pub compliance_prompt: String,
    pub compliance_scan_result: Vec<String>,
    pub compliance_show_warning: bool,
    // -- Webhook UI state --------------------------------------------
    pub webhook_port: String,
    pub webhook_enabled: bool,
    pub webhook_status_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard, Pipeline, Mockup, Create, Research, Templates,
    Bundles, Scheduler, Presets, Contract, Analytics, Publish, Settings,
    Admin, QC, AssetLibrary, Compliance, Webhooks, Variants, Adverts,
    Clients,
    LogoGenerator,
    VectorGenerator,
}

impl DpfApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let fonts = egui::FontDefinitions::default();
        // REMOVED 2026-10-02 (build-blocker): this was a dead `if false` block that called
        //   egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf"))
        // `include_bytes!` is a COMPILE-TIME macro — the `if false` guard did NOT stop the file
        // from being read at build time, and `assets/Inter-Regular.ttf` is in .gitignore, so the
        // repository did NOT compile from a clean clone (error: couldn't read
        // `src/../assets/Inter-Regular.ttf`). The block was already disabled, so removing it
        // changes nothing at runtime. If the custom font is wanted later, commit the .ttf and
        // re-add this properly under a real `#[cfg(feature = "embed-font")]`.
        cc.egui_ctx.set_fonts(fonts);

        // Everything below runs on the UI thread BEFORE the first frame is ever painted. If any
        // step blocks or panics, the window is already on screen and simply stays black — the
        // exact symptom reported on Windows. Each step is logged so the startup log names the
        // step it died in instead of just ending. (This whole block is also the reason a black
        // window cannot be assumed to be a GPU problem: if the log stops here, it never was.)
        let t0 = std::time::Instant::now();
        crate::log_line("startup: reading config");
        let config = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        crate::log_line("startup: creating Tokio runtime");
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

        crate::log_line("startup: opening database");
        let db = Arc::new(Database::new().expect("Failed to initialize database"));

        crate::log_line("startup: loading pipeline");
        let pipeline = Pipeline::load(&db);

        crate::log_line("startup: building generator");
        let mut generator = ProductGenerator::new(&db, runtime.clone());
        generator.set_api_keys(
            config.openai_key.clone(),
            config.anthropic_key.clone(),
            config.google_key.clone(),
            config.deepseek_key.clone(),
            config.moonshot_key.clone(),
        );

        crate::log_line("startup: licence + registries");
        let license_manager = LicenseManager::new(&db);
        let template_registry = TemplateRegistry::new();
        let research = MarketResearch::new(runtime.clone());
        let scheduler = Scheduler::new(&db, runtime.clone());
        let bundler = Bundler::new();
        let exporter = Exporter::new();
        let mut contract_generator = ContractGenerator::new();
        contract_generator.set_api_keys(
            config.openai_key.clone(),
            config.anthropic_key.clone(),
            config.google_key.clone(),
            config.deepseek_key.clone(),
            config.moonshot_key.clone(),
        );
        let preset_registry = PresetRegistry::new();
        let analytics = Analytics::new(&db);
        let publish_manager = PublishManager::new(&db);
        let mockup_compositor = MockupCompositor::new();
        let admin = AdminState::new();

        let format_path = std::path::Path::new("platform_formats.json");
        if !format_path.exists() {
            PublishManager::save_formats_to_file("platform_formats.json");
        }

        // -- NEW MODULES INIT ---------------------------------------
        let qc_engine = QcEngine::new("dpf_data.db");
        let mut asset_library = AssetLibrary::new();
        asset_library.load_from_db(&db);

        // Client Management (Agency+ feature)
        let clients = ClientManager::new(&db);

        // Save default disclosure rules
        let disclosure_path = std::path::Path::new("ai_disclosure_rules.json");
        if !disclosure_path.exists() {
            AiDisclosureRule::save("ai_disclosure_rules.json");
        }
        let disclosure_rules = AiDisclosureRule::load("ai_disclosure_rules.json");
        let db_clone = Arc::clone(&db);

        Self {
            db, runtime, config,
            pipeline, generator, license_manager, template_registry,
            research, scheduler, bundler, exporter, contract_generator,
            preset_registry, analytics, publish_manager, mockup_compositor,
            admin,
            // -- NEW MODULES ----------------------------------------
            qc_engine,
            webhook_state: WebhookState::new(false, 9823),
            variant_manager: VariantManager::new(&db_clone),
            asset_library,
            adverts_manager: AdvertsManager::new(),
            vector_state: VectorState::default(),
            create_state: CreateState::default(),
            denylist_scanner: DenylistScanner::new(),
            disclosure_rules,
            strategy_state: StrategyState::default(),
            contract_selected: None,
            contract_answers: std::collections::HashMap::new(),
            contract_category: None,
            contract_result: None,
            contract_error: None,
            scheduler_adding: false,
            scheduler_draft_name: String::new(),
            scheduler_draft_type: crate::ui::scheduler_view::TaskTypeChoice::ResearchMarket,
            scheduler_draft_query: String::new(),
            scheduler_draft_freq: crate::ui::scheduler_view::FrequencyChoice::Daily,
            // -- UI State -------------------------------------------
            current_tab: Tab::Dashboard,
            sidebar_expanded: true,
            search_query: String::new(),
            selected_product: None,
            show_settings: false, show_license_dialog: false, show_add_sale_dialog: false,
            selected_preset_id: None, loaded_preset_id: None,
            selected_platform: None,
            new_api_key: String::new(),
            license_key_input: String::new(),
            license_message: None,
            clients,
            client_draft: Client::new(0),
            client_editing: None,
            client_search: String::new(),
            client_status_message: None,
            last_scheduler_tick: std::time::Instant::now(),
            publish_target: String::new(),
            publish_price: 9.99,
            pending_publish: None,
            new_sale: analytics_view::NewSaleDraft::default(),
            active_help_topic: None,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            // -- QC UI state ---------------------------------------
            qc_target_product_id: None,
            qc_target_platform: "etsy".into(),
            qc_current_result: None,
            qc_manual_approve: false,
            // -- Asset Library UI state -----------------------------
            asset_search: String::new(),
            asset_selected_id: Option::<usize>::None,
            asset_version_notes: String::new(),
            // -- Compliance UI state --------------------------------
            compliance_prompt: String::new(),
            compliance_scan_result: Vec::new(),
            compliance_show_warning: false,
            // -- Webhook UI state -----------------------------------
            webhook_port: "9823".into(),
            webhook_enabled: false,
            webhook_status_message: String::new(),
        }
    }
}

impl DpfApp {
    /// True when the user has not entered a single AI provider key.
    ///
    /// The product cannot generate anything until they do, so the UI must say so up front
    /// rather than only failing with "API keys not configured" on their first Generate.
    pub fn has_no_api_key(&self) -> bool {
        self.config.openai_key.trim().is_empty()
            && self.config.anthropic_key.trim().is_empty()
            && self.config.google_key.trim().is_empty()
            && self.config.deepseek_key.trim().is_empty()
            && self.config.moonshot_key.trim().is_empty()
    }

    /// Drive the scheduler from the UI loop.
    ///
    /// This runs here rather than on a background thread because carrying out a task needs the
    /// real modules — generator, pipeline, database — which live on the app. The thread this
    /// replaced was handed only the database and its body did nothing at all, so no scheduled
    /// task ever ran while every one of them reported success.
    fn tick_scheduler(&mut self, ctx: &egui::Context) {
        if !self.scheduler.is_running() {
            return;
        }

        // Once a second is plenty; the UI loop runs far faster than that.
        let now = std::time::Instant::now();
        if now.duration_since(self.last_scheduler_tick).as_secs() < 1 {
            return;
        }
        self.last_scheduler_tick = now;

        for task in self.scheduler.due_tasks() {
            let outcome = self.execute_scheduled_task(&task);
            self.scheduler.record_run(task.id, outcome);
            ctx.request_repaint();
        }
    }

    /// Carry out one scheduled task for real.
    ///
    /// Anything not implemented returns Err **on purpose**: a task that shows FAILED with a
    /// reason is honest, whereas the previous code printed a line and reported COMPLETED while
    /// doing nothing whatsoever.
    fn execute_scheduled_task(
        &mut self,
        task: &crate::scheduler::ScheduledTask,
    ) -> Result<(), String> {
        use crate::scheduler::TaskType;

        match &task.task_type {
            TaskType::GenerateProduct { template_id, params } => {
                let product = self.generator.generate_blocking(template_id, params.clone())?;

                let idea = ProductIdea {
                    id: 0,
                    title: product.name.clone(),
                    description: format!("Generated on schedule from template '{}'.", template_id),
                    stage: PipelineStage::Review,
                    product_type: template_id.clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    priority: crate::pipeline::Priority::Medium,
                    tags: vec!["scheduled".to_string()],
                    estimated_value: 0.0,
                    actual_value: Some(0.0),
                    notes: format!(
                        "Scheduled run · {} · {} tokens · {} ms",
                        product.metadata.model_used,
                        product.metadata.tokens_used,
                        product.metadata.generation_time_ms
                    ),
                    platform: Vec::new(),
                };
                self.pipeline.add_idea(&self.db, idea);
                Ok(())
            }

            TaskType::BackupData => {
                let src = std::path::Path::new("dpf_data.db");
                if !src.exists() {
                    return Err("There is no database file to back up yet.".to_string());
                }
                let dir = std::path::Path::new("backups");
                std::fs::create_dir_all(dir)
                    .map_err(|e| format!("Could not create the backups folder: {e}"))?;
                let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                let dest = dir.join(format!("dpf_data-{stamp}.db"));
                std::fs::copy(src, &dest).map_err(|e| format!("Backup failed: {e}"))?;
                Ok(())
            }

            other => Err(format!(
                "{} is not implemented yet, so this task cannot run. It will keep reporting \
                 failure until it is built — it is NOT silently doing nothing.",
                task_kind_name(other)
            )),
        }
    }
}

/// Human-readable name for a task type, used in failure messages.
fn task_kind_name(t: &crate::scheduler::TaskType) -> &'static str {
    use crate::scheduler::TaskType;
    match t {
        TaskType::GenerateProduct { .. } => "Scheduled product generation",
        TaskType::PublishProduct { .. } => "Scheduled publishing",
        TaskType::ResearchMarket { .. } => "Scheduled market research",
        TaskType::CreateBundle { .. } => "Scheduled bundle creation",
        TaskType::PinterestPin { .. } => "Scheduled Pinterest pinning",
        TaskType::BackupData => "Scheduled backup",
    }
}

impl eframe::App for DpfApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.config);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.fps = 1.0 / dt;
        self.last_frame_time = now;

        // Apply the chosen theme. `config.dark_mode` already existed and was persisted by the
        // Settings checkbox, but nothing ever called `set_visuals`, so the setting had no effect
        // whatsoever. Applying it every frame keeps the status-bar toggle instant.
        ctx.set_visuals(if self.config.dark_mode {
            crate::theme::dark()
        } else {
            crate::theme::light()
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        // Scheduler: fire any task whose time has come. Doing this from the UI loop (rather than
        // the do-nothing background thread this replaced) means tasks run against the real
        // modules and their status reflects what actually happened.
        self.tick_scheduler(ctx);

        // THE decisive line. "frame N rendered" means the app finished setup, built its UI and
        // handed frames to the renderer — so a black window on top of this is the GPU never
        // PRESENTING, and the fix is a back end / driver matter. If this line never appears, the
        // app never reached its first frame at all and no amount of renderer-switching will help:
        // the fault is in startup, above. Only the first few are logged so the file stays small.
        {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static FRAMES: AtomicUsize = AtomicUsize::new(0);
            let n = FRAMES.fetch_add(1, Ordering::Relaxed);
            if n < 3 {
                crate::log_line(&format!(
                    "frame {} rendered ({} ideas, {}x{} px)",
                    n + 1,
                    self.pipeline.ideas.len(),
                    ctx.screen_rect().width() as i32,
                    ctx.screen_rect().height() as i32
                ));
            }
        }

        if let Some((product_name, platform, price)) = self.pending_publish.take() {
            let product_id = self.pipeline.ideas.iter()
                .find(|i| i.title == product_name)
                .map(|i| i.id)
                .unwrap_or(0);
            tracing::info!("Queued publish: {} on {} for ${:.2}", product_name, platform, price);
            let log = crate::publishing::PublishLog {
                id: self.publish_manager.publish_logs.len() + 1,
                product_id,
                product_name: product_name.clone(),
                platform: platform.clone(),
                listing_url: None, listing_id: None,
                status: crate::publishing::PublishStatus::Pending,
                error_message: None,
                published_at: chrono::Utc::now(),
            };
            let _ = self.db.save_publish_log(&log);
            self.publish_manager.publish_logs.insert(0, log);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Digital Product Factory");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{:.0} FPS", self.fps));
                    if ui.button("\u{2699}").clicked() { self.show_settings = true; }
                });
            });
        });

        sidebar::show(self, ctx);
        main_content::show(self, ctx);
        status_bar::show(self, ctx);

        if self.show_settings { settings_dialog::show(self, ctx); }
        if self.show_license_dialog { license_dialog::show(self, ctx); }
    }
}
