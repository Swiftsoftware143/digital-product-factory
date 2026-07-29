//! Inline Help System — Contextual help overlay for the desktop app

use egui::*;

#[derive(Debug, Clone)]
pub struct HelpTopic {
    pub id: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub tier: &'static str,
}

pub fn all_topics() -> Vec<HelpTopic> {
    vec![
        HelpTopic {
            id: "dashboard",
            title: "Dashboard",
            body: "Your command center. Shows total ideas, in-progress products, active sales, and total revenue from Analytics.",
            tier: "personal",
        },
        HelpTopic {
            id: "dashboard_revenue",
            title: "Revenue Summary",
            body: "Total net revenue from all sales recorded in Analytics. Click the Analytics tab for detailed breakdowns by product, platform, and template type.",
            tier: "team",
        },
        HelpTopic {
            id: "pipeline",
            title: "Pipeline (Kanban Board)",
            body: "Track products from idea to sale across 7 stages. Drag and drop cards to move between stages. Filter by search or click stage headers.",
            tier: "personal",
        },
        HelpTopic {
            id: "create",
            title: "Create Products",
            body: "Select from 20+ product templates. Each has editable parameters and optimized AI prompts. Configure, generate, review, export or save to pipeline.",
            tier: "personal",
        },
        HelpTopic {
            id: "create_generate",
            title: "AI Generation",
            body: "Click Generate after configuring a template. The app picks the best AI model for the task. Generation takes 5-30 seconds.",
            tier: "personal",
        },
        HelpTopic {
            id: "research",
            title: "Market Research",
            body: "Search Etsy, Gumroad, and Amazon to validate product ideas. Shows pricing, ratings, competition level, and top keywords.",
            tier: "personal",
        },
        HelpTopic {
            id: "bundles",
            title: "Bundles",
            body: "Bundle multiple products with discount pricing. Auto-strategies or manual creation. Export as ZIP. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "scheduler",
            title: "Scheduler",
            body: "Automate repetitive tasks: generation, publishing, research, pins. Supports Once, Daily, Weekly, Interval, and Smart schedules.",
            tier: "team",
        },
        HelpTopic {
            id: "presets",
            title: "Industry Presets",
            body: "9 pre-configured workflows for different business models. Each includes stages, actions, and tips. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "contracts",
            title: "Contract Generator",
            body: "Create legal documents: NDAs, Service Agreements, Coaching Contracts, and more. Guided prompts, export as DOCX or PDF.",
            tier: "personal",
        },
        HelpTopic {
            id: "analytics",
            title: "Analytics & Sales Tracking",
            body: "Track revenue, fees, and performance across products and platforms. Add sales records, view summaries, export CSV. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "analytics_add_sale",
            title: "Adding a Sale Record",
            body: "Click Add Sale to log a sale. Enter product name, platform, units, revenue, fees. Net revenue calculated automatically.",
            tier: "team",
        },
        HelpTopic {
            id: "analytics_csv",
            title: "CSV Export",
            body: "Export full sales ledger as CSV. Openable in Excel or Google Sheets.",
            tier: "team",
        },
        HelpTopic {
            id: "publishing",
            title: "Marketplace Publishing",
            body: "Publish products to Etsy and Gumroad from inside the app. API keys stored in OS keychain. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "publishing_etsy",
            title: "Etsy Publishing",
            body: "Requires: 3000x3000 thumb, 20MB max file, 140 char title, 5000 char desc, 13 tags. API key from Etsy Developer.",
            tier: "team",
        },
        HelpTopic {
            id: "publishing_gumroad",
            title: "Gumroad Publishing",
            body: "Requires: 1280x720 thumb, 50MB max file, 255 char title, 10000 char desc. Access token from Gumroad Settings.",
            tier: "team",
        },
        HelpTopic {
            id: "publishing_formats",
            title: "Platform Format Config",
            body: "Format rules loaded from platform_formats.json. Edit this file to update without rebuilding the app.",
            tier: "team",
        },
        HelpTopic {
            id: "export",
            title: "Export",
            body: "Export in 7 formats: Markdown, HTML, PDF, DOCX, XLSX, JSON, ZIP. Single, batch, or bundle export.",
            tier: "personal",
        },
        HelpTopic {
            id: "license",
            title: "License & Upgrades",
            body: "Click license status in bottom bar. Tiers: Personal (1 device), Team (5), Agency (20), Enterprise (unlimited). Enter key to activate.",
            tier: "personal",
        },
        HelpTopic {
            id: "license_upgrade",
            title: "Upgrade Your License",
            body: "An Upgrade button appears if a higher tier is available. Purchase, receive new key, enter it to unlock features immediately.",
            tier: "personal",
        },
        HelpTopic {
            id: "settings",
            title: "Settings",
            body: "Configure API keys (OpenAI, Anthropic, Google), toggle auto-save and dark mode, set safety limits.",
            tier: "personal",
        },
        HelpTopic {
            id: "variants",
            title: "Product Variants",
            body: "Manage multiple variants per product (different formats, prices, versions). Each variant tracks version history so you can roll back to any previous version. Available on all tiers.",
            tier: "personal",
        },
        HelpTopic {
            id: "variants_add",
            title: "Adding a Variant",
            body: "Select a product, then click Add Variant. Choose a name, format (PDF, DOCX, ZIP, etc.), and price. A v1 snapshot is created automatically.",
            tier: "personal",
        },
        HelpTopic {
            id: "variants_version",
            title: "Version History",
            body: "Each variant tracks versions. Click the clipboard icon to view history. You can view old versions in read-only mode or restore them as the current version.",
            tier: "personal",
        },
    ]
}

