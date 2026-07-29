//! SVG rendering engine — converts SVG to PNG/ICO using resvg + usvg
//! Follows spec: render_to_image, render_multi_size, generate_ico, generate_webmanifest

use image::{DynamicImage, ImageBuffer, Rgba};
use usvg_parser::TreeParsing;
use resvg::tiny_skia::Pixmap;

// use usvg::TreeParsing via resvg re-export — imported via resvg
use resvg::Tree;

/// Render an SVG string to a DynamicImage at the given scale
pub fn render_to_image(svg_data: &str, scale: f32) -> Result<DynamicImage, String> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_data, &opt)
        .map_err(|e| format!("SVG parse error: {}", e))?;

    let size = tree.size;
    let width = (size.width() * scale).ceil() as u32;
    let height = (size.height() * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width, height)
        .ok_or("Failed to create pixmap")?;

    let render_tree = Tree::from_usvg(&tree);
    render_tree.render(
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, pixmap.data().to_vec())
        .ok_or("Failed to create image buffer")?;

    Ok(DynamicImage::ImageRgba8(img))
}

/// Render SVG at multiple sizes — returns Vec<(width, height, DynamicImage)>
pub fn render_multi_size(svg_data: &str, sizes: &[(u32, u32)]) -> Result<Vec<(u32, u32, DynamicImage)>, String> {
    let mut results = Vec::new();
    for &(w, h) in sizes {
        let scale_x = w as f32 / 100.0; // assumes 100px base viewBox
        let img = render_to_image(svg_data, scale_x)?;
        let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
        results.push((w, h, resized));
    }
    Ok(results)
}

/// Generate .ico file bytes from a set of PNG images at standard sizes
pub fn generate_ico(pngs: &[(u32, u32, DynamicImage)]) -> Result<Vec<u8>, String> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &(w, _h, ref img) in pngs {
        let rgba = img.to_rgba8();
        let icon_img = ico::IconImage::from_rgba_data(w, w, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_img)
            .map_err(|e| format!("ICO encode error: {}", e))?);
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    icon_dir.write(&mut buf)
        .map_err(|e| format!("ICO write error: {}", e))?;
    Ok(buf.into_inner())
}

/// Generate webmanifest JSON string for PWA favicons
pub fn generate_webmanifest(name: &str, icons: &[(u32, u32, String)]) -> String {
    let icon_entries: Vec<serde_json::Value> = icons.iter().map(|(w, _h, path)| {
        serde_json::json!({
            "src": path,
            "sizes": format!("{}x{}", w, w),
            "type": "image/png"
        })
    }).collect();

    serde_json::json!({
        "name": name,
        "short_name": name,
        "icons": icon_entries,
        "display": "standalone",
        "start_url": "/",
    }).to_string()
}
