# Introduction to Vectorless

Vectorless is a structure-grounded RAG system that uses document hierarchy instead of vector embeddings.

## Key Features

The system provides several important capabilities:

- Native document parsing for multiple formats
- Hierarchical structure preservation using Doc-AST
- Reasoning transparency with explicit trace events
- Desktop-first architecture with cloud LLM integration

## Architecture

The application is built using Tauri 2 with a Rust backend and React frontend.

### Backend Components

The backend handles document parsing, database operations, and LLM integration.

### Frontend Components

The frontend provides a 3-pane interface for document exploration and query visualization.

## Getting Started

To use Vectorless, first ingest your documents to build the Doc-AST representation.

Then configure your Gemini API key for LLM access.

Finally, query your documents using natural language and explore the reasoning trace.
