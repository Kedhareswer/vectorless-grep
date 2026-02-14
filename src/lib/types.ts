export type Provider = "gemini";

export type NodeType =
  | "document"
  | "section"
  | "subsection"
  | "paragraph"
  | "claim"
  | "table"
  | "figure"
  | "equation"
  | "caption"
  | "reference"
  | "unknown";

export interface ProjectSummary {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentSummary {
  id: string;
  projectId: string;
  name: string;
  mime: string;
  checksum: string;
  pages: number;
  createdAt: string;
}

export interface DocNodeSummary {
  id: string;
  documentId: string;
  parentId: string | null;
  nodeType: NodeType;
  title: string;
  text: string;
  ordinalPath: string;
  pageStart: number | null;
  pageEnd: number | null;
}

export interface DocNodeDetail extends DocNodeSummary {
  bboxJson: Record<string, unknown>;
  metadataJson: Record<string, unknown>;
}

export interface DocumentPreviewBlock {
  id: string;
  documentId: string;
  parentId: string | null;
  nodeType: NodeType;
  title: string;
  text: string;
  ordinalPath: string;
}

export interface ReasoningRun {
  id: string;
  projectId: string;
  documentId: string | null;
  query: string;
  status: "running" | "completed" | "failed";
  startedAt: string;
  endedAt: string | null;
  totalLatencyMs: number | null;
  tokenUsageJson: Record<string, unknown>;
  costUsd: number;
}

export interface ReasoningStep {
  runId: string;
  idx: number;
  stepType: string;
  thought: string;
  action: string;
  observation: string;
  nodeRefs: string[];
  confidence: number;
  latencyMs: number;
}

export interface AnswerRecord {
  runId: string;
  answerMarkdown: string;
  citations: string[];
  confidence: number;
  grounded: boolean;
}

export interface RunPayload {
  run: ReasoningRun;
  steps: ReasoningStep[];
  answer?: AnswerRecord;
}

export interface IngestProgressEvent {
  jobId: string;
  stage: string;
  percent: number;
  message: string;
}

export interface ReasoningStepEvent {
  runId: string;
  stepIndex: number;
  stepType: string;
  thought: string;
  action: string;
  observation: string;
  nodeRefs: string[];
  latencyMs: number;
  confidence: number;
}

export interface GraphNodePosition {
  nodeId: string;
  x: number;
  y: number;
}

export interface ReasoningCompleteEvent {
  runId: string;
  answerId: string;
  finalConfidence: number;
  totalLatencyMs: number;
  tokenUsage: Record<string, unknown>;
  costUsd: number;
}

export interface ReasoningErrorEvent {
  runId: string;
  code: string;
  message: string;
  retryable: boolean;
}
