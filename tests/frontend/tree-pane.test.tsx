import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  WorkspaceChromeProvider,
  type WorkspaceChromeContextValue,
} from "../../src/features/navigation/WorkspaceChromeContext";
import { TreePane } from "../../src/features/tree/TreePane";
import type { DocNodeSummary } from "../../src/lib/types";

const contextValue: WorkspaceChromeContextValue = {
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

const nodes: DocNodeSummary[] = [
  {
    id: "root-doc-1",
    documentId: "doc-1",
    parentId: null,
    nodeType: "document",
    title: "Annual_Report_2023.pdf",
    text: "",
    ordinalPath: "root",
    pageStart: 1,
    pageEnd: 20,
  },
  {
    id: "section-1",
    documentId: "doc-1",
    parentId: "root-doc-1",
    nodeType: "section",
    title: "1. Introduction",
    text: "",
    ordinalPath: "1",
    pageStart: 1,
    pageEnd: 2,
  },
  {
    id: "table-1",
    documentId: "doc-1",
    parentId: "section-1",
    nodeType: "table",
    title: "Table 2.1: Q4 Earnings",
    text: "",
    ordinalPath: "1.1",
    pageStart: 2,
    pageEnd: 2,
  },
  {
    id: "root-doc-2",
    documentId: "doc-2",
    parentId: null,
    nodeType: "document",
    title: "Final_Presentation.pptx",
    text: "",
    ordinalPath: "root",
    pageStart: 1,
    pageEnd: 12,
  },
  {
    id: "section-2",
    documentId: "doc-2",
    parentId: "root-doc-2",
    nodeType: "section",
    title: "Slide 9",
    text: "",
    ordinalPath: "1",
    pageStart: 9,
    pageEnd: 9,
  },
  {
    id: "figure-1",
    documentId: "doc-2",
    parentId: "section-2",
    nodeType: "figure",
    title: "Figure 2",
    text: "![chart](data:image/png;base64,abc)",
    ordinalPath: "1.1",
    pageStart: 9,
    pageEnd: 9,
  },
];

function renderTree(
  activeNodeId: string | null = null,
  onDeleteDocument?: (documentId: string) => void,
) {
  const onSelect = vi.fn();
  render(
    <WorkspaceChromeProvider value={contextValue}>
      <TreePane
        nodes={nodes}
        activeNodeId={activeNodeId}
        onSelect={onSelect}
        onDeleteDocument={onDeleteDocument}
      />
    </WorkspaceChromeProvider>,
  );
  return { onSelect };
}

describe("TreePane", () => {
  it("shows multiple document roots in the same project explorer", () => {
    renderTree();

    expect(screen.getByText("Annual_Report_2023.pdf")).toBeInTheDocument();
    expect(screen.getByText("Final_Presentation.pptx")).toBeInTheDocument();
  });

  it("does not hide root document nodes", () => {
    renderTree();

    expect(screen.getByText("Annual_Report_2023.pdf")).toBeInTheDocument();
    expect(screen.getByText("Final_Presentation.pptx")).toBeInTheDocument();
  });

  it("renders typed nodes and allows selecting descendants", () => {
    const { onSelect } = renderTree();

    const tableNode = screen.getByText("Table 2.1: Q4 Earnings");
    const figureNode = screen.getByText("Figure 2");
    expect(tableNode).toBeInTheDocument();
    expect(figureNode).toBeInTheDocument();

    fireEvent.click(tableNode.closest("button")!);
    expect(onSelect).toHaveBeenCalledWith("table-1");
  });

  it("highlights the active selected node", () => {
    renderTree("figure-1");

    const activeNode = screen.getByText("Figure 2");
    expect(activeNode.closest(".tree-row")).toHaveClass("active");
  });

  it("shows icon-only delete buttons for document roots", () => {
    const onDeleteDocument = vi.fn();
    renderTree(null, onDeleteDocument);

    const deleteButton = screen.getByRole("button", { name: "Delete Annual_Report_2023.pdf" });
    expect(deleteButton).toBeInTheDocument();
    expect(screen.queryByText("DEL")).not.toBeInTheDocument();

    fireEvent.click(deleteButton);
    expect(onDeleteDocument).toHaveBeenCalledWith("doc-1");
  });
});
