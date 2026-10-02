//! Logo Generator — UI tab for creating AI-generated SVG logos
//! Uses standard DPF egui pattern: show(app, ctx) with CentralPanel

use egui::{CentralPanel, Color32, Vec2, SidePanel};
use crate::app::DpfApp;
use crate::database::Database;
use crate::vector_generator;
use crate::vector_types::*;
use crate::ui::vector_preview;
use rusqlite::params;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Persisted-logo load happens exactly once per process, lazily when the tab is
/// first opened. (A true startup hook would live in `DpfApp::new` in app.rs,
/// which this change is not permitted to touch.)
static LOGOS_LOADED: AtomicBool = AtomicBool::new(false);

pub fn show(app: &mut DpfApp, ctx: &egui::Context) {
    load_saved_logos_once(app);

    let mut notice: Option<(bool, String)> = None;
    let state = &mut app.vector_state;

    CentralPanel::default().show(ctx, |ui| {
        // Left sidebar: saved logos
        SidePanel::left("logo_list")
            .resizable(true)
            .default_width(180.0)
            .show_inside(ui, |ui| {
                ui.heading("Logos");
                ui.separator();
                for logo in &state.saved_logos {
                    if ui.selectable_label(false, &logo.name).clicked() {
                        state.selected_logo_index = state.saved_logos.iter().position(|l| l.id == logo.id);
                    }
                }
                if state.saved_logos.is_empty() {
                    ui.label("No logos yet. Generate one!");
                }
            });

        // Right: params panel
        SidePanel::right("logo_params")
            .resizable(true)
            .default_width(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Parameters");
                ui.separator();
                ui.label("Brand Name:");
                ui.text_edit_singleline(&mut state.brand_name);
                ui.label("Tagline (optional):");
                ui.text_edit_singleline(&mut state.tagline);
                ui.label("Style:");
                ui.horizontal(|ui| {
                    for style in LogoStyle::all() {
                        let selected = state.selected_style == style;
                        if ui.selectable_label(selected, style.label()).clicked() {
                            state.selected_style = style.clone();
                        }
                    }
                });
                ui.label("Colors (comma-separated hex):");
                ui.text_edit_singleline(&mut state.palette_input);
                ui.label("Icon description (optional):");
                ui.text_edit_singleline(&mut state.icon_description);
                ui.add_space(8.0);

                if ui.button("✨ Generate Logo").clicked() {
                    let router = (!app.config.openai_key.is_empty()).then(|| {
                        crate::llm_router::LLMRouter::new(
                            app.config.openai_key.clone(),
                            app.config.anthropic_key.clone(),
                            app.config.google_key.clone(),
                            app.config.deepseek_key.clone(),
                            app.config.moonshot_key.clone(),
                        )
                    });
                    let colors: Vec<String> = state.palette_input.split(',')
                        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    if let Some(ref llm) = router {
                        let req = LogoGenerateRequest {
                            brand_name: state.brand_name.clone(),
                            tagline: if state.tagline.is_empty() { None } else { Some(state.tagline.clone()) },
                            style: state.selected_style.clone(),
                            palette: colors,
                            icon_description: if state.icon_description.is_empty() { None } else { Some(state.icon_description.clone()) },
                        };
                        match vector_generator::generate_logo(llm, &app.runtime, &req) {
                            Ok(gen) => {
                                state.current_logo = Some(Logo {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: state.brand_name.clone(),
                                    style: state.selected_style.clone(),
                                    brand_name: state.brand_name.clone(),
                                    tagline: state.tagline.clone(),
                                    icon_svg: gen.icon_svg,
                                    typography_svg: gen.typography_svg,
                                    full_svg: gen.full_svg,
                                    palette: gen.palette,
                                    favicon_enabled: state.favicon_enabled,
                                    favicon_package: None,
                                    status: "draft".to_string(),
                                    created_at: chrono::Utc::now().to_string(),
                                });
                            }
                            Err(e) => state.error = Some(e),
                        }
                    } else {
                        state.error = Some("LLM router not configured".to_string());
                    }
                }

                ui.checkbox(&mut state.favicon_enabled, "Auto-generate Favicon");

                // --- Favicon package export (wired: was 0 call sites) ----------
                if state.current_logo.is_some()
                    && ui.button("🎁 Export Favicon Package").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        // `is_some()` is checked above, but the native file dialog BLOCKS the UI
                        // thread for an unbounded time — never `.unwrap()` state across that gap.
                        // Re-check and degrade gracefully instead of panicking the whole app.
                        if let Some(logo) = state.current_logo.clone() {
                            match crate::vector_export::export_favicon_package(&logo, &dir) {
                                Ok(pkg) => {
                                    let count = pkg.sizes.len();
                                    if let Some(cur) = state.current_logo.as_mut() {
                                        cur.favicon_enabled = true;
                                        cur.favicon_package = Some(pkg);
                                    }
                                    notice = Some((false, format!(
                                        "Favicon package written to {} ({} PNGs + favicon.ico + apple-touch-icon.png + site.webmanifest)",
                                        dir.display(), count)));
                                }
                                Err(e) => notice = Some((true, format!("Favicon export failed: {}", e))),
                            }
                        }
                    }
                }

                // --- Save (wired: real INSERT + in-memory list) ----------------
                if state.current_logo.is_some()
                    && ui.button("💾 Save Logo").clicked() {
                    if let Some(mut logo) = state.current_logo.clone() {
                        // Honor the "Auto-generate Favicon" checkbox on save.
                        if logo.favicon_enabled && logo.favicon_package.is_none() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                match crate::vector_export::export_favicon_package(&logo, &dir) {
                                    Ok(pkg) => logo.favicon_package = Some(pkg),
                                    Err(e) => notice = Some((true, format!("Favicon export failed: {}", e))),
                                }
                            }
                        }
                        match save_logo_to_db(&app.db, &logo) {
                            Ok(()) => {
                                match state.saved_logos.iter().position(|l| l.id == logo.id) {
                                    Some(pos) => state.saved_logos[pos] = logo.clone(),
                                    None => state.saved_logos.insert(0, logo.clone()),
                                }
                                state.selected_logo_index = state.saved_logos.iter().position(|l| l.id == logo.id);
                                state.current_logo = Some(logo);
                                if notice.is_none() {
                                    notice = Some((false, "Logo saved.".to_string()));
                                }
                            }
                            Err(e) => notice = Some((true, e)),
                        }
                    }
                }

                if let Some((is_err, msg)) = &notice {
                    let color = if *is_err { Color32::RED } else { Color32::from_rgb(80, 200, 120) };
                    ui.colored_label(color, msg);
                }

                if let Some(ref err) = state.error {
                    ui.colored_label(Color32::RED, err);
                }
            });

        // Center: preview area
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref logo) = state.current_logo {
                ui.heading("Preview");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    vector_preview::show_svg_preview(ui, &logo.full_svg, "Logo", Vec2::new(400.0, 500.0));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Logo Generator");
                    ui.label("Enter brand details and click Generate");
                });
            }
        });
    });
}

