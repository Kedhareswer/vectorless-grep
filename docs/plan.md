# Vectorless — Product Plan

Current state, known issues, and the prioritised backlog.

---

## Current state (as of 2026-02)

### Done ✅

| Area | Status |
|------|--------|
| Tauri 2 desktop shell | ✅ Working |
| Multi-format parsing (PDF, DOCX, XLSX, PPTX, text/md) | ✅ Pure Rust, no Python |
| Document → Section → Paragraph hierarchy | ✅ Heading-detection heuristics |
| Multi-file upload (parallel ingest with per-file status chips) | ✅ |
| SQLite persistence (documents + nodes + reasoning runs) | ✅ |
| Gemini 2.0 Flash reasoning agent (ReAct loop) | ✅ |
| Real-time step streaming (Tauri events) | ✅ |
| Reasoning timeline UI (TracePane) | ✅ |
| Graph view (React Flow AST graph) | ✅ |
| Document viewer with node context | ✅ |
| Embedding proximity visualisation (SVG scatter) | ✅ |
| Design token system (tokens.css → base.css, no hardcoded values) | ✅ |
| Proper tree UI (connector lines, chevrons, depth guides) | ✅ |
| CLAUDE.md + docs/agents.md + docs/plan.md | ✅ |

---

## Known issues / bugs

| # | Issue | Priority |
|---|-------|----------|
| 1 | PDF heading detection relies on blank-line separation; continuous-flow PDFs produce one giant section | Medium |
| 2 | Querying across multiple documents is not supported (UI only shows one active document) | High |
| 3 | No streaming token display during agent steps (shows only after step completes) | Low |
| 4 | `export_markdown` writes to internal data dir; no "save as" dialog | Low |
| 5 | Document delete doesn't confirm and has no undo | Medium |
| 6 | PPTX heading extraction takes first line literally; may be a markdown `#` fragment | Low |

---

## Roadmap

### P0 — Core reliability

- [ ] **Cross-document reasoning** — allow the agent to reference nodes from multiple ingested documents in a single query. Requires `documentId` filter to be optional in `run_reasoning_query`.
- [ ] **PDF page number extraction** — `pdf-extract` loses page boundaries. Integrate `lopdf` for per-page text + page number annotation.
- [ ] **Duplicate re-ingest** — currently returns cached result silently. Surface this to the user as "already indexed" with an option to force re-parse.

### P1 — UX polish

- [ ] **Delete document confirmation dialog** — before `delete_document` runs, show a native Tauri dialog.
- [ ] **Export with save-as dialog** — use `tauri-plugin-dialog::save` so the user can choose export location.
- [ ] **Tree search highlight** — when searching, highlight the matched substring in `tree-node-title`.
- [ ] **Keyboard navigation in tree** — Arrow keys to move between nodes, Enter to select.
- [ ] **Query history dropdown** — show `recentQueries` in a dropdown under the query bar.

### P2 — Parsing improvements

- [ ] **DOCX table extraction** — `docx-rs` currently skips table cells. Parse `DocumentChild::Table` and emit `Table` nodes.
- [ ] **Image OCR** — for PNG/JPG/WEBP uploads, call a local OCR (e.g. `tesseract-rs`) instead of returning an error.
- [ ] **PDF bounding box** — extract text block coordinates from `lopdf` and store in `bbox_json` so the PDF viewer can highlight the selected node.

### P3 — Agent improvements

- [ ] **Embedding-based retrieval** — generate local embeddings (e.g. via `fastembed-rs`) and rank nodes by cosine similarity before the agent sees them, reducing hallucination.
- [ ] **Step caching** — if the same `(documentId, query)` pair was run before, show cached run with an option to re-run.
- [ ] **Multi-provider support** — add OpenAI and Anthropic provider clients alongside Gemini. Let user pick in Settings.
- [ ] **Confidence calibration** — current confidence is LLM self-reported. Add post-hoc grounding check (does answer text appear in citation nodes?).

### P4 — Infrastructure

- [ ] **Auto-update** — integrate `tauri-plugin-updater` for OTA releases.
- [ ] **Telemetry (opt-in)** — track parse errors and agent failure rates for debugging.
- [ ] **Playwright E2E tests** — the `tests/` directory is empty. Add smoke tests for upload → query flow.
- [ ] **CI pipeline** — add GitHub Actions: `cargo check`, `cargo clippy`, `eslint`, `tsc --noEmit` on every PR.

---

## Architecture decisions (ADRs)

### ADR-001: No Python sidecar
**Decision**: Removed `docling_client.rs` and the Python sidecar. All parsing is pure Rust.
**Rationale**: Cold-start latency (3–5 s), Python version management, and process lifecycle complexity outweigh the marginal quality improvement of docling for typical documents.
**Trade-off**: Image PDFs (scanned) cannot be parsed without OCR. Tracked as P2 backlog item.

### ADR-002: Design tokens only
**Decision**: Zero hardcoded colors/sizes in component files. All values in `tokens.css`.
**Rationale**: Consistent theming, easy dark/light mode switch, linter-enforceable via `no-inline-styles`.
**Pattern for dynamic values**: Pass as CSS custom property `--_name` via `style={}`, consume in base.css.

### ADR-003: Flat SQLite, no vector DB
**Decision**: Store all nodes in a single SQLite table. No embedding index.
**Rationale**: Keeps the binary self-contained (no external process, no Postgres). The agent uses LLM reasoning over the full tree rather than ANN retrieval.
**Trade-off**: Querying 10k+ nodes per document will be slow. P3 item to add `fastembed-rs` for retrieval ranking.

### ADR-004: Multi-file parallel ingest
**Decision**: `pickDocumentFiles()` (multi=true) + `Promise.allSettled()` ingest in parallel.
**Rationale**: Users often upload sets of related documents. Sequential ingest is unnecessarily slow.
**Trade-off**: Concurrent SQLite writes use a single connection pool; sqlx serialises them automatically.
