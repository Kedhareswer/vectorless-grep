# Project-Scoped Explorer + Markdown Context + Dark Square UI Plan

## Summary
I’m using the `writing-plans` skill to create the implementation plan.

This plan will:
- Add full **project CRUD** and make data/querying project-scoped.
- Show Explorer as **Project > Documents > Document nodes** with node-type icons.
- Replace right panel tabs with a **single full rendered Markdown context view** and node highlight.
- Remove metadata/reasoning UI from the right panel.
- Make the UI **dark-neutral and zero-radius** across the app.
- Resolve current failures: TS6133 unused props, `react-refresh/only-export-components`, and failing `tree-pane` test.

Current verified baseline failures to fix:
- `npm run build`: TS6133 in `src/features/tree/TreePane.tsx` (unused `documents`, `activeDocumentId`, `onChangeDocument`).
- `npm run lint`: same unused vars + `react-refresh/only-export-components` in `src/features/tree/TreeIcons.tsx`.
- `npm run test -- tests/frontend/tree-pane.test.tsx`: expects `Active document` selector that is currently missing.

## Public API / Interface Changes
- Frontend type additions in `src/lib/types.ts`:
  - Add `ProjectSummary`.
  - Extend `DocumentSummary` with `projectId`.
  - Extend `ReasoningRun` with `projectId` and `documentId: string | null`.
- Frontend Tauri API changes in `src/lib/tauriApi.ts`:
  - `listDocuments(projectId: string)`.
  - `ingestDocument(..., projectId: string)`.
  - `runReasoningQuery(projectId: string, query: string, maxSteps?: number, focusDocumentId?: string | null)`.
  - Add `listProjects`, `createProject`, `renameProject`, `deleteProject`.
  - Add optional `getProjectTree(projectId, depth?)` (recommended) or equivalent multi-doc fetch path.
- Rust command surface:
  - New commands in `src-tauri/src/commands/projects.rs`: list/create/rename/delete project.
  - Update `src-tauri/src/commands/documents.rs`: project-aware ingest/list/tree.
  - Update `src-tauri/src/commands/reasoning.rs`: project-scoped query input.
  - Register new commands in `src-tauri/src/lib.rs`.

## Data Model and Migration Plan
1. Add migration `src-tauri/src/db/migrations/0003_projects.sql`:
- Create `projects` table: `id`, `name` (unique, case-insensitive), `created_at`, `updated_at`.
- Seed default project: id `project-default`, name `My Project`.
- Rebuild `documents` table to include `project_id NOT NULL` FK to `projects(id)` with `ON DELETE CASCADE`.
- Replace global checksum uniqueness with `UNIQUE(project_id, checksum)` to allow same file in different projects.
- Add `idx_documents_project`.
- Rebuild `reasoning_runs` to include `project_id NOT NULL` FK and `document_id` nullable.
- Backfill `reasoning_runs.project_id` by joining prior document ownership.
- Add `idx_runs_project`.

2. Update Rust core types in `src-tauri/src/core/types.rs`:
- Add `ProjectSummary`, project response payloads.
- Update `DocumentSummary` and `ReasoningRun` fields.

3. Update repositories:
- New file `src-tauri/src/db/repositories/projects.rs` for CRUD.
- Update `src-tauri/src/db/repositories/mod.rs` to export projects repo.
- Update `src-tauri/src/db/repositories/documents.rs` signatures for `project_id` scoping.
- Add project-wide node retrieval helper for cross-document reasoning candidate selection.

## Backend Behavior Changes
1. Documents commands/repo:
- `ingest_document` requires `project_id`.
- Dedup checksum is checked per project.
- `list_documents` returns only active-project documents.
- Add/adjust tree retrieval so Explorer can render multiple documents under one project.

2. Reasoning:
- `run_reasoning_query` becomes project-scoped.
- `ReasoningExecutor::run` receives `project_id` and optional `focus_document_id`.
- `pick_candidates` scans nodes across all documents in project.
- Step observations/citations remain node-id based, but evidence snippets include document context to reduce ambiguity.

