import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type {
  DocNodeDetail,
  DocNodeSummary,
  DocumentPreviewBlock,
  DocumentSummary,
  GraphNodePosition,
  IngestProgressEvent,
  ProjectSummary,
  ReasoningCompleteEvent,
  ReasoningErrorEvent,
  ReasoningStepEvent,
  RunPayload,
} from "./types";

export async function setProviderKey(apiKey: string): Promise<{ stored: boolean }> {
  return invoke("set_provider_key", { provider: "gemini", apiKey });
}

export async function ingestDocument(input: {
  filePath: string;
  mimeType: string;
  displayName?: string;
  projectId: string;
}): Promise<{ documentId: string; rootNodeId: string; nodeCount: number; sectionCount: number }> {
  return invoke("ingest_document", input);
}

export async function pickDocumentFiles(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
        {
          name: "Supported Documents",
          extensions: ["pdf", "pptx", "docx", "txt", "md", "csv", "png", "jpg", "jpeg", "webp", "tiff"],
        },
      ],
  });

  if (typeof selected === "string") return [selected];
  if (Array.isArray(selected)) return selected;
  return [];
}

export async function listDocuments(projectId: string): Promise<DocumentSummary[]> {
  const result = await invoke<{ documents: DocumentSummary[] }>("list_documents", { projectId });
  return result.documents;
}

export async function getTree(
  documentId: string,
  parentNodeId?: string,
  depth = 4,
): Promise<DocNodeSummary[]> {
  const result = await invoke<{ nodes: DocNodeSummary[] }>("get_tree", {
    documentId,
    parentNodeId,
    depth,
  });
  return result.nodes;
}

export async function getNode(nodeId: string): Promise<DocNodeDetail> {
  const result = await invoke<{ node: DocNodeDetail }>("get_node", { nodeId });
  return result.node;
}

export async function getDocumentPreview(documentId: string): Promise<DocumentPreviewBlock[]> {
  const result = await invoke<{ documentId: string; blocks: DocumentPreviewBlock[] }>(
    "get_document_preview",
    { documentId },
  );
  return result.blocks;
}

export async function runReasoningQuery(
  projectId: string,
  query: string,
  maxSteps = 6,
  focusDocumentId?: string | null,
): Promise<{ runId: string; status: string }> {
  return invoke("run_reasoning_query", { projectId, query, maxSteps, focusDocumentId });
}

export async function getRun(runId: string): Promise<RunPayload> {
  return invoke("get_run", { runId });
}

export async function exportMarkdown(documentId: string): Promise<{ filePath: string }> {
  return invoke("export_markdown", { documentId });
}

export async function deleteDocument(documentId: string): Promise<{ deleted: boolean }> {
  return invoke("delete_document", { documentId });
}

export async function getGraphLayout(documentId: string): Promise<GraphNodePosition[]> {
  const result = await invoke<{ documentId: string; positions: GraphNodePosition[] }>("get_graph_layout", {
    documentId,
  });
  return result.positions;
}

export async function saveGraphLayout(
  documentId: string,
  positions: GraphNodePosition[],
): Promise<{ saved: number }> {
  return invoke("save_graph_layout", { documentId, positions });
}

export function onIngestProgress(handler: (event: IngestProgressEvent) => void): Promise<UnlistenFn> {
  return listen("ingest/progress", (event) => handler(event.payload as IngestProgressEvent));
}

export function onReasoningStep(handler: (event: ReasoningStepEvent) => void): Promise<UnlistenFn> {
  return listen("reasoning/step", (event) => handler(event.payload as ReasoningStepEvent));
}

export function onReasoningComplete(
  handler: (event: ReasoningCompleteEvent) => void,
): Promise<UnlistenFn> {
  return listen("reasoning/complete", (event) =>
    handler(event.payload as ReasoningCompleteEvent),
  );
}

export function onReasoningError(handler: (event: ReasoningErrorEvent) => void): Promise<UnlistenFn> {
  return listen("reasoning/error", (event) => handler(event.payload as ReasoningErrorEvent));
}

// Project CRUD functions
export async function listProjects(): Promise<ProjectSummary[]> {
  const result = await invoke<{ projects: ProjectSummary[] }>("list_projects");
  return result.projects;
}

export async function createProject(name: string): Promise<ProjectSummary> {
  const result = await invoke<{ project: ProjectSummary }>("create_project", { name });
  return result.project;
}

export async function renameProject(projectId: string, name: string): Promise<ProjectSummary> {
  const result = await invoke<{ project: ProjectSummary }>("rename_project", { projectId, name });
  return result.project;
}

export async function deleteProject(projectId: string): Promise<{ deleted: boolean }> {
  return invoke("delete_project", { projectId });
}

export async function getProjectTree(
  projectId: string,
  depth = 4,
): Promise<DocNodeSummary[]> {
  const result = await invoke<{ nodes: DocNodeSummary[] }>("get_project_tree", {
    projectId,
    depth,
  });
  return result.nodes;
}
