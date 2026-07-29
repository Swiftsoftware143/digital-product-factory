//! Export functions for Logo Generator & Vector Generator
//! SVG, PNG, ICO, favicon package export — follows same pattern as advert_export.rs

use crate::vector_renderer;
use crate::vector_types::{FaviconPackage, Logo, VectorAsset};
use std::path::{Path, PathBuf};

/// Export a logo's full SVG to file
pub fn export_logo_svg(logo: &Logo, path: &Path) -> Result<PathBuf, String> {
    std::fs::write(path, &logo.full_svg)
        .map_err(|e| format!("Failed to write SVG: {}", e))?;
    Ok(path.to_path_buf())
}

/// Generate and write a complete favicon package
pub fn export_favicon_package(logo: &Logo, output_dir: &Path) -> Result<FaviconPackage, String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;

    let sizes: [(u32, u32); 5] = [(16, 16), (32, 32), (48, 48), (192, 192), (512, 512)];
    let pngs = vector_renderer::render_multi_size(&logo.full_svg, &sizes)?;

    // Write each PNG
    let mut icon_paths = Vec::new();
    for &(w, h, ref img) in &pngs {
        let filename = format!("favicon-{}x{}.png", w, h);
        let path = output_dir.join(&filename);
        img.save(&path)
            .map_err(|e| format!("Failed to save PNG: {}", e))?;
        icon_paths.push((w, h, filename));
    }

    // Generate ICO (16, 32, 48)
    let ico_bytes = vector_renderer::generate_ico(&pngs[..3])?;
    let ico_path = output_dir.join("favicon.ico");
    std::fs::write(&ico_path, &ico_bytes)
        .map_err(|e| format!("Failed to write ICO: {}", e))?;

    // Webmanifest
    let webmanifest_paths: Vec<(u32, u32, String)> = icon_paths.iter()
        .map(|(w, h, fname)| (*w, *h, fname.clone())).collect();
    let webmanifest = vector_renderer::generate_webmanifest(&logo.brand_name, &webmanifest_paths);
    let manifest_path = output_dir.join("site.webmanifest");
    std::fs::write(&manifest_path, &webmanifest)
        .map_err(|e| format!("Failed to write webmanifest: {}", e))?;

    // Apple touch icon (180x180)
    let apple_icon = vector_renderer::render_to_image(&logo.full_svg, 1.8)?;
    let apple_path = output_dir.join("apple-touch-icon.png");
    apple_icon.save(&apple_path)
        .map_err(|e| format!("Failed to save apple touch icon: {}", e))?;

    Ok(FaviconPackage {
        enabled: true,
        sizes: sizes.to_vec(),
        apple_touch_icon: Some("apple-touch-icon.png".to_string()),
        ico_file: Some(ico_bytes),
        webmanifest_json: Some(webmanifest),
    })
}

/// Export a vector asset as SVG
pub fn export_vector_svg(asset: &VectorAsset, path: &Path) -> Result<PathBuf, String> {
    std::fs::write(path, &asset.svg_content)
        .map_err(|e| format!("Failed to write SVG: {}", e))?;
    Ok(path.to_path_buf())
}

/// Export a vector asset as PNG at specified size
pub fn export_vector_png(asset: &VectorAsset, path: &Path, size: u32) -> Result<PathBuf, String> {
    let img = vector_renderer::render_to_image(&asset.svg_content, size as f32 / 100.0)?;
    let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    resized.save(path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;
    Ok(path.to_path_buf())
}

/// Create a ZIP archive of all export files
pub fn export_zip(files: &[(String, Vec<u8>)], output_path: &Path) -> Result<PathBuf, String> {
    use std::io::Write;
    let file = std::fs::File::create(output_path)
        .map_err(|e| format!("Failed to create ZIP: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);

    for (name, data) in files {
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(name, options)
            .map_err(|e| format!("ZIP start_file error: {}", e))?;
        zip.write_all(data)
            .map_err(|e| format!("ZIP write error: {}", e))?;
    }

    zip.finish()
        .map_err(|e| format!("ZIP finish error: {}", e))?;
    Ok(output_path.to_path_buf())
}
