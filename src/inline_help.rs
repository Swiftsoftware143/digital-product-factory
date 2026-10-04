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
            body: "Select from 12 ready-made product templates. Each has its own editable parameters and an optimised AI prompt. Pick one, fill in the parameters, preview the prompt, then generate.",
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
            id: "strategy",
            title: "Strategy (deep thinking)",
            body: "Tells you WHAT to build, not just how to build it. Enter a niche and press one button: you get three concrete product ideas with the reasoning behind each, a pricing and bundling suggestion, one explicit 'do not build this' warning, and a confidence level. It runs ONE call on the AI key you already have — it uses whichever provider you pick, or picks the strongest one you hold if you leave it on Auto. You can also type any model id by hand, so a new or retired model never blocks you. This is deliberately a single brief, not a chat, so it cannot run up your bill.",
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
            body: "9 pre-configured workflows for different business models. Each includes stages, actions, and tips. Loading a preset fills your pipeline with the right starting ideas.",
            tier: "personal",
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
            body: "Seven formats, all real files: Markdown, HTML, PDF, Word (DOCX), Excel (XLSX), JSON and ZIP. PDF is a genuine PDF document you can open in any reader and print directly. DOCX opens in Word with headings, paragraphs and bullet lists preserved. XLSX is a real spreadsheet — any markdown table in the content becomes spreadsheet rows and columns. You can export one product, a batch, or a bundle.",
            tier: "personal",
        },
        HelpTopic {
            id: "license",
            title: "Licence & Tiers",
            body: "Click the 🔑 Licence button at the bottom of the sidebar. Your current tier is shown under the app name. Tiers: Personal (free, 1 seat), Team (5 seats), Agency (20 seats), Enterprise (unlimited seats). The app tells you which plan you hold — what it costs is on the sales page, because a licence can be a one-time payment or a subscription. Anything your tier does not include shows a 🔒 and cannot be opened.",
            tier: "personal",
        },
        HelpTopic {
            id: "license_activate",
            title: "Activating a Licence",
            body: "Click 🔑 Licence, paste your key in the form DPF-TIER-XXXXXXXX-CCCC (for example DPF-TEAM-DEMO1234-2P7A) and press Activate. Activation is saved, so it survives restarting the app. If a key is rejected the reason is shown under the box — a mistyped character is the usual cause.",
            tier: "personal",
        },
        HelpTopic {
            id: "settings",
            title: "Settings",
            body: "Enter your own AI provider keys here — all five providers are configured in Settings: OpenAI, Anthropic (Claude), Google (Gemini), DeepSeek and Moonshot. Fields are masked on screen. You also set safety limits here.",
            tier: "personal",
        },
        HelpTopic {
            id: "theme",
            title: "Daylight & Night Theme",
            body: "Two themes. Click the 🌙 Night / ☀️ Daylight button on the status bar to switch instantly, or pick one in Settings under Appearance. The button always shows the theme you are currently in. Your choice is remembered next time you open the app.",
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
        HelpTopic {
            id: "byok",
            title: "Bring Your Own Key (BYOK)",
            body: "This app does not sell or bundle AI credits. You bring your OWN API key from a provider you already pay, and generation is billed by that provider directly at their rate. Open ⚙ Settings at the bottom of the sidebar and paste your key. Five providers are supported: OpenAI, Anthropic (Claude), Google (Gemini), DeepSeek and Moonshot. You only need one to start. Your keys are stored locally on your own machine and are masked on screen. Nothing is routed through us.",
            tier: "personal",
        },
        HelpTopic {
            id: "byok_which",
            title: "Which AI Provider Should I Use?",
            body: "Any of the five works. DeepSeek and Moonshot are usually the cheapest for bulk content; OpenAI and Anthropic tend to be strongest on long-form writing; Google is a good all-rounder. You can paste several keys and the app picks a sensible model per task. Check your provider's current pricing before generating large batches.",
            tier: "personal",
        },
        HelpTopic {
            id: "templates",
            title: "Templates Catalogue",
            body: "Browse every template the app ships: name, description, category, tags, trending score and output format. Filter by category to narrow the list, then jump to Create to generate from the one you picked.",
            tier: "personal",
        },
        HelpTopic {
            id: "logo_generator",
            title: "Logo Generator",
            body: "Give a brand name, optional tagline, a hex colour palette and an icon description, then pick one of 7 styles (Minimal, Modern, Vintage, Playful, Corporate, Tech, HandDrawn). The AI returns three SVGs per logo: the icon, the typography, and the two combined. Preview updates live and saving persists the logo in your library. Tick Favicon Package to also export 16/32/48/192/512 PNGs, an .ico, an apple-touch-icon and a site.webmanifest into a folder you choose. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "vector_generator",
            title: "Vector Generator",
            body: "Describe the asset you want and pick a category: Icon, Illustration, Badge, Pattern, Decorative, Infographic or UI Element. Optionally add a style and a hex palette. The AI returns an SVG plus its palette and viewBox, previewed live. Saved vectors persist in your library, and you can export either SVG or a 512x512 PNG. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "adverts",
            title: "Adverts & Campaigns",
            body: "Build ad campaigns against your pipeline products. Write copy with a framework (PAS, AIDA or BAB), render the advert at several aspect ratios (including square 1080x1080, story 1080x1920 and landscape 1200x628), and score the result for conversion. Export a single advert or the whole campaign as JSON. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "mockup",
            title: "Mockup Compositor",
            body: "Drop a product onto a mockup and export the composite as PNG or JPG. Useful for marketplace thumbnails and listing images. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "assets",
            title: "Asset Library",
            body: "One place for every file your products have generated. Search by name, tag or format, and each asset keeps a version history so you can roll back to an earlier version. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "webhooks",
            title: "Webhooks — not yet available",
            body: "The Webhooks tab is a placeholder: the interface reports a running listener, but no web server is actually started in this build, so nothing can connect to it. Do not rely on it. This is a known gap, not a configuration problem on your side.",
            tier: "team",
        },
        HelpTopic {
            id: "qc",
            title: "QC Checklist",
            body: "Run the pre-publish quality checklist on a product: duplicate detection, required-field checks and marketplace format validation (for example Etsy's tag count and title length). Catch rejections before you list. (Team+ feature)",
            tier: "team",
        },
        HelpTopic {
            id: "clients",
            title: "Client Management",
            body: "Keep a record of the clients you produce work for: name, email, company, a status (Prospect, Active, Paused, Closed) and free notes. Filter by name, email or company, and edit or delete any entry. Everything is stored locally with your other data. (Agency and Enterprise feature)",
            tier: "agency",
        },
        HelpTopic {
            id: "compliance",
            title: "Compliance Scanner",
            body: "Scan generated copy for AI-disclosure requirements and protected terms. The protected-term list ships with sensible defaults and can be edited. (Agency+ feature)",
            tier: "agency",
        },
        HelpTopic {
            id: "admin",
            title: "Admin Panel",
            body: "Edit the tier/feature map, pricing, marketplace format rules and the key revocation list, and inspect the licence state. This is where you change what each plan unlocks without rebuilding the app. (Enterprise feature)",
            tier: "enterprise",
        },
        HelpTopic {
            id: "asset_library",
            title: "Asset Library",
            body: "Where your own files live — images, fonts, logos and anything else you reuse across products. Add assets once, then reference them from the Create, Mockup and Advert tabs instead of hunting for the file each time. Everything is stored locally on your machine.",
            tier: "personal",
        },
        HelpTopic {
            id: "mockup_compositor",
            title: "Mockup Compositor",
            body: "Puts your digital product onto a realistic product shot — a laptop screen, a tablet, a printed page — so buyers can see what they are getting. Pick a base mockup, drop your artwork onto it, and export the result. Note: this composites and arranges existing images; it does not generate new artwork, so you still supply the underlying image.",
            tier: "team",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Every `?` button passes a topic id. If that id is not defined here the button silently
    /// does nothing — a bug that shipped THREE times (`asset_library`, `mockup_compositor`, and
    /// `strategy` when it was first added).
    ///
    /// This scans the real source via CARGO_MANIFEST_DIR rather than trusting a hand-written list:
    /// a hand-maintained list is exactly what let the third one through (it said `publish` when
    /// the actual id is `publishing`), so it cannot be the guard.
    #[test]
    fn every_help_button_points_at_a_defined_topic() {
        use std::fs;
        use std::path::PathBuf;

        let defined: HashSet<&str> = all_topics().iter().map(|t| t.id).collect();
        assert!(defined.len() > 10, "topic list looks empty");

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut checked = 0usize;
        let mut missing: Vec<String> = Vec::new();

        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(&p, out);
                    } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                        out.push(p);
                    }
                }
            }
        }

        let mut files = Vec::new();
        walk(&root, &mut files);

        for f in files {
            if f.file_name().map(|n| n == "inline_help.rs").unwrap_or(false) {
                continue; // defines topics; does not reference them
            }
            let Ok(text) = fs::read_to_string(&f) else { continue };
            // find `help_button(<anything>, "topic"`
            for (i, _) in text.match_indices("help_button(") {
                let rest = &text[i..];
                let Some(comma) = rest.find(',') else { continue };
                let after = &rest[comma + 1..];
                let Some(q1) = after.find('"') else { continue };
                let after_q = &after[q1 + 1..];
                let Some(q2) = after_q.find('"') else { continue };
                let topic = &after_q[..q2];
                if topic.is_empty() || !topic.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                    continue; // not a literal topic id (e.g. a variable)
                }
                checked += 1;
                if !defined.contains(topic) {
                    missing.push(format!("{topic} ({})", f.file_name().unwrap_or_default().to_string_lossy()));
                }
            }
        }

        assert!(checked >= 10, "only found {checked} help_button calls - scanner is broken");
        assert!(
            missing.is_empty(),
            "these ? buttons have no HelpTopic and do nothing: {missing:?}"
        );
    }

    /// The Strategy topic must actually explain the two things a user needs to know: it costs
    /// them one call on their own key, and it is not a chat.
    #[test]
    fn strategy_help_is_honest_about_cost_and_scope() {
        let t = all_topics()
            .into_iter()
            .find(|t| t.id == "strategy")
            .expect("strategy topic must exist");
        let b = t.body.to_lowercase();
        assert!(b.contains("one call") || b.contains("single brief"), "does not state the cost model");
        assert!(b.contains("not a chat"), "does not say it is not a chat");
        assert!(b.contains("key"), "does not mention it uses the user's own key");
    }

    /// Topic ids must be unique, or a `?` button opens the wrong topic.
    #[test]
    fn topic_ids_are_unique() {
        let topics = all_topics();
        let unique: HashSet<&str> = topics.iter().map(|t| t.id).collect();
        assert_eq!(unique.len(), topics.len(), "duplicate help topic id");
    }

    /// Every topic needs a title and body, or the popup renders blank.
    #[test]
    fn every_topic_is_renderable() {
        for t in all_topics() {
            assert!(!t.id.trim().is_empty(), "topic with empty id");
            assert!(!t.title.trim().is_empty(), "{} has no title", t.id);
            assert!(!t.body.trim().is_empty(), "{} has no body", t.id);
        }
    }
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