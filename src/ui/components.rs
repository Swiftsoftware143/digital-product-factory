//! Reusable UI components for speed and consistency

use egui::*;

/// Fast button with icon and label
#[allow(dead_code)]
pub fn icon_button(ui: &mut Ui, icon: &str, label: &str) -> Response {
    ui.button(format!("{} {}", icon, label))
}

/// Card container with consistent styling
pub fn card<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Frame::group(ui.style())
        .fill(Color32::from_gray(40))
        .stroke(Stroke::new(1.0, Color32::from_gray(60)))
        .show(ui, content)
}

/// Status badge (colored pill)
#[allow(dead_code)]
pub fn status_badge(ui: &mut Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.fonts(|f| f.glyph_width(&egui::FontId::proportional(14.0), ' ') * text.len() as f32 + 16.0), 20.0),
        Sense::hover()
    );
    
    ui.painter().rect_filled(
        rect,
        4.0,
        color.gamma_multiply(0.2)
    );
    
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        TextStyle::Body.resolve(ui.style()),
        color
    );
}

/// Loading spinner (lightweight)
#[allow(dead_code)]
pub fn spinner(ui: &mut Ui) {
    ui.spinner();
}

/// Search input with icon
#[allow(dead_code)]
pub fn search_input(ui: &mut Ui, query: &mut String) -> Response {
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.add(egui::TextEdit::singleline(query).hint_text("Search..."))
    }).response
}

/// Stat card for dashboard
#[allow(dead_code)]
pub fn stat_card(ui: &mut Ui, label: &str, value: &str, change: Option<&str>) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(120.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(value).size(28.0).strong());
            ui.label(label);
            if let Some(chg) = change {
                ui.colored_label(Color32::GREEN, chg);
            }
        });
    });
}

/// Confirm dialog
#[allow(dead_code)]
pub fn confirm_dialog(ctx: &Context, title: &str, message: &str, on_confirm: impl FnOnce()) {
    Window::new(title)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(message);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    // Close dialog
                }
                if ui.button("Confirm").clicked() {
                    on_confirm();
                }
            });
        });
}

/// Toast notification (brief popup)
pub fn toast(ctx: &Context, message: &str, duration_secs: f32) {
    // Simple implementation - could be enhanced with animation
    Area::new("toast")
        .anchor(Align2::RIGHT_BOTTOM, vec2(-20.0, -20.0))
        .show(ctx, |ui| {
            Frame::popup(ui.style())
                .fill(Color32::from_rgb(50, 50, 50))
                .show(ui, |ui| {
                    ui.label(message);
                });
        });
}
