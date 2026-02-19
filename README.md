# Vectorless — RAG That Thinks, Not Just Searches

Vectorless is a **structure‑grounded RAG desktop app** that replaces vector embeddings with **document‑AST traversal** and **multi‑step reasoning traces**. Instead of embedding chunks and searching by cosine similarity, Vectorless builds a hierarchical tree of your documents and runs a Gemini‑powered agent that **navigates the tree, selects relevant sections, and synthesises answers with full provenance**.

## 🧩 The Core Idea: Why Ditch Vectors?

Traditional RAG works like a **search engine**: chunk text → embed → query → retrieve top‑k chunks. This approach has inherent limitations:

- **Chunk‑boundary problems** – Context gets cut mid‑sentence.
- **Embedding drift** – Semantic similarity doesn’t guarantee factual relevance.
- **Black‑box retrieval** – You can’t see why a chunk was selected.
- **Cost & latency** – Embedding models (GPU/API) add overhead.

Vectorless flips the model: treat the document as a **tree** (Document → Section → Paragraph) and let an LLM **reason over the structure**.

```
Traditional RAG Pipeline:
    Document → Chunk → Embed → Query → Top‑K → Answer

Vectorless Pipeline:
    Document → Parse → AST → Agent Navigation → Evidence → Answer
                     │                         │
                     └─── Structure‑aware ────┘
```

## 📊 How It’s Different: A Side‑by‑Side Comparison

| Aspect | Traditional RAG | Vectorless |
|--------|----------------|------------|
| **Retrieval Mechanism** | Vector similarity over chunk embeddings | AST traversal + LLM‑guided section selection |
| **Indexing Cost** | Embedding generation (GPU/API) | Pure‑rust parsing → hierarchical nodes (zero embedding cost) |
| **Explainability** | Black‑box “top‑k chunks” | Step‑by‑step reasoning trace with cited nodes |
| **Latency Profile** | Embedding query + similarity search | Direct tree navigation + LLM planning loops |
| **Accuracy Leverage** | Depends on chunk boundaries & embedding quality | Leverages document structure (headings, paragraphs) for precise context |
| **Infrastructure** | Requires embedding model & vector DB | Single SQLite file, no external vector store |
| **Multi‑Document Queries** | Merge chunks from multiple docs | Currently single‑document; cross‑doc support planned (P0) |

## 🏗️ The Three Novel Components of Vectorless

### 1. **Document AST (Abstract Syntax Tree)**
Every ingested document is parsed into a **hierarchical node tree**:

```yaml
Document: “Annual Report.pdf”
├── Section: “Executive Summary” (heading‑level 1)
│   ├── Paragraph: “Revenue grew 15%...”
│   └── Paragraph: “Market share expanded...”
├── Section: “Financials” (heading‑level 1)
│   ├── Sub‑section: “Q1 Results” (heading‑level 2)
│   │   └── Paragraph: “Net profit $2.3M...”
│   └── Sub‑section: “Q2 Projections” (heading‑level 2)
└── ...
```

- **Built entirely in Rust** – no Python, no external services.
- **Supports PDF, DOCX, PPTX, XLSX, CSV, images, text, Markdown**.
- **Persisted in SQLite** with full referential integrity.

### 2. **Reasoning Agent with Planner Loop**
The agent follows a **ReAct‑style loop** (`Planner::next_step`) with six distinct step types:

```rust
enum StepType {
    ScanRoot,          // Get overview of the document
    SelectSections,    // Choose promising sections
    DrillDown,         // Navigate deeper into a subtree
    ExtractEvidence,   // Pull relevant text from nodes
    Synthesize,        // Combine evidence into answer
    SelfCheck,         // Verify answer against source
}
```

Each step produces a **thought**, an **action**, and an **observation** that’s appended to the context window. The loop runs for up to 6 steps, then calls `Planner::synthesise()` to generate the final answer.

### 3. **Visual Trace & Graph UI**
Vectorless provides **three interactive panes** that let you see exactly how the answer was built:

- **Tree Pane** – The document AST as a collapsible outline.
- **Trace Pane** – A **timeline of reasoning steps** with confidence scores, latency, and node citations.
- **Graph Pane** – A **React Flow graph** showing the AST as a node‑edge diagram, with cluster/hierarchy layouts.

**Example trace for query** “What was the revenue growth?”

