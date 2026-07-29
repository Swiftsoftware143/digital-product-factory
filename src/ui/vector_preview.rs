//! Shared SVG preview canvas for Logo Generator & Vector Generator
//! Renders SVG content inside an egui Image widget (egui 0.24 API)

use egui::{ColorImage, TextureHandle, TextureOptions};
use usvg_parser::TreeParsing;
// use usvg::TreeParsing via resvg re-export — imported via resvg

/// Render SVG to a texture for display in egui
pub fn svg_to_texture(svg_data: &str, ctx: &egui::Context) -> Option<TextureHandle> {
    
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_data, &opt).ok()?;

    let size = tree.size;
    let width = (size.width() * 2.0).ceil() as u32;
    let height = (size.height() * 2.0).ceil() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
    let render_tree = resvg::Tree::from_usvg(&tree);
    render_tree.render(
        resvg::tiny_skia::Transform::from_scale(2.0, 2.0),
        &mut pixmap.as_mut(),
    );

    let color_img = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        pixmap.data(),
    );

    Some(ctx.load_texture(
        "svg-preview",
        color_img,
        TextureOptions::default(),
    ))
}

/// Show a preview of the SVG with proper sizing
pub fn show_svg_preview(ui: &mut egui::Ui, svg_data: &str, _label: &str, max_size: egui::Vec2) {
    if svg_data.is_empty() {
        ui.label("No preview available");
        return;
    }

    if let Some(tex) = svg_to_texture(svg_data, ui.ctx()) {
        let img_size = tex.size_vec2();
        let scale = (max_size.x / img_size.x).min(max_size.y / img_size.y).min(2.0);
        let scaled_size = img_size * scale;
        ui.add(egui::Image::new(&tex).max_width(scaled_size.x).max_height(scaled_size.y));
    } else {
        ui.monospace(format!("<svg render failed> {}", &svg_data[..svg_data.len().min(200)]));
    }
}
