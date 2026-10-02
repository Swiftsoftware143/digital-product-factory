//! Vector Generator — UI tab for AI-generated SVG icons, illustrations, etc.
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

/// Persisted-vector load happens exactly once per process, lazily when the tab
/// is first opened. (A true startup hook lives in app.rs, which is out of scope.)
static VECTORS_LOADED: AtomicBool = AtomicBool::new(false);

pub fn show(app: &mut DpfApp, ctx: &egui::Context) {
    load_saved_vectors_once(app);

    let mut notice: Option<(bool, String)> = None;
    let state = &mut app.vector_state;

    CentralPanel::default().show(ctx, |ui| {
        // Left sidebar
        SidePanel::left("vector_list")
            .resizable(true)
            .default_width(180.0)
            .show_inside(ui, |ui| {
                ui.heading("Vectors");
                ui.separator();
                for asset in &state.saved_vectors {
                    if ui.selectable_label(false, &asset.name).clicked() {
                        state.selected_vector_index = state.saved_vectors.iter().position(|a| a.id == asset.id);
                    }
                }
                if state.saved_vectors.is_empty() {
                    ui.label("No vectors yet.");
                }
            });

        // Right: params
        SidePanel::right("vector_params")
            .resizable(true)
            .default_width(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Parameters");
                ui.separator();
                ui.label("Name:");
                ui.text_edit_singleline(&mut state.vector_name);
                ui.label("Category:");
                ui.horizontal(|ui| {
                    for cat in VectorCategory::all() {
                        let selected = state.selected_category == cat;
                        if ui.selectable_label(selected, cat.label()).clicked() {
                            state.selected_category = cat.clone();
                        }
                    }
                });
                ui.label("Prompt:");
                ui.text_edit_multiline(&mut state.prompt);
                ui.label("Style (optional):");
                ui.text_edit_singleline(&mut state.style_input);
                ui.label("Colors (comma-separated hex):");
                ui.text_edit_singleline(&mut state.palette_input);
                ui.add_space(8.0);

                if ui.button("🎨 Generate Vector").clicked() {
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
                        let req = VectorGenerateRequest {
                            category: state.selected_category.clone(),
                            prompt: state.prompt.clone(),
                            style: if state.style_input.is_empty() { None } else { Some(state.style_input.clone()) },
                            palette: colors,
                        };
                        match vector_generator::generate_vector(llm, &app.runtime, &req) {
                            Ok(gen) => {
                                state.current_vector = Some(VectorAsset {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: state.vector_name.clone(),
                                    category: state.selected_category.clone(),
                                    prompt: state.prompt.clone(),
                                    svg_content: gen.svg_content,
                                    palette: gen.palette,
                                    view_box: gen.view_box,
                                    export_formats: vec!["svg".to_string(), "png".to_string()],
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

                // --- Export buttons (wired: export_vector_svg/png had 0 call sites) ---
                if let Some(asset) = state.current_vector.clone() {
                    ui.horizontal(|ui| {
                        if ui.button("⬇ Export SVG").clicked() {
                            let default_name = format!("{}.svg", sanitize_name(&asset.name));
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&default_name)
                                .add_filter("SVG", &["svg"])
                                .save_file()
                            {
                                match crate::vector_export::export_vector_svg(&asset, &path) {
                                    Ok(p) => notice = Some((false, format!("Exported SVG: {}", p.display()))),
                                    Err(e) => notice = Some((true, e)),
                                }
                            }
                        }
                        if ui.button("⬇ Export PNG").clicked() {
                            let default_name = format!("{}.png", sanitize_name(&asset.name));
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&default_name)
                                .add_filter("PNG", &["png"])
                                .save_file()
                            {
                                match crate::vector_export::export_vector_png(&asset, &path, 512) {
                                    Ok(p) => notice = Some((false, format!("Exported PNG (512x512): {}", p.display()))),
                                    Err(e) => notice = Some((true, e)),
                                }
                            }
                        }
                    });
                }

                // --- Save (wired: real INSERT + in-memory list) ----------------
                if state.current_vector.is_some()
                    && ui.button("💾 Save").clicked() {
                    if let Some(asset) = state.current_vector.clone() {
                        match save_vector_to_db(&app.db, &asset) {
                            Ok(()) => {
                                match state.saved_vectors.iter().position(|a| a.id == asset.id) {
                                    Some(pos) => state.saved_vectors[pos] = asset.clone(),
                                    None => state.saved_vectors.insert(0, asset.clone()),
                                }
                                state.selected_vector_index = state.saved_vectors.iter().position(|a| a.id == asset.id);
                                if notice.is_none() {
                                    notice = Some((false, "Vector saved.".to_string()));
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

        // Center: preview
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref asset) = state.current_vector {
                ui.heading(&asset.name);
                ui.separator();
                ui.label(format!("Category: {}", asset.category.label()));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    vector_preview::show_svg_preview(ui, &asset.svg_content, "Vector", Vec2::new(400.0, 400.0));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Vector Generator");
                    ui.label("Enter a prompt and click Generate");
                });
            }
        });
    });
}

/// Keep a user-supplied name usable as a filename.
fn sanitize_name(name: &str) -> String {
    let cleaned: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if cleaned.trim_matches('_').is_empty() { "vector".to_string() } else { cleaned }
}

// ── Persistence helpers (vector_assets table already exists in database.rs) ──

fn load_saved_vectors_once(app: &mut DpfApp) {
    if VECTORS_LOADED.swap(true, Ordering::SeqCst) {
        return;
    }
    match load_vectors(&app.db) {
        Ok(vectors) => {
            if !vectors.is_empty() {
                app.vector_state.selected_vector_index = Some(0);
            }
            app.vector_state.saved_vectors = vectors;
        }
        Err(e) => app.vector_state.error = Some(format!("Failed to load saved vectors: {}", e)),
    }
}

fn load_vectors(db: &Arc<Database>) -> Result<Vec<VectorAsset>, String> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, category, prompt, svg_content, palette, view_box, \
         export_formats, status, created_at \
         FROM vector_assets ORDER BY created_at DESC",
    ).map_err(|e| format!("vector_assets query prepare failed: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(VectorAsset {
            id: row.get(0)?,
            name: row.get(1)?,
            category: serde_json::from_str(&row.get::<_, Option<String>>(2)?.unwrap_or_default())
                .unwrap_or(VectorCategory::Icon),
            prompt: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            svg_content: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            palette: serde_json::from_str(&row.get::<_, Option<String>>(5)?.unwrap_or_default())
                .unwrap_or_default(),
            view_box: row.get::<_, Option<String>>(6)?
                .unwrap_or_else(|| "0 0 200 200".to_string()),
            export_formats: serde_json::from_str(&row.get::<_, Option<String>>(7)?.unwrap_or_default())
                .unwrap_or_default(),
            status: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "draft".to_string()),
            created_at: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
        })
    }).map_err(|e| format!("vector_assets query failed: {}", e))?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn save_vector_to_db(db: &Arc<Database>, asset: &VectorAsset) -> Result<(), String> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO vector_assets \
         (id, name, category, prompt, svg_content, palette, view_box, export_formats, status, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            asset.id,
            asset.name,
            serde_json::to_string(&asset.category).unwrap_or_default(),
            asset.prompt,
            asset.svg_content,
            serde_json::to_string(&asset.palette).unwrap_or_else(|_| "[]".to_string()),
            asset.view_box,
            serde_json::to_string(&asset.export_formats).unwrap_or_else(|_| "[]".to_string()),
            asset.status,
            asset.created_at,
        ],
    ).map_err(|e| format!("Failed to save vector: {}", e))?;
    Ok(())
}
