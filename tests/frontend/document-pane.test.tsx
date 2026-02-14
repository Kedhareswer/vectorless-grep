import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { DocumentPaneNew } from "../../src/features/document/DocumentPaneNew";
import { getDocumentPreview } from "../../src/lib/tauriApi";
import type { DocNodeDetail, DocumentPreviewBlock, DocumentSummary } from "../../src/lib/types";

vi.mock("../../src/lib/tauriApi", () => ({
  getDocumentPreview: vi.fn(),
}));

const mockedGetDocumentPreview = vi.mocked(getDocumentPreview);
const scrollIntoViewMock = vi.fn();

const docOne: DocumentSummary = {
  id: "doc-1",
  projectId: "project-default",
  name: "Doc One",
  mime: "text/plain",
  checksum: "checksum-1",
  pages: 1,
  createdAt: new Date().toISOString(),
};

const selectedNode: DocNodeDetail = {
  id: "sec-1",
  documentId: "doc-1",
  parentId: "root-1",
  nodeType: "section",
  title: "Introduction",
  text: "Intro text",
  ordinalPath: "1",
  pageStart: 1,
  pageEnd: 1,
  bboxJson: {},
  metadataJson: {},
};

const markdownBlock = [
  "![image](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO6p6acAAAAASUVORK5CYII=)",
  "[OpenAI](https://openai.com)",
  "",
  "- item one",
  "- item two",
].join("\n");

const markdownTableBlock = [
  "| col1 | col2 |",
  "| --- | --- |",
  "| A | B |",
].join("\n");

const previewBlocks: DocumentPreviewBlock[] = [
  {
    id: "root-1",
    documentId: "doc-1",
    parentId: null,
    nodeType: "document",
    title: "Doc One",
    text: "Root text",
    ordinalPath: "root",
  },
  {
    id: "sec-1",
    documentId: "doc-1",
    parentId: "root-1",
    nodeType: "section",
    title: "Introduction",
    text: markdownBlock,
    ordinalPath: "1",
  },
  {
    id: "para-1",
    documentId: "doc-1",
    parentId: "sec-1",
    nodeType: "paragraph",
    title: "",
    text: "Paragraph text",
    ordinalPath: "1.1",
  },
  {
    id: "para-2",
    documentId: "doc-1",
    parentId: "sec-1",
    nodeType: "paragraph",
    title: "",
    text: markdownTableBlock,
    ordinalPath: "1.2",
  },
];

beforeEach(() => {
  mockedGetDocumentPreview.mockReset();
  scrollIntoViewMock.mockReset();
  Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: scrollIntoViewMock,
  });
});

describe("DocumentPaneNew", () => {
  it("loads full reader flow and renders headings and markdown content", async () => {
    mockedGetDocumentPreview.mockResolvedValue(previewBlocks);

    const { container } = render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    expect(await screen.findByText("Doc One")).toBeInTheDocument();
    expect(screen.getByText("Introduction")).toBeInTheDocument();
    expect(screen.getByText("Paragraph text")).toBeInTheDocument();
    expect(container.querySelector(".doc-reader-flow")).toBeInTheDocument();
    expect(container.querySelector(".preview-block")).not.toBeInTheDocument();
  });

  it("renders markdown image syntax as img element", async () => {
    mockedGetDocumentPreview.mockResolvedValue(previewBlocks);

    const { container } = render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await screen.findByText("Introduction");
    const img = container.querySelector('img[alt="image"]');
    expect(img).toBeInTheDocument();
    expect(img?.getAttribute("src") ?? "").toContain("data:image/png;base64");
    expect(screen.queryByText(/!\[image\]/i)).not.toBeInTheDocument();
  });

  it("renders markdown links, tables, and lists", async () => {
    mockedGetDocumentPreview.mockResolvedValue(previewBlocks);

    const { container } = render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await screen.findByText("Introduction");
    const link = screen.getByRole("link", { name: "OpenAI" });
    expect(link).toHaveAttribute("href", "https://openai.com");
    expect(link).toHaveAttribute("target", "_blank");

    const listItem = screen.getByText("item one");
    expect(listItem.closest("li")).toBeInTheDocument();

    const table = container.querySelector(".doc-flow-markdown table");
    expect(table).toBeInTheDocument();
    expect(screen.getByText("col1")).toBeInTheDocument();
    expect(screen.getByText("A")).toBeInTheDocument();
  });

  it("renders tab-delimited spreadsheet text as a table", async () => {
    mockedGetDocumentPreview.mockResolvedValue([
      {
        id: "root-1",
        documentId: "doc-1",
        parentId: null,
        nodeType: "document",
        title: "SheetDoc",
        text: "",
        ordinalPath: "root",
      },
      {
        id: "tsv-1",
        documentId: "doc-1",
        parentId: "root-1",
        nodeType: "paragraph",
        title: "",
        text: "Name\tScore\nAlice\t95\nBob\t88",
        ordinalPath: "1.1",
      },
    ]);

    const { container } = render(
      <DocumentPaneNew
        document={docOne}
        node={{ ...selectedNode, id: "tsv-1", nodeType: "paragraph", title: "", text: "Name\tScore" }}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await screen.findByText("SheetDoc");
    const table = container.querySelector(".doc-flow-markdown table");
    expect(table).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("applies active highlight class to selected flow block", async () => {
    mockedGetDocumentPreview.mockResolvedValue(previewBlocks);

    const { container } = render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await screen.findByText("Introduction");
    const selected = container.querySelector('[data-node-id="sec-1"]');
    expect(selected).toHaveClass("doc-flow-block", "doc-flow-active");
  });

  it("auto-scrolls when selected node changes", async () => {
    mockedGetDocumentPreview.mockResolvedValue(previewBlocks);

    const { rerender } = render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await screen.findByText("Introduction");
    const initialCalls = scrollIntoViewMock.mock.calls.length;

    rerender(
      <DocumentPaneNew
        document={docOne}
        node={{ ...selectedNode, id: "para-1", ordinalPath: "1.1", nodeType: "paragraph" }}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await waitFor(() => {
      expect(scrollIntoViewMock.mock.calls.length).toBeGreaterThan(initialCalls);
    });
  });

  it("switches preview document for cross-document selected node", async () => {
    mockedGetDocumentPreview
      .mockResolvedValueOnce(previewBlocks)
      .mockResolvedValueOnce([
        {
          id: "root-2",
          documentId: "doc-2",
          parentId: null,
          nodeType: "document",
          title: "Doc Two",
          text: "",
          ordinalPath: "root",
        },
      ]);

    const { rerender } = render(
      <DocumentPaneNew
        document={docOne}
        node={null}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await waitFor(() => {
      expect(mockedGetDocumentPreview).toHaveBeenCalledWith("doc-1");
    });

    rerender(
      <DocumentPaneNew
        document={docOne}
        node={{ ...selectedNode, id: "root-2", documentId: "doc-2", parentId: null }}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    await waitFor(() => {
      expect(mockedGetDocumentPreview).toHaveBeenCalledWith("doc-2");
    });
  });

  it("renders error state when preview API rejects", async () => {
    mockedGetDocumentPreview.mockRejectedValue(new Error("boom"));

    render(
      <DocumentPaneNew
        document={docOne}
        node={selectedNode}
        confidence={0.75}
        onSelectNode={() => {}}
        trace={[]}
        tree={[]}
        queryText=""
      />,
    );

    expect(await screen.findByText(/Preview failed to load/i)).toBeInTheDocument();
  });
});
