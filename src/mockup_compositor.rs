//! Mockup Compositor â€” DropMock-style image mockup creation
//!
//! Features:
//! - Load product images (PNG/JPG)
//! - Load scene templates (PNG with alpha guide regions)
//! - Drag/resize overlay placement
//! - Export composite as PNG or JPG
//!
//! Tier: Agency+ (gated)

use image::{DynamicImage, ImageBuffer, GenericImageView, imageops};
use std::path::Path;

/// A template scene with defined overlay regions
pub struct SceneTemplate {
    pub name: String,
    pub path: String,
    pub preview: Option<DynamicImage>,
    /// Guide regions: areas where product images should be placed
    pub guides: Vec<GuideRegion>,
}

/// A guide region defining where to place a product image
#[derive(Clone, Debug)]
pub struct GuideRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub label: String,
    pub rotation_degrees: f32,
}

/// A loaded product image ready for compositing
pub struct ProductLayer {
    pub image: DynamicImage,
    pub path: String,
    pub name: String,
}

/// Compositing result
pub struct CompositeResult {
    pub image: DynamicImage,
    pub width: u32,
    pub height: u32,
}

/// Main compositor engine
pub struct MockupCompositor {
    pub templates: Vec<SceneTemplate>,
    pub product: Option<ProductLayer>,
    pub current_template: Option<usize>,
    pub selected_guide: Option<usize>,
    pub scale: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub mesh_path: Option<String>,
    pub has_valid_license: bool,
}

impl Default for MockupCompositor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockupCompositor {
    pub fn new() -> Self {
        let templates = Self::discover_templates();
        Self {
            templates,
            product: None,
            current_template: None,
            selected_guide: None,
            scale: 1.0,
            offset_x: 0,
            offset_y: 0,
            mesh_path: None,
            has_valid_license: false,
        }
    }

    /// Scan the mockups directory for template files
    fn discover_templates() -> Vec<SceneTemplate> {
        let mut templates = Vec::new();
        let mockup_dir = Path::new("mockups");
        if mockup_dir.exists() && mockup_dir.is_dir() {
            for entry in std::fs::read_dir(mockup_dir).ok().into_iter().flatten().flatten() {
                let path = entry.path();
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("template")
                    .to_string();
                if let Ok(img) = image::open(&path) {
                    templates.push(SceneTemplate {
                        name,
                        path: path.to_string_lossy().to_string(),
                        preview: Some(img),
                        guides: vec![
                            GuideRegion {
                                x: 0, y: 0,
                                width: 100, height: 100,
                                label: "Default".into(),
                                rotation_degrees: 0.0,
                            }
                        ],
                    });
                }
            }
        }
        templates
    }

    /// Load a product image from file
    pub fn load_product(&mut self, path: &str) -> Result<(), String> {
        let img = image::open(path).map_err(|e| format!("Failed to load image: {}", e))?;
        let name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("product")
            .to_string();
        self.product = Some(ProductLayer {
            image: img,
            path: path.to_string(),
            name,
        });
        Ok(())
    }

    /// Resize product to fit guide region while maintaining aspect ratio
    pub fn fit_to_guide(&self, guide: &GuideRegion) -> (u32, u32) {
        if let Some(ref product) = self.product {
            let (pw, ph) = product.image.dimensions();
            let gw = guide.width;
            let gh = guide.height;
            let scale = f64::min(gw as f64 / pw as f64, gh as f64 / ph as f64);
            (
                (pw as f64 * scale) as u32,
                (ph as f64 * scale) as u32,
            )
        } else {
            (0, 0)
        }
    }

    /// Composite the product image onto the template at the selected guide position
    pub fn compose(&self, template_idx: usize, guide_idx: usize) -> Result<CompositeResult, String> {
        let template = self.templates.get(template_idx)
            .ok_or("Template not found")?;
        let template_img = template.preview.as_ref()
            .ok_or("Template has no image loaded")?;
        let product = self.product.as_ref()
            .ok_or("No product image loaded")?;
        let guide = template.guides.get(guide_idx)
            .ok_or("Guide not found")?;

        let (tw, th) = template_img.dimensions();
        let mut canvas = ImageBuffer::new(tw, th);

        // Copy template onto canvas
        for (x, y, pixel) in template_img.pixels() {
            canvas.put_pixel(x, y, pixel);
        }

        // Calculate placement within guide region
        let (fw, fh) = self.fit_to_guide(guide);
        let resized = imageops::resize(
            &product.image,
            fw.max(1), fh.max(1),
            imageops::FilterType::Lanczos3
        );

        // Center in guide region with offsets
        let cx = guide.x + (guide.width.saturating_sub(fw)) / 2;
        let cy = guide.y + (guide.height.saturating_sub(fh)) / 2;
        let ox = ((cx as i32) + self.offset_x).max(0) as u32;
        let oy = ((cy as i32) + self.offset_y).max(0) as u32;

        imageops::overlay(&mut canvas, &resized, ox as i64, oy as i64);

        Ok(CompositeResult {
            image: DynamicImage::ImageRgba8(canvas),
            width: tw,
            height: th,
        })
    }

    /// Export composited image to file
    pub fn export(&self, result: &CompositeResult, path: &str, format: &str) -> Result<(), String> {
        match format.to_lowercase().as_str() {
            "png" => result.image.save(path)
                .map_err(|e| format!("Failed to save PNG: {}", e)),
            "jpg" | "jpeg" => {
                let rgb_img = result.image.to_rgb8();
                rgb_img.save(path)
                    .map_err(|e| format!("Failed to save JPG: {}", e))
            }
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }

    /// Get list of built-in scene names
    pub fn scene_names(&self) -> Vec<String> {
        if self.templates.is_empty() {
            vec!["No templates found â€” place PNG files in ./mockups/".into()]
        } else {
            self.templates.iter().map(|t| t.name.clone()).collect()
        }
    }

    /// Set license validity
    pub fn set_license_valid(&mut self, valid: bool) {
        self.has_valid_license = valid;
    }

    /// Check if the user can use this feature (Agency+ tier)
    pub fn can_use_compositor(&self) -> bool {
        self.has_valid_license
    }
}