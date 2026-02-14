use std::io::{Cursor, Write};

use tempfile::NamedTempFile;
use vectorless_lib::sidecar::native_parser;
use zip::write::FileOptions;

fn build_fallback_docx_bytes() -> Vec<u8> {
    let cursor = Cursor::new(Vec::<u8>::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let options: FileOptions<'_, ()> = FileOptions::default();

    // Intentionally minimal DOCX-like package: contains document.xml but omits
    // relationship/content-type parts that docx-rs expects. This exercises
    // native_parser's XML fallback path.
    zip.start_file("word/document.xml", options)
        .expect("start file");
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>Intro</w:t></w:r>
    </w:p>
    <w:p>
      <w:r><w:t>Fallback parser extracted this paragraph.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#,
    )
    .expect("write xml");

    zip.finish().expect("finish zip").into_inner()
}

#[test]
fn parse_docx_uses_xml_fallback_when_docx_rs_fails() {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(&build_fallback_docx_bytes()).expect("write docx bytes");
    let path = file
        .path()
        .with_extension("docx");
    std::fs::copy(file.path(), &path).expect("copy with .docx extension");

    let payload = native_parser::parse(
        path.as_path(),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    )
    .expect("docx fallback parse should succeed");

    assert!(
        !payload.nodes.is_empty(),
        "expected non-empty node payload from fallback parser"
    );
    assert!(
        payload.nodes.iter().any(|node| node.parent_id.is_none()),
        "expected root node in parsed payload"
    );
    assert!(
        payload
            .nodes
            .iter()
            .any(|node| node.node_type == "Paragraph" && node.text.contains("Fallback parser")),
        "expected at least one paragraph node with extracted fallback text"
    );
}

#[test]
fn parse_user_failing_docx_fixture_when_available() {
    let fixture = std::path::Path::new("tests/fixtures/docx/user-failing.docx");
    if !fixture.exists() {
        // User-provided fixture is optional in shared CI/dev contexts.
        return;
    }

    let payload = native_parser::parse(
        fixture,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    )
    .expect("user failing fixture should parse");

    assert!(
        payload.nodes.len() > 1,
        "expected docx fixture to produce document + content nodes"
    );
    assert!(
        payload
            .nodes
            .iter()
            .any(|node| node.node_type == "Paragraph" && !node.text.trim().is_empty()),
        "expected at least one non-empty paragraph node from fixture"
    );
}
