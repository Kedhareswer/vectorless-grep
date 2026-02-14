PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO projects (id, name)
VALUES ('project-default', 'My Project')
ON CONFLICT(id) DO NOTHING;

ALTER TABLE documents RENAME TO documents_old;

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

INSERT INTO documents (id, project_id, name, mime, checksum, pages, created_at)
SELECT id, 'project-default', name, mime, checksum, pages, created_at
FROM documents_old;

DROP TABLE documents_old;

DROP INDEX IF EXISTS idx_runs_document;

ALTER TABLE reasoning_runs RENAME TO reasoning_runs_old;

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

INSERT INTO reasoning_runs (
  id,
  project_id,
  document_id,
  query,
  status,
  started_at,
  ended_at,
  total_latency_ms,
  token_usage_json,
  cost_usd
)
SELECT
  rr.id,
  COALESCE(d.project_id, 'project-default'),
  rr.document_id,
  rr.query,
  rr.status,
  rr.started_at,
  rr.ended_at,
  rr.total_latency_ms,
  rr.token_usage_json,
  rr.cost_usd
FROM reasoning_runs_old rr
LEFT JOIN documents d ON d.id = rr.document_id;

DROP TABLE reasoning_runs_old;

CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id);
CREATE INDEX IF NOT EXISTS idx_runs_document ON reasoning_runs(document_id);
CREATE INDEX IF NOT EXISTS idx_runs_project ON reasoning_runs(project_id);

PRAGMA foreign_keys = ON;
