# Vectorless

Vectorless is a Windows-first desktop application for structure-grounded RAG.
It uses Doc-AST traversal + reasoning traces instead of vector embeddings.

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop runtime | Tauri 2 (Rust backend + WebView frontend) |
| UI | React 19 + TypeScript + Vite |
| Storage | SQLite (`src-tauri/src/db/migrations/`) |
| Document parsing | Pure Rust: `pdf-extract`, `docx-rs`, `calamine`, `pptx-to-md`, `image` |
| LLM provider | Gemini API (cloud-only) |

## Key Features

- Native Rust document parser (PDF, DOCX, PPTX, XLSX, CSV, Images, plain text, Markdown)
- Image metadata extraction (dimensions, format) using `image` crate
- Doc-AST persistence with hierarchical nodes (Document → Section → Paragraph)
- Project-based document organization
- Reasoning planner loop (`ScanRoot -> SelectSections -> DrillDown -> ExtractEvidence -> Synthesize -> SelfCheck`)
- Explainable trace events: `ingest/progress`, `reasoning/step`, `reasoning/complete`, `reasoning/error`
- 3-pane UI: Tree | Trace | Document + Answer
- Comprehensive test suite: 23 parser E2E tests (all passing)

## Run Locally

```bash
# one-time setup (auto-installs Node/Rust/MSVC if missing)
npm run setup:win

# run app
npm run tauri:dev
```

Before running queries, store your Gemini key in the app header (`Save Key`).

`setup:win` installs/verifies:
- Node.js LTS + npm (winget -> choco -> direct installer fallback)
- Rustup/cargo (winget -> choco -> direct installer fallback)
- Visual Studio Build Tools (C++ toolchain)
- Node dependencies
- Rust `rustfmt` component

`bootstrap-windows.ps1` is a compatibility wrapper for `setup.ps1`.

## Tests

```bash
# Frontend tests
npm run test

# Backend tests (all Rust tests)
cargo test --manifest-path src-tauri/Cargo.toml

# Parser E2E tests only (23 tests covering all formats)
cargo test --manifest-path src-tauri/Cargo.toml --test parser_e2e_tests

# E2E tests
npm run test:e2e
```
