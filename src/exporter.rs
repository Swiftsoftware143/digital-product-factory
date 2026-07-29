//! Export module for products - PDF, DOCX, XLSX, ZIP support

use std::fs;
use std::path::Path;
use crate::product_generator::GeneratedProduct;
use crate::templates::OutputFormat;

pub struct Exporter;

impl Exporter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn export(&self, product: &GeneratedProduct, output_dir: &str) -> Result<String, String> {
        let output_path = format!("{}/{}", output_dir, self.sanitize_filename(&product.name));
        
        match product.format {
            OutputFormat::Markdown => self.export_markdown(product, &output_path),
            OutputFormat::Html => self.export_html(product, &output_path),
            OutputFormat::Pdf => self.export_pdf(product, &output_path),
            OutputFormat::Docx => self.export_docx(product, &output_path),
            OutputFormat::Xlsx => self.export_xlsx(product, &output_path),
            OutputFormat::Json => self.export_json(product, &output_path),
        }
    }
    
    fn export_markdown(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        let filepath = format!("{}.md", path);
        fs::write(&filepath, &product.content)
            .map_err(|e| format!("Failed to write markdown: {}", e))?;
        Ok(filepath)
    }
    
    fn export_html(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ color: #333; }}
        h2 {{ color: #555; border-bottom: 1px solid #ddd; padding-bottom: 5px; }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
            product.name,
            self.markdown_to_html(&product.content)
        );
        
        let filepath = format!("{}.html", path);
        fs::write(&filepath, html)
            .map_err(|e| format!("Failed to write HTML: {}", e))?;
        Ok(filepath)
    }
    
    fn export_pdf(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        // For now, export as HTML that can be printed to PDF
        // In production, use a PDF library like printpdf or headless Chrome
        self.export_html(product, &format!("{}_print", path))
    }
    
    fn export_docx(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        // For now, export as markdown with .docx extension
        // In production, use docx-rs library
        let filepath = format!("{}.docx.md", path);
        fs::write(&filepath, &product.content)
            .map_err(|e| format!("Failed to write DOCX placeholder: {}", e))?;
        Ok(filepath)
    }
    
    fn export_xlsx(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        // For now, export as CSV
        // In production, use rust_xlsxwriter
        let filepath = format!("{}.csv", path);
        fs::write(&filepath, &product.content)
            .map_err(|e| format!("Failed to write CSV: {}", e))?;
        Ok(filepath)
    }
    
    fn export_json(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        let json = serde_json::json!({
            "name": product.name,
            "template_id": product.template_id,
            "content": product.content,
            "created_at": product.created_at,
            "metadata": product.metadata,
        });
        
        let filepath = format!("{}.json", path);
        fs::write(&filepath, serde_json::to_string_pretty(&json).unwrap())
            .map_err(|e| format!("Failed to write JSON: {}", e))?;
        Ok(filepath)
    }
    
    pub fn export_zip(&self, products: &[GeneratedProduct], output_path: &str) -> Result<String, String> {
        use zip::write::FileOptions;
        use std::io::Write;
        
        let file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create ZIP: {}", e))?;
        let mut zip = zip::ZipWriter::new(file);
        
        for product in products {
            let filename = format!("{}.md", self.sanitize_filename(&product.name));
            zip.start_file(&filename, FileOptions::default())
                .map_err(|e| format!("Failed to add file to ZIP: {}", e))?;
            zip.write_all(product.content.as_bytes())
                .map_err(|e| format!("Failed to write to ZIP: {}", e))?;
        }
        
        zip.finish()
            .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
        
        Ok(output_path.to_string())
    }
    
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "_")
    }
    
    fn markdown_to_html(&self, markdown: &str) -> String {
        // Simple markdown to HTML conversion
        // In production, use a proper markdown parser like pulldown-cmark
        let mut html = markdown.to_string();
        
        // Headers
        for i in (1..=6).rev() {
            let prefix = "#".repeat(i);
            html = html.replace(&format!("{} ", prefix), &format!("<h{}>", i));
            html = html.replace(&format!("\n{} ", prefix), &format!("</h{}>\n<h{}>", i, i));
        }
        
        // Bold
        html = html.replace("**", "<strong>");
        // This is a simple replacement - would need proper parsing
        
        // Paragraphs
        html = html.replace("\n\n", "</p><p>");
        
        format!("<p>{}</p>", html)
    }
}
