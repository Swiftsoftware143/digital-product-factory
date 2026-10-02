//! Export module for products — Markdown, HTML, PDF, DOCX, XLSX, JSON, ZIP.
//!
//! PDF, DOCX and XLSX previously wrote files with the *wrong format inside*: "PDF" produced
//! a print-ready HTML file, "DOCX" produced a `.docx.md` Markdown file, and "XLSX" produced
//! a CSV. The extensions lied. They now emit genuine files in those formats.

use std::fs;

use crate::product_generator::GeneratedProduct;
use crate::templates::OutputFormat;

/// A content line classified for document rendering.
enum Block {
    Heading(u8, String),
    Bullet(String),
    Body(String),
    Blank,
}

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
        fs::write(&filepath, html).map_err(|e| format!("Failed to write HTML: {}", e))?;
        Ok(filepath)
    }

    /// A genuine PDF 1.4 document (Helvetica, paginated).
    ///
    /// Hand-rolled rather than adding a PDF crate: the payload is plain text and a text-only
    /// PDF is a small, fully specified structure. The file is assembled as bytes while the
    /// offset of every object is recorded, so the cross-reference table is correct by
    /// construction rather than by hand-counting.
    fn export_pdf(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        let filepath = format!("{}.pdf", path);
        let bytes = build_pdf(&product.name, &product.content);
        fs::write(&filepath, bytes).map_err(|e| format!("Failed to write PDF: {}", e))?;
        Ok(filepath)
    }

    /// A genuine Word document with headings, paragraphs and bullet lists (docx-rs).
    fn export_docx(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        use docx_rs::*;

        let filepath = format!("{}.docx", path);
        let mut docx = Docx::new();

        for block in parse_blocks(&product.name, &product.content) {
            docx = match block {
                Block::Heading(level, text) => {
                    let style = format!("Heading{}", level.clamp(1, 6));
                    docx.add_paragraph(
                        Paragraph::new()
                            .style(style.as_str())
                            .add_run(Run::new().add_text(text)),
                    )
                }
                Block::Bullet(text) => docx
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!("• {}", text)))),
                Block::Body(text) => {
                    docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)))
                }
                Block::Blank => docx.add_paragraph(Paragraph::new()),
            };
        }

        let file =
            fs::File::create(&filepath).map_err(|e| format!("Failed to create DOCX: {}", e))?;
        docx.build()
            .pack(file)
            .map_err(|e| format!("Failed to write DOCX: {:?}", e))?;
        Ok(filepath)
    }

    /// A genuine Excel workbook (rust_xlsxwriter).
    ///
    /// Markdown table rows become spreadsheet rows (split on `|`); anything else becomes a
    /// single-cell row, so prose content still lands in a usable grid rather than one blob.
    fn export_xlsx(&self, product: &GeneratedProduct, path: &str) -> Result<String, String> {
        use rust_xlsxwriter::{Format, Workbook};

        let filepath = format!("{}.xlsx", path);
        let mut workbook = Workbook::new();

        let title_fmt = Format::new().set_bold();
        let head_fmt = Format::new().set_bold().set_background_color("#DDDDDD");
        let plain_fmt = Format::new();

        let sheet = workbook.add_worksheet();
        sheet
            .set_name("Product")
            .map_err(|e| format!("Failed to name sheet: {:?}", e))?;

        sheet
            .write_with_format(0, 0, product.name.as_str(), &title_fmt)
            .map_err(|e| format!("Failed to write XLSX: {:?}", e))?;
        sheet
            .write_with_format(
                1,
                0,
                format!("Format: {}", format_name(&product.format)),
                &plain_fmt,
            )
            .map_err(|e| format!("Failed to write XLSX: {:?}", e))?;

        let mut row: u32 = 3;
        for (i, cells) in rows_from_content(&product.content).iter().enumerate() {
            for (c, cell) in cells.iter().enumerate() {
                let cell = cell.trim();
                if cell.is_empty() {
                    continue;
                }
                let fmt = if i == 0 { &head_fmt } else { &plain_fmt };
                sheet
                    .write_with_format(row, c as u16, cell, fmt)
                    .map_err(|e| format!("Failed to write XLSX: {:?}", e))?;
            }
            row += 1;
        }

        workbook
            .save(&filepath)
            .map_err(|e| format!("Failed to save XLSX: {:?}", e))?;
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

    pub fn export_zip(
        &self,
        products: &[GeneratedProduct],
        output_path: &str,
    ) -> Result<String, String> {
        use std::io::Write;
        use zip::write::FileOptions;

        let file =
            fs::File::create(output_path).map_err(|e| format!("Failed to create ZIP: {}", e))?;
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
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .replace(' ', "_")
    }

    fn markdown_to_html(&self, markdown: &str) -> String {
        let mut html = markdown.to_string();

        for i in (1..=6).rev() {
            let prefix = "#".repeat(i);
            html = html.replace(&format!("{} ", prefix), &format!("<h{}>", i));
            html = html.replace(&format!("\n{} ", prefix), &format!("</h{}>\n<h{}>", i, i));
        }

        html = html.replace("**", "<strong>");
        html = html.replace("\n\n", "</p><p>");

        format!("<p>{}</p>", html)
    }
}

