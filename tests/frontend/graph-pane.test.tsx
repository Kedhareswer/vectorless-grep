import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { MouseEvent, ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GraphPane } from "../../src/features/graph/GraphPane";
import type { DocNodeSummary } from "../../src/lib/types";

const flowMocks = vi.hoisted(() => ({
  fitView: vi.fn(),
  zoomIn: vi.fn(),
  zoomOut: vi.fn(),
  getNodes: vi.fn(() => []),
}));

const apiMocks = vi.hoisted(() => ({
  getGraphLayout: vi.fn(),
  saveGraphLayout: vi.fn(),
}));

vi.mock("@xyflow/react", () => ({
  ReactFlowProvider: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  ReactFlow: ({
    nodes,
    edges,
    onNodeClick,
    nodesDraggable,
  }: {
    nodes: Array<{ id: string; data: { label: string } }>;
    edges: unknown[];
    onNodeClick?: (event: MouseEvent, node: { id: string }) => void;
    nodesDraggable?: boolean;
  }) => (
    <div data-testid="react-flow" data-nodes-draggable={String(Boolean(nodesDraggable))}>
      <span data-testid="edge-count">{edges.length}</span>
      {nodes.map((node) => (
        <button
          key={node.id}
          type="button"
          data-testid={`flow-node-${node.id}`}
          onClick={(event) => onNodeClick?.(event, node)}
        >
          {node.data.label}
        </button>
      ))}
    </div>
  ),
  Background: () => null,
  BackgroundVariant: { Dots: "dots" },
  Handle: () => null,
  Position: { Top: "top", Bottom: "bottom" },
  useReactFlow: () => flowMocks,
}));

vi.mock("../../src/lib/tauriApi", () => ({
  getGraphLayout: apiMocks.getGraphLayout,
  saveGraphLayout: apiMocks.saveGraphLayout,
}));

const nodes: DocNodeSummary[] = [
  {
    id: "doc1-root",
    documentId: "doc-1",
    parentId: null,
    nodeType: "document",
    title: "Doc One",
    text: "",
    ordinalPath: "root",
    pageStart: 1,
    pageEnd: 1,
  },
  {
    id: "doc1-sec-1",
    documentId: "doc-1",
    parentId: "doc1-root",
    nodeType: "section",
    title: "Slide 1",
    text: "",
    ordinalPath: "1",
    pageStart: 1,
    pageEnd: 1,
  },
  {
    id: "doc2-root",
    documentId: "doc-2",
    parentId: null,
    nodeType: "document",
    title: "Doc Two",
    text: "",
    ordinalPath: "root",
    pageStart: 1,
    pageEnd: 1,
  },
];

describe("GraphPane", () => {
  beforeEach(() => {
    window.localStorage.clear();
    apiMocks.getGraphLayout.mockReset();
    apiMocks.saveGraphLayout.mockReset();
    apiMocks.getGraphLayout.mockResolvedValue([]);
    apiMocks.saveGraphLayout.mockResolvedValue({ saved: 0 });
    flowMocks.fitView.mockReset();
    flowMocks.zoomIn.mockReset();
    flowMocks.zoomOut.mockReset();
    flowMocks.getNodes.mockReset();
    flowMocks.getNodes.mockReturnValue([]);
  });

  it("shows active-document hierarchy by default and switches to whole project scope", async () => {
    render(
      <GraphPane
        documentId="doc-1"
        nodes={nodes}
        activeNodeId={null}
        onSelect={() => {}}
        onToggleView={() => {}}
      />,
    );

    await screen.findByTestId("flow-node-doc1-root");
    expect(screen.queryByTestId("flow-node-doc2-root")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Whole Project" }));
    expect(await screen.findByTestId("flow-node-doc2-root")).toBeInTheDocument();
  });

  it("keeps hierarchy auto-layout non-draggable and cluster mode draggable", async () => {
    render(
      <GraphPane
        documentId="doc-1"
        nodes={nodes}
        activeNodeId={null}
        onSelect={() => {}}
        onToggleView={() => {}}
      />,
    );

    const flow = await screen.findByTestId("react-flow");
    expect(flow).toHaveAttribute("data-nodes-draggable", "false");

    fireEvent.click(screen.getByRole("button", { name: "Global Cluster" }));
    expect(screen.getByTestId("react-flow")).toHaveAttribute("data-nodes-draggable", "true");
  });

  it("clears stale saved layouts once when entering hierarchy mode", async () => {
    render(
      <GraphPane
        documentId="doc-1"
        nodes={nodes}
        activeNodeId={null}
        onSelect={() => {}}
        onToggleView={() => {}}
      />,
    );

    await waitFor(() => {
      expect(apiMocks.saveGraphLayout).toHaveBeenCalledWith("doc-1", []);
    });
  });
});