```
┌─────────────────────────────────────────────────────────────┐
│ Step 1: ScanRoot                                            │
│   Thought: “I need to locate financial sections.”           │
│   Action: Retrieve root nodes                               │
│   Observation: Found “Executive Summary”, “Financials”      │
├─────────────────────────────────────────────────────────────┤
│ Step 2: SelectSections                                      │
│   Thought: “Financials likely contains revenue figures.”    │
│   Action: Select section “Financials” (node‑id: 42)        │
│   Observation: Section has 2 sub‑sections                   │
├─────────────────────────────────────────────────────────────┤
│ Step 3: DrillDown                                           │
│   Thought: “Drill into Q1 Results.”                         │
│   Action: Navigate to child node “Q1 Results”              │
│   Observation: Paragraph “Revenue grew 15%...” cited        │
├─────────────────────────────────────────────────────────────┤
│ Step 4: ExtractEvidence                                     │
│   Thought: “Extract the exact percentage.”                  │
│   Action: Pull text from node 47                            │
│   Observation: “Revenue grew 15% year‑over‑year.”           │
├─────────────────────────────────────────────────────────────┤
│ Step 5: Synthesize                                          │
│   Thought: “Formulate answer.”                              │
│   Action: Generate final answer with citation               │
│   Observation: Answer ready, confidence 0.92                │
└─────────────────────────────────────────────────────────────┘
```

## 📈 Performance & Benchmarks

| Metric | Vectorless (AST‑based) | Traditional RAG (embedding‑based) |
|--------|------------------------|-----------------------------------|
| **Indexing time** (10‑page PDF) | ~2s (Rust parsing) | ~5‑10s (embedding generation) |
| **Query latency** (first answer) | ~3‑6s (6‑step loop) | ~1‑3s (vector search + LLM) |
| **Token usage per query** | ~2‑3k (step‑wise context) | ~1‑2k (chunk + prompt) |
| **Accuracy on structured docs** | **Higher** (uses headings) | Lower (chunk‑boundary noise) |
| **Explainability** | **Full trace** | Limited to chunk scores |

*Note: Benchmarks based on internal testing with Gemini 2.5 Flash; your mileage may vary.*

## 🚀 Getting Started in 5 Minutes

### 1. Install & Run
```bash
# One‑time setup (auto‑installs Node, Rust, MSVC if missing)
npm run setup:win

# Launch the app
npm run tauri:dev
```

### 2. Add Your Gemini Key
Paste your [Google AI Studio](https://aistudio.google.com/apikey) key into the app header (`Save Key` button).

### 3. Ingest a Document
Drag‑and‑drop a PDF, DOCX, or text file into the upload area. Watch the **progress chips** as each file is parsed.

### 4. Ask a Question
Type a question in the query bar, press Enter, and watch the **Trace Pane** light up with the agent’s steps. The answer appears in the **Answer Card** with citations you can click to jump to the source.

## 🧪 Test Suite & Quality

Vectorless comes with a **comprehensive test suite** to ensure reliability:

```bash
# Frontend unit tests (Vitest)
npm run test

# Backend Rust tests
cargo test --manifest‑path src‑tauri/Cargo.toml

# Parser E2E tests (23 tests covering all document formats)
cargo test --manifest‑path src‑tauri/Cargo.toml --test parser_e2e_tests

# End‑to‑end UI tests (Playwright)
npm run test:e2e
```

## 🗺️ Project Structure

```
vectorless‑grep/
├── src/                    # React frontend
│   ├── app/               # App shell, routing, settings
│   ├── features/          # Tree, trace, graph, answer panes
│   ├── lib/               # Zustand store, formatters, types
│   └── styles/            # Design tokens & base CSS
├── src‑tauri/             # Rust backend
│   ├── src/               # Commands, DB, reasoner, parsers
│   ├── db/migrations/     # SQLite schema
│   └── tests/             # Integration & E2E tests
├── docs/                  # Architecture, agents, plan
└── tests/                 # Frontend & E2E test suites
```

## 📚 Deep Dives

- [CLAUDE.md](CLAUDE.md) – Developer guide for working with the codebase.
- [docs/agents.md](docs/agents.md) – Deep dive into the reasoning‑agent architecture.
- [docs/plan.md](docs/plan.md) – Current state, known issues, and roadmap.

## 🐛 Known Limitations & What’s Next

- **Cross‑document reasoning** – Currently limited to a single active document; multi‑document support is a **P0 priority**.
- **PDF heading detection** – Relies on blank‑line separation; continuous‑flow PDFs may produce oversized sections.
- **Streaming token display** – Steps are shown after completion, not token‑by‑token.

See [docs/plan.md](docs/plan.md) for the full backlog and upcoming features.

---

**Vectorless** – RAG that thinks, not just searches.
