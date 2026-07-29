//! UI modules for egui interface

pub mod sidebar;
pub mod main_content;
pub mod status_bar;
pub mod pipeline_view;
pub mod dashboard_view;
pub mod create_view;
pub mod settings_dialog;
pub mod license_dialog;
pub mod components;
pub mod contract_view;
pub mod research_view;
pub mod scheduler_view;
pub mod bundle_view;
pub mod presets_view;
pub mod analytics_view;
pub mod mockup_view;
pub mod publish_view;
pub mod variants_view;
pub mod admin_view;
pub mod advert_preview;
pub mod advert_composer;
pub mod adverts_view;
pub mod logo_view;
pub mod vector_view;
pub mod vector_preview;


pub use sidebar::show as sidebar;
pub use main_content::show as main_content;
pub use status_bar::show as status_bar;
