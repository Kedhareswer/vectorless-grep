# Agentic Reasoning Rebuild Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace deterministic pseudo-agent behavior with real model-driven planning, add answer quality evaluation, and redesign the run UI/workflow so outputs feel reliable and product-grade.

**Architecture:** Introduce a true plan-act-observe loop where Gemini emits structured next-step plans each turn, the executor runs those actions against document repositories, and a deterministic quality evaluator validates answer/query alignment + grounding before finalizing. Persist workflow phases and quality metrics in run records so UI reflects real system state (not cosmetic timeline theater).

**Tech Stack:** Rust (Tauri 2 backend, sqlx + SQLite FTS5, Tokio), Gemini REST API JSON mode, React 19 + TypeScript + Zustand, CSS token system in `src/styles/tokens.css` + `src/styles/base.css`.

---

### Task 1: Define Agent Contract (Step Plan Schema + Workflow Phases)

**Files:**
- Create: `src-tauri/src/reasoner/agent_schema.rs`
- Modify: `src-tauri/src/core/types.rs`
- Modify: `src-tauri/src/db/migrations/0006_reasoning_workflow.sql`
- Modify: `src-tauri/src/db/repositories/reasoning.rs`
- Test: `src-tauri/tests/reasoning_workflow_schema_tests.rs`

**Step 1: Write the failing test**

Create tests for:
- Allowed planner step kinds (`search`, `inspect`, `synthesize`, `self_check`, `finish`)
- Required fields (`objective`, `reasoning`, `params`, `stop`) and phase enum validation.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_workflow_schema_tests`
Expected: FAIL with missing schema/types/migration columns.

**Step 3: Write minimal implementation**

- Add `AgentPlannedStep`, `RunPhase`, `QualityMetrics` structs.
- Add DB columns: `phase`, `quality_json`, `planner_trace_json`.
- Add serialization/deserialization and repository mapping.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_workflow_schema_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/src/reasoner/agent_schema.rs src-tauri/src/core/types.rs src-tauri/src/db/migrations/0006_reasoning_workflow.sql src-tauri/src/db/repositories/reasoning.rs src-tauri/tests/reasoning_workflow_schema_tests.rs && git commit -m "feat(reasoning): define agent step schema and workflow phases"`

---

### Task 2: Replace Deterministic Planner With LLM Planner Calls

**Files:**
- Modify: `src-tauri/src/providers/gemini.rs`
- Modify: `src-tauri/src/reasoner/planner.rs`
- Modify: `src-tauri/src/reasoner/prompts.rs`
- Test: `src-tauri/tests/planner_tests.rs`
- Test: `src-tauri/tests/agentic_planner_tests.rs`

**Step 1: Write the failing test**

Add tests that assert:
- Planner consumes model JSON to produce next step.
- Invalid JSON falls back safely to `search` then `self_check` (not panic).
- Planner can emit `finish` only when evidence exists.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test planner_tests --test agentic_planner_tests`
Expected: FAIL because planner is still static sequence logic.

**Step 3: Write minimal implementation**

- Add `GeminiClient::generate_plan_step(...)` returning validated step JSON.
- Refactor `Planner::next_steps(...)` to call model with compact run state + recent observations.
- Keep deterministic fallback for provider/network errors.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test planner_tests --test agentic_planner_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/src/providers/gemini.rs src-tauri/src/reasoner/planner.rs src-tauri/src/reasoner/prompts.rs src-tauri/tests/planner_tests.rs src-tauri/tests/agentic_planner_tests.rs && git commit -m "feat(reasoning): switch planner to model-driven step generation"`

---

### Task 3: Real Plan-Act-Observe Executor Workflow

**Files:**
- Modify: `src-tauri/src/reasoner/executor.rs`
- Modify: `src-tauri/src/commands/reasoning.rs`
- Modify: `src-tauri/src/db/repositories/documents.rs`
- Test: `src-tauri/tests/reasoning_event_tests.rs`
- Test: `src-tauri/tests/reasoning_workflow_tests.rs`

**Step 1: Write the failing test**

Add tests for:
- Executor phase transitions (`planning -> retrieval -> synthesis -> validation -> completed`).
- Backtrack path when self-check score below threshold.
- Multi-doc relation query remains project-scoped and diverse by document.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_event_tests --test reasoning_workflow_tests`
Expected: FAIL on phase/loop behavior.

**Step 3: Write minimal implementation**

- Executor consumes planner step output each iteration.
- Add bounded retries + stop conditions driven by planner + quality gate.
- Keep retrieval actions explicit and auditable (`search`, `inspect`, `expand_neighbors`).

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_event_tests --test reasoning_workflow_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/src/reasoner/executor.rs src-tauri/src/commands/reasoning.rs src-tauri/src/db/repositories/documents.rs src-tauri/tests/reasoning_event_tests.rs src-tauri/tests/reasoning_workflow_tests.rs && git commit -m "feat(reasoning): implement real plan-act-observe workflow executor"`

---

### Task 4: Add Answer Quality Evaluator (Deterministic + LLM-Aware)

**Files:**
- Create: `src-tauri/src/reasoner/evaluator.rs`
- Modify: `src-tauri/src/reasoner/mod.rs`
- Modify: `src-tauri/src/reasoner/executor.rs`
- Test: `src-tauri/tests/answer_quality_tests.rs`

**Step 1: Write the failing test**

