//! Pure-Rust document parser.
//!
//! Produces a hierarchical [`NormalizedPayload`]:
//!   Document → Section* → Paragraph*
//!
//! Heading detection uses simple heuristics (short lines, all-caps, markdown
//! `#` prefixes, DOCX style names) so PDFs and DOCX files yield a proper
//! two-level tree instead of a flat list of chunks.

use std::path::Path;

use image::GenericImageView;
use serde_json::Value;
use uuid::Uuid;

use crate::core::errors::{AppError, AppResult};
use crate::sidecar::types::{NormalizedPayload, SidecarDocument, SidecarEdge, SidecarNode};

const CHUNK_SIZE: usize = 600;
const HEADING_MAX_LEN: usize = 120;

// ─────────────────────────────────────────────────────────────────────────────

pub fn parse(file_path: &Path, mime_type: &str) -> AppResult<NormalizedPayload> {
    let mime = mime_type.trim().to_ascii_lowercase();
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if mime.contains("pdf") || ext == "pdf" {
        parse_pdf(file_path)
    } else if mime.contains("wordprocessingml") || ext == "docx" {
        parse_docx(file_path)
    } else if mime.contains("spreadsheetml") || ext == "xlsx" || ext == "xls" || ext == "xlsm" {
        parse_xlsx(file_path)
    } else if mime.contains("presentationml") || ext == "pptx" {
        parse_pptx(file_path)
    } else if mime.contains("image") || matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif") {
        parse_image(file_path)
    } else {
        parse_text(file_path)
    }
}

// ── PDF ───────────────────────────────────────────────────────────────────────

fn parse_pdf(file_path: &Path) -> AppResult<NormalizedPayload> {
    let bytes = std::fs::read(file_path)
        .map_err(|e| AppError::Io(format!("cannot read PDF: {e}")))?;
    
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| {
            eprintln!("PDF extraction error for {:?}: {}", file_path, e);
            AppError::Sidecar(format!("pdf-extract failed: {e}"))
        })?;
    
    if text.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "PDF contains no extractable text (may be image-based or encrypted)".to_string()
        ));
    }
    
    let title = stem(file_path);
    build_hierarchy(title, 1, text_to_sections(&text))
}

// ── DOCX ──────────────────────────────────────────────────────────────────────

fn parse_docx(file_path: &Path) -> AppResult<NormalizedPayload> {
    let bytes = std::fs::read(file_path)
        .map_err(|e| AppError::Io(format!("cannot read DOCX: {e}")))?;
    let items = match parse_docx_with_docx_rs(&bytes) {
        Ok(items) => items,
        Err(primary_err) => match parse_docx_with_xml_fallback(&bytes) {
            Ok(items) => items,
            Err(fallback_err) => {
                return Err(AppError::Sidecar(format!(
                    "DOCX parse failed (docx-rs: {primary_err}; xml fallback: {fallback_err})"
                )));
            }
        },
    };

    let title = stem(file_path);
    build_hierarchy(title, 1, group_by_headings(items))
}

fn parse_docx_with_docx_rs(bytes: &[u8]) -> AppResult<Vec<(bool, String)>> {
    let docx = docx_rs::read_docx(bytes)
        .map_err(|e| AppError::Sidecar(format!("docx-rs failed: {e}")))?;

    let mut items: Vec<(bool, String)> = Vec::new();
    for child in &docx.document.children {
        if let docx_rs::DocumentChild::Paragraph(para) = child {
            let style_id = para
                .property
                .style
                .as_ref()
                .map(|s| s.val.to_ascii_lowercase())
                .unwrap_or_default();
            let is_heading_style =
                style_id.starts_with("heading") || style_id.starts_with("title");

            let mut buf = String::new();
            for run_child in &para.children {
                if let docx_rs::ParagraphChild::Run(run) = run_child {
                    for r in &run.children {
                        if let docx_rs::RunChild::Text(t) = r {
                            buf.push_str(&t.text);
                        }
                    }
                }
            }
            let trimmed = buf.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            let is_heading = is_heading_style || looks_like_heading(&trimmed);
            items.push((is_heading, trimmed));
        }
    }

    if items.is_empty() {
        return Err(AppError::InvalidInput(
            "DOCX contains no extractable paragraph text (docx-rs path)".to_string(),
        ));
    }

    Ok(items)
}

