ALTER TABLE reasoning_runs
ADD COLUMN phase TEXT NOT NULL DEFAULT 'planning';

ALTER TABLE reasoning_runs
ADD COLUMN quality_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE reasoning_runs
ADD COLUMN planner_trace_json TEXT NOT NULL DEFAULT '[]';
