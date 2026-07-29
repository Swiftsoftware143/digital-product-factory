//! Main application state and UI

use crate::ui::adverts_view::AdvertsManager;
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
    pub publish_manager: PublishManager,
    pub mockup_compositor: MockupCompositor,
    pub variant_manager: VariantManager,
    pub admin: AdminState,
    // -- NEW MODULES -------------------------------------------------
    pub qc_engine: QcEngine,
    pub webhook_state: WebhookState,
    pub asset_library: AssetLibrary,
    pub denylist_scanner: DenylistScanner,
    pub disclosure_rules: Vec<AiDisclosureRule>,
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
    LogoGenerator,
    VectorGenerator,
}

impl DpfApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        if false /* cfg(not(feature = embed-font)) */
        {
            fonts.font_data.insert(
                "inter".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf")),
            );
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "inter".to_owned());
        }
        cc.egui_ctx.set_fonts(fonts);

        let config = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
        let db = Arc::new(Database::new().expect("Failed to initialize database"));

        let pipeline = Pipeline::load(&db);
        let mut generator = ProductGenerator::new(&db, runtime.clone());
        generator.set_api_keys(
            config.openai_key.clone(),
            config.anthropic_key.clone(),
            config.google_key.clone(),
            config.deepseek_key.clone(),
            config.moonshot_key.clone(),
        );

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
            denylist_scanner: DenylistScanner::new(),
            disclosure_rules,
            // -- UI State -------------------------------------------
            current_tab: Tab::Dashboard,
            sidebar_expanded: true,
            search_query: String::new(),
            selected_product: None,
            show_settings: false, show_license_dialog: false, show_add_sale_dialog: false,
            selected_preset_id: None, loaded_preset_id: None,
            selected_platform: None,
            new_api_key: String::new(),
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

impl eframe::App for DpfApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.config);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.fps = 1.0 / dt;
        self.last_frame_time = now;
        ctx.request_repaint_after(std::time::Duration::from_millis(16));

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