fn parse_docx_with_xml_fallback(bytes: &[u8]) -> AppResult<Vec<(bool, String)>> {
    use std::io::Read;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| AppError::Sidecar(format!("zip open failed: {e}")))?;
    let mut doc_xml = archive
        .by_name("word/document.xml")
        .map_err(|e| AppError::Sidecar(format!("word/document.xml missing: {e}")))?;
    let mut xml = String::new();
    doc_xml
        .read_to_string(&mut xml)
        .map_err(|e| AppError::Sidecar(format!("cannot read document.xml: {e}")))?;

    let xml_doc = roxmltree::Document::parse(&xml)
        .map_err(|e| AppError::Sidecar(format!("document.xml parse failed: {e}")))?;

    let mut items: Vec<(bool, String)> = Vec::new();
    for para in xml_doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "p")
    {
        let style_id = para
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "pStyle")
            .find_map(|style_node| {
                style_node.attributes().find_map(|attr| {
                    let key = attr.name();
                    if key.eq_ignore_ascii_case("val") || key.ends_with(":val") {
                        Some(attr.value().to_ascii_lowercase())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        let is_heading_style =
            style_id.starts_with("heading") || style_id.starts_with("title");

        let mut buf = String::new();
        for node in para.descendants().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "t" => {
                    if let Some(text) = node.text() {
                        buf.push_str(text);
                    }
                }
                "tab" => buf.push('\t'),
                "br" | "cr" => buf.push('\n'),
                _ => {}
            }
        }
        let trimmed = buf.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        let is_heading = is_heading_style || looks_like_heading(&trimmed);
        items.push((is_heading, trimmed));
    }

    if items.is_empty() {
        return Err(AppError::InvalidInput(
            "DOCX contains no extractable paragraph text (xml fallback path)".to_string(),
        ));
    }

    Ok(items)
}

// ── XLSX ──────────────────────────────────────────────────────────────────────

fn parse_xlsx(file_path: &Path) -> AppResult<NormalizedPayload> {
    use calamine::{open_workbook_auto, Reader};

    let mut workbook = open_workbook_auto(file_path)
        .map_err(|e| AppError::Sidecar(format!("calamine failed: {e}")))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let mut sections: Vec<Section> = Vec::new();

    for sheet_name in &sheet_names {
        if let Some(Ok(range)) = workbook.worksheet_range(sheet_name) {
            let mut rows: Vec<String> = Vec::new();
            for row in range.rows() {
                let cells: Vec<String> = row.iter().map(ToString::to_string).collect();
                let line = cells.join("\t");
                if !line.trim().is_empty() {
                    rows.push(line);
                }
            }
            if !rows.is_empty() {
                let paragraphs = text_to_chunks(&rows.join("\n"));
                sections.push(Section {
                    heading: format!("Sheet: {sheet_name}"),
                    paragraphs,
                });
            }
        }
    }

    if sections.is_empty() {
        return Err(AppError::InvalidInput(
            "native parser: XLSX contains no data".to_string(),
        ));
    }

    build_hierarchy(stem(file_path), 1, sections)
}

// ── PPTX ──────────────────────────────────────────────────────────────────────

fn parse_pptx(file_path: &Path) -> AppResult<NormalizedPayload> {
    use pptx_to_md::{ParserConfig, PptxContainer};

    let config = ParserConfig::builder().build();
    let mut container = PptxContainer::open(file_path, config)
        .map_err(|e| AppError::Sidecar(format!("pptx-to-md open failed: {e}")))?;
    let slides = container
        .parse_all()
        .map_err(|e| AppError::Sidecar(format!("pptx-to-md parse failed: {e}")))?;

    let mut sections: Vec<Section> = Vec::new();
    for (i, slide) in slides.iter().enumerate() {
        let md = slide.convert_to_md().unwrap_or_default();
        let text = md.trim().to_string();
        if text.is_empty() {
            continue;
        }
        let mut lines = text.lines();
        let heading = lines
            .next()
            .map(clean_pptx_heading)
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| format!("Slide {}", i + 1));
        let body: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();
        let paragraphs = if body.is_empty() {
            vec![text]
        } else {
            text_to_chunks(&body)
        };
        sections.push(Section { heading, paragraphs });
    }

    if sections.is_empty() {
        return Err(AppError::InvalidInput(
            "native parser: PPTX contains no extractable text".to_string(),
        ));
    }

    build_hierarchy(stem(file_path), slides.len().max(1) as i64, sections)
}

// ── Plain text / Markdown / fallback ─────────────────────────────────────────

fn parse_text(file_path: &Path) -> AppResult<NormalizedPayload> {
    let text = std::fs::read_to_string(file_path)
        .map_err(|e| AppError::Io(format!("cannot read file as text: {e}")))?;
    build_hierarchy(stem(file_path), 1, text_to_sections(&text))
}

// ── Image ─────────────────────────────────────────────────────────────────────

