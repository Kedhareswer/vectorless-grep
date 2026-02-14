---
inclusion: always
---

# Technology Stack

## Architecture

Vectorless is a Tauri 2 desktop application with a Rust backend and React frontend.

## Backend (Rust)

- Runtime: Tauri 2.10.0
- Database: SQLite with sqlx (migrations in `src-tauri/src/db/migrations/`)
- Document parsing: Pure Rust libraries
  - PDF: `pdf-extract`
  - DOCX: `docx-rs` (with XML fallback)
  - PPTX: `pptx-to-md`
  - XLSX/CSV: `calamine`
  - Images: `image` crate (metadata extraction)
  - Text/Markdown: Built-in with heading detection heuristics
- LLM provider: Gemini API via `reqwest` (cloud-only)
- Security: `keyring` for secure API key storage (Windows native)
- Async runtime: Tokio

## Frontend (TypeScript/React)

- Framework: React 19 with TypeScript
- Build tool: Vite 7
- State management: Zustand
- Data fetching: TanStack React Query
- Graph visualization: @xyflow/react
- Routing: Custom lightweight router (see `src/app/routes.tsx`)
- Styling: CSS with design tokens (see `src/styles/`)

## Testing

- Frontend unit tests: Vitest with jsdom and React Testing Library
- E2E tests: Playwright
- Backend tests: Cargo test framework
- Parser E2E tests: 23 comprehensive tests covering all document formats
- Test location: `tests/frontend/` for React, `tests/e2e/` for Playwright, `src-tauri/tests/` for Rust
- Test fixtures: `src-tauri/tests/fixtures/` with sample files and synthetic tests

## Common Commands

```bash
# Setup (Windows only - installs Node, Rust, MSVC if missing)
npm run setup:win

# Development
npm run tauri:dev          # Run Tauri app in dev mode
npm run dev                # Run Vite dev server only

# Building
npm run build              # Build frontend
npm run tauri:build        # Build complete Tauri app

# Testing
npm run test               # Run frontend unit tests (Vitest)
npm run test:watch         # Run tests in watch mode
npm run test:e2e           # Run E2E tests (Playwright)
cargo test --manifest-path src-tauri/Cargo.toml  # Run Rust tests

# Code quality
npm run lint               # Run ESLint
npm run check              # Run lint + tests

# Preview
npm run preview            # Preview production build
```

## Environment Variables

- `VECTORLESS_LOG`: Set log level (trace, debug, info, warn, error)
- `VECTORLESS_SQLX_DEBUG`: Enable SQLx query logging (1, true, yes, on)

## Module Organization

Backend modules (Rust):
- `commands/`: Tauri command handlers (exposed to frontend)
- `core/`: Core types and error handling
- `db/`: Database layer with repositories pattern
- `providers/`: LLM provider integrations (Gemini)
- `reasoner/`: Reasoning planner and executor
- `security/`: Keyring integration for secure storage
- `sidecar/`: Native document parser

Frontend modules (TypeScript):
- `app/`: App shell, routing, settings
- `features/`: Feature-based components (answer, document, graph, navigation, query, status, trace, tree)
- `lib/`: Shared utilities (state, API client, types, formatters)
- `styles/`: Global styles and design tokens
