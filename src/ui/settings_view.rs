//! The Settings screen — the ONE place a user sets up their business.
//!
//! Before this, setup was split in two: two of the five AI keys were here, while the marketplace
//! rules (the Etsy/Gumroad limits that decide whether a listing is accepted at all) were only
//! reachable from the Admin panel — which is the owner's. A customer could not find, let alone
//! change, the rules their products had to satisfy. Everything a user must supply now lives here.

use crate::app::DpfApp;
use crate::inline_help;
use crate::publishing::PlatformFormat;
use egui::*;

/// One AI provider row. Kept as data so the five providers cannot drift apart in the UI.
struct ProviderRow {
    label: &'static str,
    hint: &'static str,
    /// Where the user CREATES this key. Shown as a clickable link, because "paste your key" is
    /// useless advice to someone who does not yet have one.
    key_url: &'static str,
    /// The `config` field this row edits.
    field: fn(&mut crate::config::AppConfig) -> &mut String,
}

const PROVIDERS: &[ProviderRow] = &[
    ProviderRow {
        label: "OpenAI",
        hint: "gpt-4o and friends. Strong all-rounder for creative copy and visuals.",
        key_url: "https://platform.openai.com/api-keys",
        field: |c| &mut c.openai_key,
    },
    ProviderRow {
        label: "Anthropic",
        hint: "Claude. Strong at long, structured documents and business strategy.",
        key_url: "https://console.anthropic.com/settings/keys",
        field: |c| &mut c.anthropic_key,
    },
    ProviderRow {
        label: "Google",
        hint: "Gemini. Fast and cheap; good for bulk, technical and factual work.",
        key_url: "https://aistudio.google.com/app/apikey",
        field: |c| &mut c.google_key,
    },
    ProviderRow {
        label: "DeepSeek",
        hint: "Strong reasoning for the money. Good default if you want one key to do most things.",
        key_url: "https://platform.deepseek.com/api_keys",
        field: |c| &mut c.deepseek_key,
    },
    ProviderRow {
        label: "Moonshot",
        hint: "Kimi. Best choice if you also produce Chinese-language content.",
        key_url: "https://platform.moonshot.ai/console/api-keys",
        field: |c| &mut c.moonshot_key,
    },
];

/// The marketplaces, read live from `platform_formats.json` rather than hardcoded here, so editing
/// that file (or shipping a new one) changes what this screen says.
fn marketplace_rows() -> Vec<(String, PlatformFormat)> {
    let path = "platform_formats.json";
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, PlatformFormat>>(&raw)
    else {
        return Vec::new();
    };
    let mut rows: Vec<(String, PlatformFormat)> = map.into_iter().collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

pub fn show(app: &mut DpfApp, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Settings");
                inline_help::help_button(ui, "settings", &mut app.active_help_topic);
            });
            ui.label(
                RichText::new("Everything you have to fill in to run your business is on this one page.")
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(10.0);

            show_checklist(app, ui);
            ui.add_space(8.0);
            show_providers(app, ui);
            ui.add_space(8.0);
            show_marketplaces(ui);
            ui.add_space(8.0);
            show_preferences(app, ui);
        });
    });
}

/// What the user still has to do, computed from live state — never a static tick list, because a
/// checklist that shows the same thing regardless of what you have configured is worse than none.
fn show_checklist(app: &mut DpfApp, ui: &mut Ui) {
    let keys_held = PROVIDERS
        .iter()
        .filter(|p| !(p.field)(&mut app.config).trim().is_empty())
        .count();
    let licensed = app.license_manager.is_licensed();
    let selling = !marketplace_rows().is_empty();

    ui.group(|ui| {
        ui.strong("What you need to get going");
        ui.add_space(6.0);

        let row = |ui: &mut Ui, done: bool, text: &str, why: &str| {
            ui.horizontal(|ui| {
                ui.label(if done {
                    RichText::new("✅").size(15.0)
                } else {
                    RichText::new("⬜").size(15.0)
                });
                ui.vertical(|ui| {
                    ui.label(RichText::new(text).strong());
                    ui.label(RichText::new(why).size(11.0).color(Color32::from_gray(140)));
                });
            });
            ui.add_space(4.0);
        };

        row(
            ui,
            keys_held > 0,
            "An AI provider key",
            if keys_held > 0 {
                "done — the app can generate"
            } else {
                "REQUIRED. One key from any provider below is enough. You buy it from that provider; \
                 the app never resells or proxies it."
            },
        );
        row(
            ui,
            licensed,
            "Your licence key",
            if licensed {
                "done"
            } else {
                "Optional — the free Personal tier works with no key. Enter one to unlock more modules."
            },
        );
        row(
            ui,
            selling,
            "Choose where you sell",
            "Every marketplace has its own limits. The app checks your products against them, so a \
             listing is not rejected after you have made it.",
        );
    });
}