Add tests for metrics:
- `query_alignment` (query intent terms answered)
- `citation_coverage` (claims supported by citations)
- `cross_document_coverage` (for relation queries)
- `grounding_fail` when answer has zero valid citations.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test answer_quality_tests`
Expected: FAIL because evaluator module does not exist.

**Step 3: Write minimal implementation**

- Implement `evaluate_answer(query, answer, citations, evidence)` returning `QualityMetrics`.
- Add acceptance threshold in executor; trigger one revision loop if below threshold.
- Persist metrics into run/answer records.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test answer_quality_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/src/reasoner/evaluator.rs src-tauri/src/reasoner/mod.rs src-tauri/src/reasoner/executor.rs src-tauri/tests/answer_quality_tests.rs && git commit -m "feat(reasoning): add answer quality evaluator and validation gate"`

---

### Task 5: Redesign UX Language + Workflow UI (Kill "Timeline")

**Files:**
- Modify: `src/features/trace/TracePane.tsx`
- Modify: `src/app/App.tsx`
- Modify: `src/lib/types.ts`
- Modify: `src/styles/base.css`
- Test: `tests/frontend/trace-pane.test.tsx`

**Step 1: Write the failing test**

Add frontend tests for:
- Heading changed to `Reasoning Workflow`.
- Stages rendered as phase chips (`Plan`, `Retrieve`, `Draft`, `Validate`, `Final`).
- Debug internals hidden by default; reveal only with explicit toggle.

**Step 2: Run test to verify it fails**

Run: `npm run test -- tests/frontend/trace-pane.test.tsx`
Expected: FAIL because old labels (`Reasoning Timeline`) still exist.

**Step 3: Write minimal implementation**

- Replace timeline naming and copy.
- Introduce workflow-centric components in same pane (no fake “thought theater” in primary view).
- Keep developer diagnostics behind `Show Debug Details` toggle.

**Step 4: Run test to verify it passes**

Run: `npm run test -- tests/frontend/trace-pane.test.tsx`
Expected: PASS.

**Step 5: Commit**

`git add src/features/trace/TracePane.tsx src/app/App.tsx src/lib/types.ts src/styles/base.css tests/frontend/trace-pane.test.tsx && git commit -m "feat(ui): redesign run panel as reasoning workflow and hide debug by default"`

---

### Task 6: Build Real Workflow Observability + Failure Handling

**Files:**
- Modify: `src-tauri/src/core/types.rs`
- Modify: `src-tauri/src/commands/reasoning.rs`
- Modify: `src/lib/tauriApi.ts`
- Modify: `src/lib/state.ts`
- Test: `src-tauri/tests/reasoning_event_tests.rs`

**Step 1: Write the failing test**

Add tests ensuring emitted events include:
- `phase`, `qualityScore`, `retryCount`, `stopReason`.
- failure categories (`provider_error`, `retrieval_empty`, `quality_rejected`).

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_event_tests`
Expected: FAIL due to missing event fields.

**Step 3: Write minimal implementation**

- Extend event payloads and frontend mapping.
- Show hard failure reason and guidance in UI.
- Add guard rails for empty evidence and malformed model output.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_event_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/src/core/types.rs src-tauri/src/commands/reasoning.rs src/lib/tauriApi.ts src/lib/state.ts src-tauri/tests/reasoning_event_tests.rs && git commit -m "feat(reasoning): expose workflow phases and failure semantics in events"`

---

### Task 7: Quality Regression Suite + Golden Queries

**Files:**
- Create: `src-tauri/tests/reasoning_quality_regression_tests.rs`
- Create: `src-tauri/tests/fixtures/quality_queries.json`
- Modify: `docs/plan.md`
- Modify: `README.md`

**Step 1: Write the failing test**

Create golden queries including:
- “Explain what these files are about and how they are related”
- compare/difference prompts
- single-file factual prompts

Add assertions on:
- non-empty answer,
- citations present,
- relation queries include >=2 docs when available,
- quality score threshold.

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_quality_regression_tests`
Expected: FAIL until evaluator + workflow outputs are wired.

**Step 3: Write minimal implementation**

- Wire fixtures to in-memory DB scenarios.
- Add deterministic acceptance checks and clear failure diffs.

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test reasoning_quality_regression_tests`
Expected: PASS.

**Step 5: Commit**

`git add src-tauri/tests/reasoning_quality_regression_tests.rs src-tauri/tests/fixtures/quality_queries.json docs/plan.md README.md && git commit -m "test(reasoning): add regression suite for answer quality and multi-doc relation queries"`

---

### Task 8: Final Verification Gate

**Files:**
- Modify: `docs/plan.md`

**Step 1: Run full backend verification**

Run: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all checks pass (or document known unrelated failures explicitly).

**Step 2: Run frontend verification**

Run: `npx tsc --noEmit && npx eslint src --ext .ts,.tsx --max-warnings 0 && npm run test`
Expected: pass.

**Step 3: Update roadmap/progress**

Mark completed items and capture residual known issues.

**Step 4: Commit**

`git add docs/plan.md && git commit -m "docs: record completion status for agentic reasoning rebuild"`

---

## Naming + UX Direction (explicit)

- Rename center pane title from `Reasoning Timeline` to `Reasoning Workflow`.
- Replace step labels with stage labels users understand: `Plan`, `Retrieve`, `Draft`, `Validate`, `Final`.
- Keep model “thought/action” text in a secondary `Debug` panel only.
- In answer card, add `Quality` badge next to confidence with tooltip explaining metrics.
- Make failure states explicit and non-gaslighting (no `SUCCESS` badge if quality gate failed).

---

## Non-Goals (to avoid scope creep)

- No vector DB adoption in this track.
- No new LLM provider in this track.
- No full parser rewrite in this track.

---

## Risk Controls

- Keep deterministic fallback for planner if provider JSON is invalid.
- Bound retries (`max_revision_loops = 1` initially).
- Keep old fields backward-compatible in API payloads during migration.
- Add snapshot tests for UI labels so naming regressions are caught quickly.
