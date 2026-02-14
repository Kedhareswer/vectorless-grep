-- no-transaction
PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS documents_old (
  id TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS reasoning_runs_old (
  id TEXT PRIMARY KEY NOT NULL
);

DROP TRIGGER IF EXISTS doc_nodes_ai;
DROP TRIGGER IF EXISTS doc_nodes_ad;
DROP TRIGGER IF EXISTS doc_nodes_au;

DROP TABLE IF EXISTS doc_nodes_fts;
DROP TABLE IF EXISTS graph_layouts;
DROP TABLE IF EXISTS answers;
DROP TABLE IF EXISTS reasoning_steps;
DROP TABLE IF EXISTS reasoning_runs;
DROP TABLE IF EXISTS doc_nodes;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS app_settings;
DROP TABLE IF EXISTS documents_old;
DROP TABLE IF EXISTS reasoning_runs_old;

CREATE TABLE projects (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO projects (id, name)
VALUES ('project-default', 'My Project');

CREATE TABLE documents (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  mime TEXT NOT NULL,
  checksum TEXT NOT NULL,
  pages INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
  UNIQUE(project_id, checksum)
);

CREATE TABLE doc_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  document_id TEXT NOT NULL,
  parent_id TEXT,
  node_type TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  text TEXT NOT NULL DEFAULT '',
  page_start INTEGER,
  page_end INTEGER,
  bbox_json TEXT NOT NULL DEFAULT '{}',
  metadata_json TEXT NOT NULL DEFAULT '{}',
  ordinal_path TEXT NOT NULL,
  FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE CASCADE,
  FOREIGN KEY(parent_id) REFERENCES doc_nodes(id) ON DELETE CASCADE
);

CREATE TABLE reasoning_runs (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  document_id TEXT,
  query TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  ended_at TEXT,
  total_latency_ms INTEGER,
  token_usage_json TEXT NOT NULL DEFAULT '{}',
  cost_usd REAL NOT NULL DEFAULT 0.0,
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE SET NULL
);

CREATE TABLE reasoning_steps (
  run_id TEXT NOT NULL,
  idx INTEGER NOT NULL,
  step_type TEXT NOT NULL,
  thought TEXT NOT NULL,
  action TEXT NOT NULL,
  observation TEXT NOT NULL,
  node_refs_json TEXT NOT NULL DEFAULT '[]',
  confidence REAL NOT NULL DEFAULT 0.0,
  latency_ms INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (run_id, idx),
  FOREIGN KEY(run_id) REFERENCES reasoning_runs(id) ON DELETE CASCADE
);

CREATE TABLE answers (
  run_id TEXT PRIMARY KEY NOT NULL,
  answer_markdown TEXT NOT NULL,
  citations_json TEXT NOT NULL DEFAULT '[]',
  confidence REAL NOT NULL DEFAULT 0.0,
  grounded INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(run_id) REFERENCES reasoning_runs(id) ON DELETE CASCADE
);

CREATE TABLE graph_layouts (
  document_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  x REAL NOT NULL,
  y REAL NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  PRIMARY KEY (document_id, node_id),
  FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE CASCADE,
  FOREIGN KEY(node_id) REFERENCES doc_nodes(id) ON DELETE CASCADE
);

CREATE TABLE app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value_json TEXT NOT NULL
);

CREATE INDEX idx_documents_project ON documents(project_id);
CREATE INDEX idx_doc_nodes_document ON doc_nodes(document_id);
CREATE INDEX idx_doc_nodes_parent ON doc_nodes(parent_id);
CREATE INDEX idx_doc_nodes_type ON doc_nodes(node_type);
CREATE INDEX idx_doc_nodes_ordinal ON doc_nodes(ordinal_path);
CREATE INDEX idx_runs_document ON reasoning_runs(document_id);
CREATE INDEX idx_runs_project ON reasoning_runs(project_id);
CREATE INDEX idx_graph_layouts_document ON graph_layouts(document_id);

CREATE VIRTUAL TABLE doc_nodes_fts
USING fts5(node_id UNINDEXED, document_id UNINDEXED, title, text, tokenize='unicode61');

CREATE TRIGGER doc_nodes_ai AFTER INSERT ON doc_nodes BEGIN
  INSERT INTO doc_nodes_fts (node_id, document_id, title, text)
  VALUES (new.id, new.document_id, new.title, new.text);
END;

CREATE TRIGGER doc_nodes_ad AFTER DELETE ON doc_nodes BEGIN
  DELETE FROM doc_nodes_fts WHERE node_id = old.id;
END;

CREATE TRIGGER doc_nodes_au AFTER UPDATE ON doc_nodes BEGIN
  DELETE FROM doc_nodes_fts WHERE node_id = old.id;
  INSERT INTO doc_nodes_fts (node_id, document_id, title, text)
  VALUES (new.id, new.document_id, new.title, new.text);
END;

PRAGMA foreign_keys = ON;