fn parse_image(file_path: &Path) -> AppResult<NormalizedPayload> {
    let img = image::open(file_path)
        .map_err(|e| AppError::Sidecar(format!("image open failed: {e}")))?;
    
    let (width, height) = img.dimensions();
    let format = image::guess_format(&std::fs::read(file_path)
        .map_err(|e| AppError::Io(format!("cannot read image: {e}")))?)
        .map(|f| format!("{:?}", f))
        .unwrap_or_else(|_| "Unknown".to_string());
    
    let title = stem(file_path);
    let metadata_text = format!(
        "Image: {}\nFormat: {}\nDimensions: {}x{} pixels",
        title, format, width, height
    );
    
    let sections = vec![Section {
        heading: "Image Metadata".to_string(),
        paragraphs: vec![metadata_text],
    }];
    
    build_hierarchy(title, 1, sections)
}

// ── Section detection ─────────────────────────────────────────────────────────

struct Section {
    heading: String,
    paragraphs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Paragraph,
    Table,
    Figure,
}

/// Split raw text into sections using heading heuristics.
fn text_to_sections(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::from("Overview");
    let mut current_body: Vec<String> = Vec::new();

    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if looks_like_heading(para) {
            if !current_body.is_empty() {
                sections.push(Section {
                    heading: current_heading.clone(),
                    paragraphs: current_body.drain(..).collect(),
                });
            }
            current_heading = clean_heading(para);
        } else {
            for chunk in text_to_chunks(para) {
                current_body.push(chunk);
            }
        }
    }

    if !current_body.is_empty() {
        sections.push(Section {
            heading: current_heading,
            paragraphs: current_body,
        });
    }

    // Fallback: no headings detected — number the chunks
    if sections.is_empty() {
        for (i, chunk) in text_to_chunks(text).into_iter().enumerate() {
            sections.push(Section {
                heading: format!("Part {}", i + 1),
                paragraphs: vec![chunk],
            });
        }
    }

    sections
}

/// Group (is_heading, text) DOCX items into sections.
fn group_by_headings(items: Vec<(bool, String)>) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::from("Overview");
    let mut current_body: Vec<String> = Vec::new();

    for (is_heading, text) in items {
        if is_heading {
            if !current_body.is_empty() {
                sections.push(Section {
                    heading: current_heading.clone(),
                    paragraphs: current_body.drain(..).collect(),
                });
            }
            current_heading = text;
        } else {
            for chunk in text_to_chunks(&text) {
                current_body.push(chunk);
            }
        }
    }

    if !current_body.is_empty() {
        sections.push(Section {
            heading: current_heading,
            paragraphs: current_body,
        });
    }

    if sections.is_empty() {
        sections.push(Section {
            heading: "Document".to_string(),
            paragraphs: vec!["(No extractable body text)".to_string()],
        });
    }

    sections
}

// ── Tree builder ──────────────────────────────────────────────────────────────

