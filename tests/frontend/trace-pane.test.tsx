import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  WorkspaceChromeProvider,
  type WorkspaceChromeContextValue,
} from "../../src/features/navigation/WorkspaceChromeContext";
import { TracePane } from "../../src/features/trace/TracePane";
import type { DocNodeSummary, ReasoningRun, ReasoningStep } from "../../src/lib/types";

const baseContext: WorkspaceChromeContextValue = {
  projects: [{ id: "project-1", name: "My Project", createdAt: "", updatedAt: "" }],
  activeProjectId: "project-1",
  setActiveProject: vi.fn(),
  createProject: vi.fn(async () => {}),
  documents: [],
  activeDocumentId: null,
  setActiveDocument: vi.fn(),
  uploading: false,
  fileStatuses: [],
  runIngestion: vi.fn(async () => {}),
  openSettings: vi.fn(),
};

const tree: DocNodeSummary[] = [
  {
    id: "node-1",
    documentId: "doc-1",
    parentId: null,
    nodeType: "section",
    title: "Overview",
    text: "",
    ordinalPath: "1",
    pageStart: 1,
    pageEnd: 1,
  },
];

const run: ReasoningRun = {
  id: "run-1",
  projectId: "project-1",
  documentId: "doc-1",
  query: "What is latency?",
  status: "completed",
  startedAt: new Date().toISOString(),
  endedAt: new Date().toISOString(),
  totalLatencyMs: 420,
  tokenUsageJson: { total: 1200 },
  costUsd: 0.012,
};

const steps: ReasoningStep[] = [
  {
    runId: "run-1",
    idx: 1,
    stepType: "scan_root",
    thought: "Scanning the root for latency terms.",
    action: "{\"target\":\"root\"}",
    observation: "Found candidate section.",
    nodeRefs: ["node-1"],
    confidence: 0.82,
    latencyMs: 120,
  },
];

function renderTrace(options: {
  documents?: WorkspaceChromeContextValue["documents"];
  queryText?: string;
  run?: ReasoningRun | null;
  steps?: ReasoningStep[];
}) {
  return render(
    <WorkspaceChromeProvider
      value={{
        ...baseContext,
        documents: options.documents ?? [],
      }}
    >
      <TracePane
        steps={options.steps ?? []}
        running={false}
        answer={null}
        tree={tree}
        run={options.run ?? null}
        queryText={options.queryText ?? ""}
        onQueryChange={() => {}}
        onSubmit={() => {}}
        onSelectNode={() => {}}
        onRerun={() => {}}
        onToggleView={() => {}}
      />
    </WorkspaceChromeProvider>,
  );
}

describe("TracePane", () => {
  it("shows no-documents empty state", () => {
    renderTrace({ documents: [] });
    expect(screen.getByText("Create a project and upload files to get started.")).toBeInTheDocument();
  });

  it("shows ask-first-query empty state when documents exist but no run data", () => {
    renderTrace({
      documents: [
        {
          id: "doc-1",
          projectId: "project-1",
          name: "Doc One",
          mime: "application/pdf",
          checksum: "abc",
          pages: 1,
          createdAt: new Date().toISOString(),
        },
      ],
    });

    expect(screen.getByText("Ask Your First Query")).toBeInTheDocument();
    expect(screen.queryByText("STEP 01")).not.toBeInTheDocument();
  });

  it("does not show timeline action buttons without real run data", () => {
    renderTrace({
      documents: [
        {
          id: "doc-1",
          projectId: "project-1",
          name: "Doc One",
          mime: "application/pdf",
          checksum: "abc",
          pages: 1,
          createdAt: new Date().toISOString(),
        },
      ],
      queryText: "typed but not run",
    });

    expect(screen.queryByRole("button", { name: "Re-run Trace" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Download Trace" })).not.toBeInTheDocument();
  });

  it("renders timeline and actions when run data exists", () => {
    renderTrace({
      documents: [
        {
          id: "doc-1",
          projectId: "project-1",
          name: "Doc One",
          mime: "application/pdf",
          checksum: "abc",
          pages: 1,
          createdAt: new Date().toISOString(),
        },
      ],
      queryText: "What is latency?",
      run,
      steps,
    });

    expect(screen.getByText("STEP 01")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Re-run Trace" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Download Trace" })).toBeInTheDocument();
  });
});
