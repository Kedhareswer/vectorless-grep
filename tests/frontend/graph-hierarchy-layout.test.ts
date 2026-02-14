import { describe, expect, it } from "vitest";

import { buildHierarchyLayout } from "../../src/features/graph/hierarchyLayout";
import type { DocNodeSummary } from "../../src/lib/types";

function node(input: Partial<DocNodeSummary> & Pick<DocNodeSummary, "id" | "documentId" | "ordinalPath">): DocNodeSummary {
  return {
    id: input.id,
    documentId: input.documentId,
    parentId: input.parentId ?? null,
    nodeType: input.nodeType ?? "paragraph",
    title: input.title ?? input.id,
    text: input.text ?? "",
    ordinalPath: input.ordinalPath,
    pageStart: input.pageStart ?? null,
    pageEnd: input.pageEnd ?? null,
  };
}

describe("buildHierarchyLayout", () => {
  it("places children below their parent in strict top-down order", () => {
    const nodes: DocNodeSummary[] = [
      node({ id: "root", documentId: "doc-1", nodeType: "document", ordinalPath: "root" }),
      node({ id: "sec-1", documentId: "doc-1", parentId: "root", nodeType: "section", ordinalPath: "1" }),
      node({ id: "para-1", documentId: "doc-1", parentId: "sec-1", nodeType: "paragraph", ordinalPath: "1.1" }),
    ];

    const layout = buildHierarchyLayout(nodes);
    const rootPos = layout.positions.get("root");
    const secPos = layout.positions.get("sec-1");
    const paraPos = layout.positions.get("para-1");

    expect(rootPos).toBeDefined();
    expect(secPos).toBeDefined();
    expect(paraPos).toBeDefined();
    expect((secPos?.y ?? 0) > (rootPos?.y ?? 0)).toBe(true);
    expect((paraPos?.y ?? 0) > (secPos?.y ?? 0)).toBe(true);
  });

  it("orders numeric ordinals naturally (1,2,7,10)", () => {
    const nodes: DocNodeSummary[] = [
      node({ id: "root", documentId: "doc-1", nodeType: "document", ordinalPath: "root" }),
      node({ id: "slide-10", documentId: "doc-1", parentId: "root", nodeType: "section", title: "Slide 10", ordinalPath: "10" }),
      node({ id: "slide-2", documentId: "doc-1", parentId: "root", nodeType: "section", title: "Slide 2", ordinalPath: "2" }),
      node({ id: "slide-7", documentId: "doc-1", parentId: "root", nodeType: "section", title: "Slide 7", ordinalPath: "7" }),
      node({ id: "slide-1", documentId: "doc-1", parentId: "root", nodeType: "section", title: "Slide 1", ordinalPath: "1" }),
    ];

    const layout = buildHierarchyLayout(nodes);
    const sectionOrder = layout.orderedNodes
      .filter((item) => item.parentId === "root")
      .map((item) => item.title);

    expect(sectionOrder).toEqual(["Slide 1", "Slide 2", "Slide 7", "Slide 10"]);
  });

  it("includes multiple document roots in one project layout", () => {
    const nodes: DocNodeSummary[] = [
      node({ id: "doc-root-1", documentId: "doc-1", nodeType: "document", title: "Doc A", ordinalPath: "root" }),
      node({ id: "doc-root-2", documentId: "doc-2", nodeType: "document", title: "Doc B", ordinalPath: "root" }),
      node({ id: "doc2-sec-1", documentId: "doc-2", parentId: "doc-root-2", nodeType: "section", ordinalPath: "1" }),
    ];

    const layout = buildHierarchyLayout(nodes);
    const rootIds = layout.orderedNodes.filter((item) => item.parentId === null).map((item) => item.id);

    expect(rootIds).toEqual(["doc-root-1", "doc-root-2"]);
    expect(layout.edges.some((edge) => edge.source === "doc-root-2" && edge.target === "doc2-sec-1")).toBe(true);
  });
});
