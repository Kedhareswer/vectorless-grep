# Test Fixtures

This directory contains sample files for testing the document parser.

## Available Fixtures

- `sample.md`: Markdown file with multiple heading levels
- `sample.txt`: Plain text file with heading heuristics
- `sample.csv`: CSV file (in xlsx directory)
- `sample.xlsx`: Excel spreadsheet (in xlsx directory)
- `sample.pdf`: PDF document (in pdf directory)
- `sample.docx`: Word document (in docx directory)
- `sample.pptx`: PowerPoint presentation (in pptx directory)
- `sample.jpg`, `sample.png`: Image files (in images directory)

## Adding New Fixtures

To add new test fixtures:

1. Place the file in the appropriate subdirectory
2. Add corresponding tests in `parser_e2e_tests.rs`
3. Ensure the test checks for expected structure and content

## Notes

- Some fixtures may be missing and tests will skip gracefully
- Tests validate both structure (nodes, edges) and content accuracy
- Image tests verify metadata extraction (dimensions, format)
