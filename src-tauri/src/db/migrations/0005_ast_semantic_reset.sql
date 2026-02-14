-- 0005_ast_semantic_reset.sql
-- Destructive reset of ingest-derived content during semantic parser correction phase.
-- Keeps projects and app settings intact.

PRAGMA foreign_keys = ON;

DELETE FROM answers;
DELETE FROM reasoning_steps;
DELETE FROM reasoning_runs;
DELETE FROM graph_layouts;
DELETE FROM doc_nodes;
DELETE FROM documents;
