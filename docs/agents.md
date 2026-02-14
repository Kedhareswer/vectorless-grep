# Agents Architecture

How the reasoning agent works in Vectorless, and what each component does.

---

## Overview

The reasoning system is a **multi-step ReAct agent** (Reason → Act → Observe loop) that reads the document AST from SQLite, calls Gemini to generate reasoning steps, and synthesises a final grounded answer.

```
User Query
    │
    ▼
ReasoningExecutor.run()
    │  (async loop, max N steps)
    │
    ├─── Step 1: Planner.plan()
    │       LLM generates: thought, action, stepType
    │
    ├─── Step 2: Execute action
    │       (retrieve node, scan section, navigate, synthesise)
    │
    ├─── Step 3: Observe
    │       Result appended to context window
    │
    └─── (repeat until answer or max steps)
            │
            ▼
    FinalAnswer: markdown + citations + confidence
```

---

## Agent components

### `src-tauri/src/reasoner/executor.rs` — ReasoningExecutor

The main agent loop:

1. Creates a `ReasoningRun` record in SQLite (`status = "running"`)
2. Builds the initial prompt context: document title + tree summary
3. Iterates up to `max_steps` (default: 6):
   - Calls `Planner::next_step()` → gets `thought`, `action`, `step_type`
   - Executes the action against the document DB
   - Emits `reasoning/step` Tauri event (real-time frontend update)
   - Appends observation to the running context
4. Calls `Planner::synthesise()` to generate the final answer
5. Stores steps + answer in SQLite
6. Emits `reasoning/complete` or `reasoning/error`

### `src-tauri/src/reasoner/planner.rs` — Planner

Stateless LLM call wrapper:
- `next_step(context) → ReasoningStep` — single step generation
- `synthesise(context, steps) → AnswerRecord` — final answer synthesis

### `src-tauri/src/reasoner/prompts.rs` — Prompts

System and user prompts for the agent. Key templates:
- `SYSTEM_PROMPT` — role definition, output format (JSON), capabilities
- `step_prompt(context, history)` — builds the per-step prompt
- `synthesis_prompt(query, steps)` — builds the final answer prompt

### `src-tauri/src/providers/gemini.rs` — GeminiClient

HTTP client for the Gemini REST API:
- Model: `gemini-2.0-flash`
- Authentication: API key from OS keyring (set via Settings page)
- Output: structured JSON parsed into `ReasoningStep` or `AnswerRecord`

---

## Step types

| Step type | Description |
|-----------|-------------|
| `RETRIEVE` | Fetch a specific node by ID or title |
| `SCAN` | Scan a section's children for relevant content |
| `NAVIGATE` | Move up/down the AST (parent, children, siblings) |
| `EXTRACT` | Extract a specific fact from node text |
| `SYNTHESIZE` | Combine observations into partial answer |
| `FINALIZE` | Generate the final grounded answer |

---

## Data flow (frontend)

```
App.tsx
├── runReasoningQuery(documentId, query, maxSteps)
│       → Tauri: run_reasoning_query command
│       ← { runId, status: "running" }
│
├── onReasoningStep listener
│       ← ReasoningStepEvent { runId, stepIndex, stepType, thought, action, observation, confidence, latencyMs }
│       → appendTraceStep(mapped) → TracePane re-renders
│
├── onReasoningComplete listener
│       ← ReasoningCompleteEvent { runId }
│       → getRun(runId) → { run, steps, answer }
│       → setTrace(steps), setCurrentRun(run), setAnswer(answer)
│
└── onReasoningError listener
        ← ReasoningErrorEvent { runId, message }
        → setPending(false), setErrorMessage(message)
```

---

## Extending the agent

### Adding a new step type

1. Add variant to `StepType` enum in `src-tauri/src/core/types.rs`
2. Add handling in `executor.rs` (what DB query to run for this action)
3. Add `stepTypeClass()` mapping in `src/features/trace/TracePane.tsx`
4. Add CSS class `.is-<steptype>` in `base.css` for the timeline dot colour

### Changing the LLM provider

1. Implement a new `ProviderClient` trait (modelled on `GeminiClient`)
2. Swap in `AppState` in `lib.rs`
3. Store the new API key via `set_provider_key` with a new `provider` string

### Increasing context quality

- Increase `max_steps` parameter (frontend sends this to `run_reasoning_query`)
- Improve `prompts.rs` to include more document context per step
- Add embedding-based node ranking before retrieval steps

---

## Evaluation

To test reasoning quality:
1. Ingest a known document
2. Run `export_markdown` to get ground truth
3. Ask questions whose answers appear verbatim in the markdown
4. Check `answer.grounded == true` and `citations[]` point to correct nodes

The `eval/` directory (if present) contains sample documents and expected QA pairs.