/// Build Document → Section* → Paragraph* hierarchy.
fn build_hierarchy(
    title: String,
    pages: i64,
    sections: Vec<Section>,
) -> AppResult<NormalizedPayload> {
    if sections.is_empty() {
        return Err(AppError::InvalidInput(
            "native parser: document contains no extractable text".to_string(),
        ));
    }

    let root_id = format!("root-{}", Uuid::new_v4());
    let root = SidecarNode {
        id: root_id.clone(),
        parent_id: None,
        node_type: "Document".to_string(),
        title: title.clone(),
        text: String::new(),
        page_start: Some(1),
        page_end: Some(pages.max(1)),
        ordinal_path: "root".to_string(),
        bbox: serde_json::json!({}),
        metadata: serde_json::json!({ "parser": "native" }),
    };

    let mut nodes = vec![root];
    let mut edges: Vec<SidecarEdge> = Vec::new();

    for (sec_idx, section) in sections.into_iter().enumerate() {
        let sec_ordinal = format!("{}", sec_idx + 1);
        let sec_id = format!("s-{}", Uuid::new_v4());

        nodes.push(SidecarNode {
            id: sec_id.clone(),
            parent_id: Some(root_id.clone()),
            node_type: "Section".to_string(),
            title: section.heading,
            text: String::new(),
            page_start: None,
            page_end: None,
            ordinal_path: sec_ordinal.clone(),
            bbox: Value::Null,
            metadata: serde_json::json!({ "parser": "native" }),
        });
        edges.push(SidecarEdge {
            from: root_id.clone(),
            to: sec_id.clone(),
            relation: "contains".to_string(),
        });

        for (para_idx, para_text) in section.paragraphs.into_iter().enumerate() {
            let kind = classify_block(&para_text);
            let node_type = match kind {
                BlockKind::Paragraph => "Paragraph",
                BlockKind::Table => "Table",
                BlockKind::Figure => "Figure",
            };
            let title = match kind {
                BlockKind::Paragraph => format!("\u{00b6} {}", para_idx + 1),
                BlockKind::Table => format!("Table {}", para_idx + 1),
                BlockKind::Figure => format!("Figure {}", para_idx + 1),
            };
            let para_id = format!("p-{}", Uuid::new_v4());
            nodes.push(SidecarNode {
                id: para_id.clone(),
                parent_id: Some(sec_id.clone()),
                node_type: node_type.to_string(),
                title,
                text: para_text,
                page_start: None,
                page_end: None,
                ordinal_path: format!("{}.{}", sec_idx + 1, para_idx + 1),
                bbox: Value::Null,
                metadata: serde_json::json!({
                    "parser": "native",
                    "kind": match kind {
                        BlockKind::Paragraph => "paragraph",
                        BlockKind::Table => "markdown_table",
                        BlockKind::Figure => "markdown_image",
                    }
                }),
            });
            edges.push(SidecarEdge {
                from: sec_id.clone(),
                to: para_id,
                relation: "contains".to_string(),
            });
        }
    }

    Ok(NormalizedPayload {
        document: SidecarDocument {
            title,
            pages: pages.max(1),
            metadata: serde_json::json!({ "parser": "native" }),
        },
        nodes,
        edges,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true when a paragraph looks like a section heading.
fn looks_like_heading(para: &str) -> bool {
    let line = para.lines().next().unwrap_or("").trim();
    if line.is_empty() || line.len() > HEADING_MAX_LEN {
        return false;
    }
    // Markdown-style
    if line.starts_with('#') {
        return true;
    }
    // No sentence-ending punctuation
    if line.ends_with('.') || line.ends_with('?') || line.ends_with('!') {
        return false;
    }
    // Must be a single-line paragraph
    if para.contains("\n\n") {
        return false;
    }
    let word_count = line.split_whitespace().count();
    if word_count == 0 || word_count > 12 {
        return false;
    }
    let starts_upper = line.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
    let alpha: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
    let is_mostly_upper = if alpha.is_empty() {
        false
    } else {
        let upper = alpha.iter().filter(|c| c.is_uppercase()).count();
        upper as f64 / alpha.len() as f64 > 0.65
    };
    starts_upper || is_mostly_upper
}

/// Strip markdown `#` prefixes and trim.
fn clean_heading(heading: &str) -> String {
    heading.trim_start_matches('#').trim().to_string()
}

fn clean_pptx_heading(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
        let inner = trimmed
            .trim_start_matches("<!--")
            .trim_end_matches("-->")
            .trim();
        return clean_heading(inner);
    }
    clean_heading(trimmed)
}

fn classify_block(text: &str) -> BlockKind {
    let value = text.trim();
    if value.is_empty() {
        return BlockKind::Paragraph;
    }
    if looks_like_figure_block(value) {
        return BlockKind::Figure;
    }
    if looks_like_markdown_table(value) || looks_like_tsv_table(value) {
        return BlockKind::Table;
    }
    BlockKind::Paragraph
}

fn looks_like_figure_block(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains("<img") || lower.contains("data:image/") {
        return true;
    }
    if let Some(start) = text.find("![") {
        if let Some(open) = text[start..].find("](") {
            if let Some(close) = text[start + open + 2..].find(')') {
                let url = &text[start + open + 2..start + open + 2 + close];
                let url_lower = url.to_ascii_lowercase();
                return url_lower.starts_with("data:image/")
                    || url_lower.ends_with(".png")
                    || url_lower.ends_with(".jpg")
                    || url_lower.ends_with(".jpeg")
                    || url_lower.ends_with(".webp")
                    || url_lower.ends_with(".gif")
                    || url_lower.ends_with(".svg");
            }
        }
    }
    false
}

fn looks_like_markdown_table(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.len() < 2 || !lines[0].contains('|') {
        return false;
    }
    let separator = lines[1].replace('|', "").replace(':', "").replace('-', "");
    lines[1].contains('-') && separator.trim().is_empty()
}

fn looks_like_tsv_table(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.len() < 2 {
        return false;
    }
    let tabbed = lines.iter().filter(|line| line.contains('\t')).count();
    tabbed >= 2 && (tabbed as f64 / lines.len() as f64) >= 0.8
}

/// Split text on blank lines into chunks up to CHUNK_SIZE.
fn text_to_chunks(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if current.len() + para.len() + 2 > CHUNK_SIZE && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }
    chunks
}

/// File stem as title.
fn stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document")
        .to_string()
}
