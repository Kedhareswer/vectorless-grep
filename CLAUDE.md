# CLAUDE.md — Vectorless

Developer guide for Claude Code working on this codebase.

## Project in one sentence

**Vectorless** is a Tauri 2 desktop app that ingests documents (PDF, DOCX, PPTX, XLSX, text), parses them into a hierarchical AST, and runs a Gemini-powered multi-step reasoning agent over that AST to answer free-text queries with cited responses.

---

## Tech stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 (Rust backend + WebView frontend) |
| Frontend | React 19 + TypeScript + Vite |
| State | Zustand 5 |
| Graph view | @xyflow/react (React Flow v12) |
| Styling | Plain CSS with design tokens (`tokens.css` → `base.css`) |
| Database | SQLite via sqlx |
| LLM | Google Gemini 2.5 Flash |
| Document parsing | Pure Rust: `pdf-extract`, `docx-rs`, `calamine`, `pptx-to-md`, `image` |
| Async runtime | Tokio (features: macros, rt-multi-thread, time, sync, io-util) |

---

## Repository layout

```
vectorless-grep/
├── src/                        # React frontend
│   ├── app/
│   │   ├── App.tsx             # Reasoning flow, event listeners
│   │   ├── AppShell.tsx        # Upload bar, document selector
│   │   ├── routes.tsx          # Route management
│   │   └── SettingsPage.tsx    # API key config
│   ├── features/
│   │   ├── answer/AnswerCard.tsx
│   │   ├── document/
│   │   │   ├── DocumentPane.tsx
│   │   │   └── EmbeddingProximity.tsx
│   │   ├── graph/
│   │   │   ├── GraphPane.tsx
│   │   │   └── GraphInsights.tsx
│   │   ├── status/StatusBar.tsx
│   │   ├── trace/TracePane.tsx
│   │   └── tree/TreePane.tsx
│   ├── lib/
│   │   ├── formatters.ts       # nodeTypeLabel, formatLatency
│   │   ├── state.ts            # Zustand store (single source of truth)
│   │   ├── tauriApi.ts         # All invoke() + listen() wrappers
│   │   └── types.ts            # TypeScript interfaces
│   └── styles/
│       ├── tokens.css          # ALL design tokens (colors, spacing, etc.)
│       └── base.css            # All component styles (uses tokens)
│
├── src-tauri/src/              # Rust backend
│   ├── commands/
│   │   ├── documents.rs        # ingest_document, get_tree, export_markdown …
│   │   ├── reasoning.rs        # run_reasoning_query, get_run
│   │   └── settings.rs         # set_provider_key
│   ├── core/
│   │   ├── errors.rs           # AppError enum + AppResult
│   │   └── types.rs            # Shared response types, NodeType enum
│   ├── db/
│   │   ├── mod.rs              # Database init (migrations)
│   │   └── repositories/
│   │       ├── documents.rs    # SQL CRUD + export_markdown
│   │       └── reasoning.rs    # Reasoning run storage
│   ├── providers/gemini.rs     # Gemini HTTP client
│   ├── reasoner/
│   │   ├── executor.rs         # Agent loop
│   │   ├── planner.rs
│   │   └── prompts.rs
│   ├── security/keyring.rs     # OS keyring for API key
│   ├── sidecar/
│   │   ├── native_parser.rs    # Document → Section → Paragraph hierarchy
│   │   └── types.rs            # SidecarNode, SidecarDocument, NormalizedPayload
│   └── lib.rs                  # AppState, Tauri builder, command registration
│
├── docs/
│   ├── agents.md               # Agent architecture spec
│   └── plan.md                 # Current roadmap and next steps
│
├── CLAUDE.md                   # This file
└── design_system.md            # Visual design reference
```

---

## Essential commands