3. Project delete:
- Deleting a project cascades its docs/nodes/layouts/runs/answers as requested.

## Frontend Implementation Plan
### Task 1: Project state and API wiring
- Modify `src/lib/types.ts`, `src/lib/tauriApi.ts`, `src/lib/state.ts`.
- Add store slices: `projects`, `activeProjectId`, project CRUD actions.
- Keep `activeDocumentId` for graph/right-panel focus.

### Task 2: Workspace chrome and load flow
- Modify `src/app/AppShell.tsx`, `src/features/navigation/WorkspaceChromeContext.tsx`.
- On startup: load projects, select default/first active project, then load project docs.
- Update upload flow to ingest into active project.
- Add project CRUD handlers exposed via context.

### Task 3: Explorer Tree redesign
- Modify `src/features/tree/TreePane.tsx`.
- Explorer hierarchy: `Project root` > `Documents` > `doc node hierarchy`.
- Show document name first, then its sections/subsections/tables/images/others.
- Keep collapse/expand per branch.
- Node selection sets both active node and active document.
- Remove obsolete selector expectation; use project-level header controls instead.

### Task 4: Icon cleanup + lint fix
- Split icon selector from component-export file to satisfy `react-refresh/only-export-components`.
- Files:
  - `src/features/tree/TreeIcons.tsx` (components only).
  - Add `src/features/tree/getNodeIcon.tsx` (or equivalent selector helper).
- Use monochrome icon color via CSS (`currentColor`).

### Task 5: Right panel redesign to full Markdown context
- Rewrite `src/features/document/DocumentPane.tsx`:
  - Remove `Content/Metadata/Reasoning` tabs.
  - Render complete selected-document context as markdown blocks (via `react-markdown`).
  - Highlight selected node block persistently.
  - Auto-scroll to selected block on tree selection.
  - Keep header minimal: document title + markdown body only.
- Remove metadata/reasoning panels and related dead code paths.

### Task 6: App orchestration updates
- Modify `src/app/App.tsx`:
  - Load tree data for project-level explorer.
  - Derive active-document nodes for Graph and right pane.
  - Query execution uses `activeProjectId`.
  - Keep center panel behavior (trace/graph) with graph scoped to active document.

### Task 7: Theme and shape normalization
- Modify `src/styles/tokens.css`, `src/styles/base.css`:
  - Set radius tokens to `0`.
  - Remove/override explicit rounded values (`999px`, `5px`, `6px`, etc.) to square.
  - Remove blue-tinted tree refresh overrides and align to neutral black/charcoal tokens.
  - Ensure selected/hover states keep readable contrast.

## Test Plan
### Frontend tests
- Update `tests/frontend/tree-pane.test.tsx`:
  - Replace old “Active document selector” assertion with:
    - project root rendered,
    - multiple documents rendered,
    - node hierarchy rendered with typed icons.
- Add/extend tests for `DocumentPane`:
  - full markdown rendering from full document context,
  - selected node auto-scroll invocation,
  - persistent highlight class on selected block,
  - metadata/reasoning tabs absent.
- Add project workflow tests (or AppShell-level tests):
  - create/switch/rename/delete project actions update visible docs.

### Backend tests
- Update/add repository tests in:
  - `src-tauri/tests/db_repository_tests.rs`
  - `src-tauri/tests/reasoning_event_tests.rs`
- Validate:
  - project CRUD,
  - per-project checksum dedupe behavior,
  - project-scoped `list_documents`,
  - cross-document candidate retrieval under same project,
  - project delete cascade behavior.

### Verification commands
- `npm run lint`
- `npm run build`
- `npm run test`
- `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions and Defaults
- Default migrated project: `My Project` with id `project-default`.
- Project names are unique (case-insensitive).
- Cross-document reasoning runs across all docs in active project.
- Graph view remains scoped to currently selected document.
- Explorer controls for project CRUD live in Explorer header.
- Right panel is simplified to title + markdown context only.
- Monochrome icons are used for all node types.
- Entire app uses square corners (no rounded edges).
