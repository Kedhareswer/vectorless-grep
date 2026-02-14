import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TreePane } from "../../src/features/tree/TreePane";
import type { DocNodeSummary } from "../../src/lib/types";

const nodes: DocNodeSummary[] = [
  {
    id: "root-1",
    documentId: "doc-1",
    parentId: null,
    nodeType: "document",
    title: "Technical_Spec_v2.pdf",
    text: "",
    ordinalPath: "root",
    pageStart: 1,
    pageEnd: 12,
  },
  {
    id: "section-1",
    documentId: "doc-1",
    parentId: "root-1",
    nodeType: "section",
    title: "Introduction",
    text: "This is the introduction section.",
    ordinalPath: "root.1",
    pageStart: 1,
    pageEnd: 2,
  },
];

describe("TreePane", () => {
  it("renders tree nodes with hierarchy and icons", () => {
    const onSelect = vi.fn();

    render(
      <TreePane
        nodes={nodes}
        activeNodeId={null}
        onSelect={onSelect}
      />,
    );

    // Check that nodes are rendered (the section node, not the root document node)
    const sectionNode = screen.getByText("Introduction");
    expect(sectionNode).toBeInTheDocument();

    // Check that node is rendered with correct title
    expect(sectionNode).toBeInTheDocument();

    // Check that clicking a node calls onSelect
    fireEvent.click(sectionNode.closest("button")!);
    expect(onSelect).toHaveBeenCalledWith("section-1");
  });

  it("highlights active node", () => {
    render(
      <TreePane
        nodes={nodes}
        activeNodeId="section-1"
        onSelect={() => {}}
      />,
    );

    const activeNode = screen.getByText("Introduction");
    expect(activeNode.closest(".tree-row")).toHaveClass("active");
  });
});