/// Human-readable name for an output format (used in exported metadata rows).
fn format_name(f: &OutputFormat) -> &'static str {
    match f {
        OutputFormat::Markdown => "Markdown",
        OutputFormat::Html => "HTML",
        OutputFormat::Pdf => "PDF",
        OutputFormat::Docx => "Word (DOCX)",
        OutputFormat::Xlsx => "Excel (XLSX)",
        OutputFormat::Json => "JSON",
    }
}

/// Strip markdown emphasis markers for plain-text targets.
fn strip_emphasis(s: &str) -> String {
    s.replace("**", "").replace("__", "")
}

/// Split markdown-ish content into typed blocks for document rendering.
fn parse_blocks(title: &str, content: &str) -> Vec<Block> {
    let mut out = vec![Block::Heading(1, title.to_string())];

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push(Block::Blank);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            out.push(Block::Heading(3, rest.to_string()));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            out.push(Block::Heading(2, rest.to_string()));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            out.push(Block::Heading(1, rest.to_string()));
        } else if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            out.push(Block::Bullet(strip_emphasis(rest)));
        } else {
            out.push(Block::Body(strip_emphasis(trimmed)));
        }
    }

    out
}

/// Rows for the spreadsheet: markdown table rows split on `|`, otherwise one cell per line.
fn rows_from_content(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        // Skip markdown table alignment rows such as |---|---|
        if t.starts_with('|') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
            continue;
        }
        if t.starts_with('|') && t.ends_with('|') && t.len() > 1 {
            rows.push(
                t[1..t.len() - 1]
                    .split('|')
                    .map(|c| strip_emphasis(c.trim()))
                    .collect(),
            );
        } else {
            rows.push(vec![strip_emphasis(t)]);
        }
    }

    rows
}

/// Escape a string for a PDF literal string. `\`, `(` and `)` must be escaped; the base-14
/// Helvetica font with WinAnsi encoding cannot represent arbitrary Unicode, so those become `?`.
fn escape_pdf_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            c if (c as u32) < 127 => out.push(c),
            _ => out.push('?'),
        }
    }
    if out.is_empty() {
        out.push(' ');
    }
    out
}

