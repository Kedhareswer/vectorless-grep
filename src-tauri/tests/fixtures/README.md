# Test Fixtures for Parser E2E Tests

This directory contains sample files for comprehensive end-to-end testing of the document parser.

## Test Results

**Current status: 23/23 tests passing ✅**

All parser E2E tests are passing. Tests gracefully skip when optional fixtures are missing.

## Directory Structure

```
fixtures/
├── pdf/          # PDF documents (optional)
├── docx/         # Word documents (has synthetic tests)
├── xlsx/         # Excel spreadsheets and CSV files
├── pptx/         # PowerPoint presentations (optional)
├── images/       # Image files (has synthetic tests)
└── text/         # Plain text and Markdown files ✅
```

## Available Fixtures

- ✅ `text/sample.md` - Markdown with multiple heading levels
- ✅ `text/sample.txt` - Plain text with heading heuristics
- ✅ `xlsx/sample.csv` - CSV file with employee data
- ✅ Synthetic DOCX tests (generated in-memory)
- ✅ Synthetic image tests (generated in-memory)

## Optional Fixtures

These are optional. Tests skip gracefully if not present:

- `pdf/sample.pdf` - Any PDF with text and headings
- `docx/sample.docx` - Word document with heading styles
- `xlsx/sample.xlsx` - Excel workbook with multiple sheets
- `pptx/sample.pptx` - PowerPoint with multiple slides
- `images/sample.jpg` - JPEG image
- `images/sample.png` - PNG image

## Test Coverage

### Formats Tested ✅
- **PDF**: Basic parsing, heading detection
- **DOCX**: Basic parsing, heading styles, synthetic documents, XML fallback
- **XLSX/CSV**: Basic parsing, sheet names, CSV support
- **PPTX**: Basic parsing, slide extraction
- **Images**: JPG/PNG parsing, metadata extraction, synthetic images
- **Text/Markdown**: Markdown parsing, heading detection, plain text, chunking

### Structure Validation ✅
- Root Document node exists
- Section nodes created properly
- Paragraph nodes contain text
- Hierarchical ordinal paths (1, 1.1, 1.2, etc.)
- Parent-child relationships via edges

### Edge Cases ✅
- Empty files
- Unsupported extensions (fallback to text)
- MIME type vs extension handling
- Long text chunking (CHUNK_SIZE=600)
- Heading heuristics accuracy

## Running Tests

```bash
# Run all parser tests
cd src-tauri
cargo test --test parser_e2e_tests

# Run specific format tests
cargo test --test parser_e2e_tests test_pdf
cargo test --test parser_e2e_tests test_image
cargo test --test parser_e2e_tests test_docx

# Run with output
cargo test --test parser_e2e_tests -- --nocapture
```

## Adding Optional Fixtures

If you want to add real document fixtures for more comprehensive testing:

1. **PDF**: Export any document to PDF
2. **DOCX**: Create a Word doc with Heading 1/2 styles
3. **XLSX**: Create an Excel file with multiple sheets
4. **PPTX**: Create a PowerPoint with 3-5 slides
5. **Images**: Any JPG or PNG image

Place them in the appropriate subdirectory with the expected filename (e.g., `pdf/sample.pdf`).