// ── Persistence helpers (logos table already exists in database.rs) ─────────

fn load_saved_logos_once(app: &mut DpfApp) {
    if LOGOS_LOADED.swap(true, Ordering::SeqCst) {
        return;
    }
    match load_logos(&app.db) {
        Ok(logos) => {
            if !logos.is_empty() {
                app.vector_state.selected_logo_index = Some(0);
            }
            app.vector_state.saved_logos = logos;
        }
        Err(e) => app.vector_state.error = Some(format!("Failed to load saved logos: {}", e)),
    }
}

fn load_logos(db: &Arc<Database>) -> Result<Vec<Logo>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, brand_name, tagline, style, palette, icon_svg, typography_svg, \
         full_svg, favicon_enabled, favicon_package_json, status, created_at \
         FROM logos ORDER BY created_at DESC",
    ).map_err(|e| format!("logos query prepare failed: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(Logo {
            id: row.get(0)?,
            name: row.get(1)?,
            brand_name: row.get(2)?,
            tagline: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            style: serde_json::from_str(&row.get::<_, Option<String>>(4)?.unwrap_or_default())
                .unwrap_or(LogoStyle::Modern),
            palette: serde_json::from_str(&row.get::<_, Option<String>>(5)?.unwrap_or_default())
                .unwrap_or_default(),
            icon_svg: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            typography_svg: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
            full_svg: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
            favicon_enabled: row.get::<_, Option<i64>>(9)?.unwrap_or(0) != 0,
            favicon_package: row.get::<_, Option<String>>(10)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            status: row.get::<_, Option<String>>(11)?.unwrap_or_else(|| "draft".to_string()),
            created_at: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
        })
    }).map_err(|e| format!("logos query failed: {}", e))?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn save_logo_to_db(db: &Arc<Database>, logo: &Logo) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO logos \
         (id, name, brand_name, tagline, style, palette, icon_svg, typography_svg, full_svg, \
          favicon_enabled, favicon_package_json, status, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            logo.id,
            logo.name,
            logo.brand_name,
            logo.tagline,
            serde_json::to_string(&logo.style).unwrap_or_default(),
            serde_json::to_string(&logo.palette).unwrap_or_else(|_| "[]".to_string()),
            logo.icon_svg,
            logo.typography_svg,
            logo.full_svg,
            logo.favicon_enabled as i64,
            logo.favicon_package.as_ref().and_then(|p| serde_json::to_string(p).ok()),
            logo.status,
            logo.created_at,
        ],
    ).map_err(|e| format!("Failed to save logo: {}", e))?;
    Ok(())
}