/// Wrap a line to `max` characters so it fits within the printable width.
fn wrap_line(s: &str, max: usize) -> Vec<String> {
    if s.chars().count() <= max {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in s.split(' ') {
        if current.chars().count() + word.chars().count() + 1 > max && !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Build a minimal but valid PDF 1.4 document (Helvetica, multi-page) from text.
fn build_pdf(title: &str, content: &str) -> Vec<u8> {
    const LINES_PER_PAGE: usize = 46;
    const MAX_CHARS: usize = 92;

    // Lay out the text: title, blank line, then the wrapped body.
    let mut lines: Vec<String> = vec![title.to_string(), String::new()];
    for l in content.lines() {
        let cleaned = strip_emphasis(l.trim_end());
        if cleaned.trim().is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(wrap_line(&cleaned, MAX_CHARS));
        }
    }

    let chunks: Vec<Vec<String>> = if lines.is_empty() {
        vec![Vec::new()]
    } else {
        lines.chunks(LINES_PER_PAGE).map(|c| c.to_vec()).collect()
    };
    let page_count = chunks.len();

    // Object layout: 1 Catalog, 2 Pages, 3 Font, then per page: page object, content object.
    let first_page_obj = 4usize;
    let mut kids = String::new();
    for i in 0..page_count {
        kids.push_str(&format!("{} 0 R ", first_page_obj + i * 2));
    }

    let mut objs: Vec<Vec<u8>> = Vec::new();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objs.push(
        format!("<< /Type /Pages /Count {} /Kids [ {}] >>", page_count, kids).into_bytes(),
    );
    objs.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );

    const MEDIA_W: f32 = 595.0; // A4 at 72 dpi
    const MEDIA_H: f32 = 842.0;
    const MARGIN: f32 = 56.0;
    const LEADING: f32 = 14.0;

    for (i, chunk) in chunks.iter().enumerate() {
        let content_num = first_page_obj + i * 2 + 1;
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
                MEDIA_W, MEDIA_H, content_num
            )
            .into_bytes(),
        );

        let mut stream = String::from("BT\n/F1 11 Tf\n");
        let mut y = MEDIA_H - MARGIN;
        for l in chunk {
            stream.push_str(&format!(
                "1 0 0 1 {:.2} {:.2} Tm ({}) Tj\n",
                MARGIN,
                y,
                escape_pdf_text(l)
            ));
            y -= LEADING;
        }
        stream.push_str("ET\n");

        let s = stream.into_bytes();
        let mut obj = format!("<< /Length {} >>\nstream\n", s.len()).into_bytes();
        obj.extend_from_slice(&s);
        obj.extend_from_slice(b"endstream");
        objs.push(obj);
    }

    // Assemble the file, recording each object's byte offset as we go.
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(objs.len());
    for (i, body) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_at = out.len();
    let size = objs.len() + 1;
    out.extend_from_slice(format!("xref\n0 {}\n", size).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            size, xref_at
        )
        .as_bytes(),
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product(format: OutputFormat, content: &str) -> GeneratedProduct {
        GeneratedProduct {
            id: 1,
            name: "Test Product".to_string(),
            template_id: "t1".to_string(),
            content: content.to_string(),
            format,
            created_at: chrono::Utc::now(),
            metadata: crate::product_generator::ProductMetadata {
                model_used: "test-model".to_string(),
                tokens_used: 0,
                generation_time_ms: 0,
                parameters: serde_json::json!({}),
            },
        }
    }

    fn tmp(name: &str) -> String {
        let dir = std::env::temp_dir().join("dpf-export-tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name).to_string_lossy().to_string()
    }

    /// The three formats that used to write the wrong file type must now write the real thing.
    /// Checked by magic bytes, not by the extension.
    #[test]
    fn exports_are_real_formats_not_renamed_text() {
        let e = Exporter::new();
        let content = "# Heading\n\nSome body text.\n\n- one\n- two\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";

        let pdf = e.export_pdf(&product(OutputFormat::Pdf, content), &tmp("t.pdf")).unwrap();
        let pdf_bytes = std::fs::read(&pdf).unwrap();
        assert!(pdf_bytes.starts_with(b"%PDF-1."), "not a PDF: {:?}", &pdf_bytes[..8.min(pdf_bytes.len())]);
        assert!(
            pdf_bytes.windows(5).any(|w| w == b"%%EOF"),
            "PDF has no EOF marker"
        );
        assert!(
            pdf_bytes.windows(4).any(|w| w == b"xref"),
            "PDF missing xref table"
        );
        // A real PDF has binary structure, not just the source text.
        assert!(pdf_bytes.len() > 400, "PDF suspiciously small: {} bytes", pdf_bytes.len());

        let docx = e.export_docx(&product(OutputFormat::Docx, content), &tmp("t.docx")).unwrap();
        let docx_bytes = std::fs::read(&docx).unwrap();
        assert!(docx.ends_with(".docx"), "wrong extension: {docx}");
        assert_eq!(&docx_bytes[..2], b"PK", "DOCX is not a ZIP container");
        // Real OOXML must contain the Word document part.
        let s = String::from_utf8_lossy(&docx_bytes);
        assert!(s.contains("word/document.xml"), "DOCX has no word/document.xml");

        let xlsx = e.export_xlsx(&product(OutputFormat::Xlsx, content), &tmp("t.xlsx")).unwrap();
        let xlsx_bytes = std::fs::read(&xlsx).unwrap();
        assert!(xlsx.ends_with(".xlsx"), "wrong extension: {xlsx}");
        assert_eq!(&xlsx_bytes[..2], b"PK", "XLSX is not a ZIP container");
        let s = String::from_utf8_lossy(&xlsx_bytes);
        assert!(s.contains("xl/workbook.xml"), "XLSX has no xl/workbook.xml");
    }

    #[test]
    fn markdown_table_rows_become_spreadsheet_rows() {
        let rows = rows_from_content("| A | B |\n|---|---|\n| 1 | 2 |\nplain line\n");
        assert_eq!(rows.len(), 3, "alignment row should be skipped: {rows:?}");
        assert_eq!(rows[0], vec!["A".to_string(), "B".to_string()]);
        assert_eq!(rows[1], vec!["1".to_string(), "2".to_string()]);
        assert_eq!(rows[2], vec!["plain line".to_string()]);
    }

    #[test]
    fn pdf_text_is_escaped_so_it_cannot_break_the_file() {
        // Unbalanced parentheses and backslashes must not corrupt the content stream.
        let s = escape_pdf_text("a (b) \\ c");
        assert_eq!(s, "a \\(b\\) \\\\ c");
        assert_eq!(escape_pdf_text(""), " ");
    }

    #[test]
    fn headings_and_bullets_are_classified() {
        let blocks = parse_blocks("Title", "## Sub\n- point\nplain");
        assert!(matches!(blocks[0], Block::Heading(1, _)));
        assert!(matches!(blocks[1], Block::Heading(2, _)));
        assert!(matches!(blocks[2], Block::Bullet(_)));
        assert!(matches!(blocks[3], Block::Body(_)));
    }
}
