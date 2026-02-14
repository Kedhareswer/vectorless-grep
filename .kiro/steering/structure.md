---
inclusion: always
---

# Project Structure

## Root Layout

```
vectorless/
├── src/                    # Frontend React application
├── src-tauri/              # Rust backend
├── tests/                  # Test suites
├── scripts/                # Setup and bootstrap scripts
├── docs/                   # Documentation
└── public/                 # Static assets
```

## Frontend Structure (`src/`)

Feature-based organization with shared libraries:

```
src/
├── app/                    # Application shell and routing
│   ├── App.tsx            # Main app component
│   ├── AppShell.tsx       # Layout wrapper
│   ├── routes.tsx         # Custom router
│   └── SettingsPage.tsx   # Settings UI
├── features/              # Feature modules (one per domain)
│   ├── answer/            # Answer display
│   ├── document/          # Document viewer and embedding proximity
│   ├── graph/             # Graph visualization
│   ├── navigation/        # Utility rail and workspace chrome
│   ├── query/             # Query input (empty - to be implemented)
│   ├── status/            # Status bar
│   ├── trace/             # Reasoning trace viewer
│   └── tree/              # Tree view and icons
├── lib/                   # Shared utilities
│   ├── formatters.ts      # Display formatting helpers
│   ├── state.ts           # Zustand store
│   ├── tauriApi.ts        # Tauri command wrappers
│   └── types.ts           # TypeScript type definitions
├── styles/                # Global styles
│   ├── base.css           # Base styles
│   └── tokens.css         # Design tokens
├── test/                  # Test setup
│   └── setup.ts           # Vitest configuration
└── main.tsx               # Application entry point
```

## Backend Structure (`src-tauri/src/`)

Layered architecture with clear separation of concerns:

```
src-tauri/src/
├── commands/              # Tauri command handlers (API layer)
│   ├── documents.rs       # Document operations
│   ├── projects.rs        # Project management
│   ├── reasoning.rs       # Query execution
│   ├── settings.rs        # Settings and API keys
│   └── mod.rs
├── core/                  # Core types and errors
│   ├── errors.rs          # Error types
│   ├── types.rs           # Shared types
│   └── mod.rs
├── db/                    # Database layer
│   ├── migrations/        # SQL migration files
│   ├── repositories/      # Repository pattern implementations
│   │   ├── documents.rs
│   │   ├── projects.rs
│   │   ├── reasoning.rs
│   │   └── mod.rs
│   └── mod.rs
├── providers/             # External service integrations
│   ├── gemini.rs          # Gemini API client
│   └── mod.rs
├── reasoner/              # Reasoning engine
│   ├── executor.rs        # Reasoning execution
│   ├── planner.rs         # Step planning
│   ├── prompts.rs         # LLM prompts
│   └── mod.rs
├── security/              # Security utilities
│   ├── keyring.rs         # Secure key storage
│   └── mod.rs
├── sidecar/               # Document parsing
│   ├── native_parser.rs   # Parser implementations
│   ├── types.rs           # Parser types
│   └── mod.rs
├── lib.rs                 # Library entry point
└── main.rs                # Binary entry point
```

## Test Structure

```
tests/
├── frontend/              # Frontend unit tests (Vitest)
│   ├── answer-card.test.tsx
│   ├── state.test.ts
│   ├── status-bar.test.tsx
│   ├── tree-pane.test.tsx
│   └── utility-rail.test.tsx
└── e2e/                   # End-to-end tests (Playwright)
    └── app.spec.ts

src-tauri/tests/           # Backend tests (Cargo)
├── db_repository_tests.rs
├── native_parser_docx_tests.rs
├── parser_e2e_tests.rs    # Comprehensive parser tests (23 tests, all passing)
├── planner_tests.rs
├── reasoning_event_tests.rs
├── sidecar_schema_tests.rs
└── fixtures/              # Test fixtures
    ├── pdf/               # PDF samples (optional)
    ├── docx/              # DOCX samples (has synthetic tests)
    ├── xlsx/              # Excel/CSV samples
    ├── pptx/              # PowerPoint samples (optional)
    ├── images/            # Image samples (has synthetic tests)
    └── text/              # Text/Markdown samples ✅
```

## Key Conventions

- Frontend components use PascalCase (e.g., `AnswerCard.tsx`)
- Feature modules are self-contained with related components
- Backend uses repository pattern for data access
- Database migrations are numbered sequentially (0001_, 0002_, etc.)
- Tauri commands are exposed via `commands/` module and registered in `lib.rs`
- State management is centralized in `src/lib/state.ts` using Zustand
- API calls to backend use wrapper functions in `src/lib/tauriApi.ts`
- Path alias `@/` maps to `src/` directory (configured in `vite.config.ts`)