fn show_providers(app: &mut DpfApp, ui: &mut Ui) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong("AI providers");
            ui.label(
                RichText::new("You need ONE. Add more to route different jobs to different models.")
                    .size(11.0)
                    .color(Color32::from_gray(140)),
            );
        });
        ui.add_space(6.0);

        for p in PROVIDERS {
            let held = !(p.field)(&mut app.config).trim().is_empty();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if held { "✅" } else { "○" })
                        .size(14.0)
                        .color(if held {
                            Color32::from_rgb(110, 200, 130)
                        } else {
                            Color32::from_gray(120)
                        }),
                );
                ui.label(RichText::new(p.label).strong().size(13.0));
                ui.add_sized(
                    [340.0, 22.0],
                    TextEdit::singleline((p.field)(&mut app.config))
                        .hint_text("paste your key - see the link below")
                        .password(true),
                );
                ui.label(
                    RichText::new(p.hint)
                        .size(11.0)
                        .color(Color32::from_gray(140)),
                );
            });
            // Where to actually GET this key. Without this the row just says "paste your key",
            // which is no help to someone who does not have one yet.
            ui.horizontal(|ui| {
                ui.add_space(26.0);
                ui.label(
                    RichText::new(if held { "replace it at:" } else { "get one at:" })
                        .size(10.5)
                        .color(Color32::from_gray(130)),
                );
                ui.hyperlink_to(
                    RichText::new(p.key_url).size(10.5),
                    p.key_url,
                );
            });
            ui.add_space(4.0);
        }

        ui.add_space(4.0);
        ui.label(
            RichText::new(
                "Keys are stored in your own settings file on this machine and are sent only to the \
                 provider you chose. They are never sent anywhere else.",
            )
            .size(11.0)
            .color(Color32::from_gray(140)),
        );
    });
}

fn show_marketplaces(ui: &mut Ui) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong("Where you sell");
            ui.label(
                RichText::new("The limits your products must meet on each marketplace.")
                    .size(11.0)
                    .color(Color32::from_gray(140)),
            );
        });
        ui.add_space(6.0);

        let rows = marketplace_rows();
        if rows.is_empty() {
            ui.colored_label(
                Color32::from_rgb(220, 160, 90),
                "No marketplace rules found. `platform_formats.json` should sit next to the app; \
                 without it the quality checks have nothing to check against.",
            );
            return;
        }

        Grid::new("marketplace_grid")
            .num_columns(5)
            .spacing([16.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                for h in ["Marketplace", "Image", "Tags", "Title limit", "Accepted files"] {
                    ui.label(RichText::new(h).strong().size(11.0));
                }
                ui.end_row();

                for (slug, f) in &rows {
                    // The pretty name when the file gives one, else the slug capitalised — so a
                    // newly added marketplace shows up with no code change.
                    let name = if f.name.trim().is_empty() {
                        let mut c = slug.chars();
                        match c.next() {
                            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                            None => slug.clone(),
                        }
                    } else {
                        f.name.clone()
                    };
                    ui.label(RichText::new(name).strong().size(12.0));
                    ui.label(
                        RichText::new(format!("{}×{}", f.thumbnail_width, f.thumbnail_height))
                            .size(11.0),
                    );
                    ui.label(RichText::new(f.max_tags.to_string()).size(11.0));
                    ui.label(
                        RichText::new(format!("{} chars", f.max_title_length)).size(11.0),
                    );
                    let mut exts: Vec<String> = f
                        .allowed_formats
                        .iter()
                        .map(|s| format!(".{s}"))
                        .collect();
                    exts.sort();
                    ui.label(RichText::new(exts.join(" ")).size(11.0));
                    ui.end_row();
                }
            });

        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "These rules drive the Pre-Publish QC check and the exporters. Editing \
                 `platform_formats.json` (next to the app) changes them; no rebuild needed.",
            )
            .size(11.0)
            .color(Color32::from_gray(140)),
        );
    });
}

fn show_preferences(app: &mut DpfApp, ui: &mut Ui) {
    ui.group(|ui| {
        ui.strong("Preferences");
        ui.add_space(6.0);
        ui.checkbox(&mut app.config.auto_save, "Auto-save my work");
        ui.checkbox(&mut app.config.dark_mode, "Night theme");
        ui.label(
            RichText::new("Switch themes any time from 🌙 / ☀️ in the status bar.")
                .size(11.0)
                .color(Color32::from_gray(140)),
        );
    });
}