pub fn help_button(ui: &mut Ui, topic_id: &str, active_topic: &mut Option<String>) {
    let response = ui.small_button("?");
    if response.clicked() {
        *active_topic = Some(topic_id.to_string());
    }
    response.on_hover_text("Click for help");
}

pub fn show_help_popup(ctx: &Context, topic_id: &str, active_topic: &mut Option<String>) {
    let topics = all_topics();
    let topic = topics.iter().find(|t| t.id == topic_id);

    if let Some(t) = topic {
        Window::new(format!("Help: {}", t.title))
            .id(egui::Id::new("help_popup"))
            .collapsible(false)
            .resizable(false)
            .default_size([400.0, 250.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(t.body).size(14.0));
                    ui.add_space(8.0);

                    let (tier_label, color) = match t.tier {
                        "personal" => ("Available on all tiers", Color32::GREEN),
                        "team" => ("Requires Team+ license", Color32::YELLOW),
                        "agency" => ("Requires Agency+ license", Color32::from_rgb(255, 165, 0)),
                        "enterprise" => ("Requires Enterprise license", Color32::RED),
                        _ => ("", Color32::GRAY),
                    };
                    if !tier_label.is_empty() {
                        ui.colored_label(color, RichText::new(tier_label).size(11.0));
                    }

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() {
                        *active_topic = None;
                    }
                });
            });
    }
}

pub fn show_help_index(ctx: &Context, active_topic: &mut Option<String>) {
    Window::new("Help Index")
        .id(egui::Id::new("help_index"))
        .collapsible(false)
        .resizable(true)
        .default_size([450.0, 450.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Digital Product Factory — Help");
            ui.separator();
            ui.label("Click any topic to learn more.");
            ui.add_space(8.0);

            ScrollArea::vertical().show(ui, |ui| {
                for topic in all_topics() {
                    let tier_tag = match topic.tier {
                        "team" => " [Team+]",
                        "agency" => " [Agency+]",
                        "enterprise" => " [Enterprise]",
                        _ => "",
                    };
                    if ui.button(format!("{} — {}{}", topic.title, topic.body, tier_tag)).clicked() {
                        *active_topic = Some(topic.id.to_string());
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            if ui.button("Close Help").clicked() {
                *active_topic = None;
            }
        });
}