```bash
# Frontend dev (Vite)
npm run dev

# Full Tauri dev (opens desktop window)
npm run tauri dev

# Rust type-check only (fast)
cd src-tauri && cargo check

# Rust clippy
cd src-tauri && cargo clippy

# Frontend lint
npx eslint src --ext .ts,.tsx --max-warnings 0

# Frontend type-check
npx tsc --noEmit
```

---

## Design system rules

**Never hardcode colors or pixel constants in component files.**

All visual values live in `src/styles/tokens.css`:
- Colors → `var(--bg-*)`, `var(--text-*)`, `var(--accent-*)`, `var(--ok/warn/danger)`
- Spacing → `var(--tree-indent-base)`, `var(--tree-indent-step)`, etc.
- Typography → `var(--font-sans)`, `var(--font-mono)`, `var(--font-size-xs)`
- Radius → `var(--radius-xs/sm/md/lg)`
- Shadows → `var(--shadow-1/2)`, `var(--shadow-inset)`

For dynamic values (computed from props), pass them via CSS custom properties as data:
```tsx
// CORRECT — sets a CSS variable, not an inline style rule
style={{ "--_depth": depth } as React.CSSProperties}

// WRONG — inline style rule
style={{ paddingLeft: `${depth * 16}px` }}
```

Then consume in CSS:
```css
.tree-depth-0 .tree-node-btn { padding-left: var(--tree-indent-base); }
.tree-depth-1 .tree-node-btn { padding-left: calc(var(--tree-indent-base) + 1 * var(--tree-indent-step)); }
```

Or for truly dynamic values (like bar fill percentages), class-based enumeration is not possible — in that case `--_fill` custom property is acceptable because it's data, not a style rule, and the actual CSS rule stays in base.css.

---

## State management

`src/lib/state.ts` — single Zustand store. Key flows:

- **Document switch** → `setActiveDocument(id)` clears tree, trace, answer, nodeDetail
- **Tree loaded** → `setTree(nodes)` auto-selects root node
- **Trace step** → `appendTraceStep(step)` updates activeNodeId to step.nodeRefs[0]
- **Query submit** → App.tsx `runQuery()` calls `runReasoningQuery`, listens for events

---

## Document parsing pipeline

```
File path + MIME type
    ↓
native_parser::parse()           (src-tauri/src/sidecar/native_parser.rs)
    ↓ produces NormalizedPayload
        Document node (root)
          ├── Section node 1
          │     ├── Paragraph 1.1
          │     └── Paragraph 1.2
          └── Section node 2
                └── Paragraph 2.1
    ↓
documents::insert_nodes()        (src-tauri/src/db/repositories/documents.rs)
    ↓ stored in SQLite doc_nodes table
    ↓
get_tree() command               → frontend TreePane
```

Heading detection heuristics (in `looks_like_heading()`):
- Markdown `#` prefix → always heading
- Ends with `.`/`?`/`!` → never heading
- Single-line, ≤12 words, starts uppercase or ≥65% uppercase letters → heading

---

## Adding a new Tauri command

1. Add function in `src-tauri/src/commands/<module>.rs` with `#[tauri::command]`
2. Register in `src-tauri/src/lib.rs` → `tauri::Builder::invoke_handler(tauri::generate_handler![...])`
3. Add typed wrapper in `src/lib/tauriApi.ts`
4. Add return type to `src/lib/types.ts` if needed

---

## Common pitfalls

- **`cargo check` must pass before every commit** — the CI is unforgiving
- **ESLint `no-inline-styles`** — webhint flags any `style={{ cssProperty: value }}`. Use CSS custom properties for dynamic data, static classes for static styles.
- **React Hook order** — hooks must come before any early returns. EmbeddingProximity.tsx had a bug here previously.
- **Tokio `"process"` feature removed** — it was only needed for the old docling Python sidecar (deleted). Do not re-add it.
- **Multi-file upload** — `pickDocumentFiles()` returns `string[]`. The old `pickDocumentFile()` function was removed; update any code that referenced it.
