---
inclusion: always
---

# Product Overview

Vectorless is a Windows-first desktop application for structure-grounded RAG (Retrieval-Augmented Generation). Unlike traditional vector-based RAG systems, it uses Doc-AST (Document Abstract Syntax Tree) traversal combined with reasoning traces instead of vector embeddings.

## Core Capabilities

- Native document parsing for PDF, DOCX, PPTX, XLSX, CSV, Images (JPG/PNG), plain text, and Markdown
- Hierarchical document structure persistence using Doc-AST
- Reasoning planner loop with explicit steps: ScanRoot → SelectSections → DrillDown → ExtractEvidence → Synthesize → SelfCheck
- Explainable trace events for transparency (ingest/progress, reasoning/step, reasoning/complete, reasoning/error)
- 3-pane UI: Tree view | Trace/Graph view | Document + Answer view
- Project-based document organization

## Key Differentiators

- Structure-grounded approach: leverages document hierarchy rather than semantic embeddings
- Reasoning transparency: every step in the query process is visible and traceable
- Native performance: pure Rust document parsing with no external dependencies
- Desktop-first: runs locally with cloud LLM integration (Gemini API)

## User Workflow

1. Ingest documents to build Doc-AST representation
2. Store Gemini API key for LLM access
3. Query documents using natural language
4. View reasoning trace showing how the answer was derived
5. Explore document structure via tree or graph visualization
