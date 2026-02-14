import { useEffect, useMemo, useRef, useState } from "react";
import ReactMarkdown, { defaultUrlTransform } from "react-markdown";
import remarkGfm from "remark-gfm";

import { getDocumentPreview } from "../../lib/tauriApi";
import { compareOrdinalPath } from "../../lib/formatters";
import type {
  DocNodeDetail,
  DocNodeSummary,
  DocumentPreviewBlock,
  DocumentSummary,
  ReasoningStep,
} from "../../lib/types";

interface DocumentPaneProps {
  document: DocumentSummary | null;
  node: DocNodeDetail | null;
  confidence: number;
  onSelectNode: (nodeId: string) => void;
  trace: ReasoningStep[];
  tree: DocNodeSummary[];
  queryText: string;
}

interface DocumentRenderBlock extends DocumentPreviewBlock {
  headingLevel: 0 | 1 | 2 | 3;
  anchorId: string;
}

interface TsvTableModel {
  header: string[];
  rows: string[][];
}

function labelForNodeType(nodeType: DocumentPreviewBlock["nodeType"]): string {
  switch (nodeType) {
    case "document":
      return "Document";
    case "section":
      return "Section";
    case "subsection":
      return "Subsection";
    case "paragraph":
      return "Paragraph";
    default:
      return "Block";
  }
}

function headingLevelForNodeType(nodeType: DocumentPreviewBlock["nodeType"]): 0 | 1 | 2 | 3 {
  switch (nodeType) {
    case "document":
      return 1;
    case "section":
      return 2;
    case "subsection":
      return 3;
    default:
      return 0;
  }
}

function parseTsvTable(text: string): TsvTableModel | null {
  const lines = text
    .split(/\r?\n/g)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

  if (lines.length < 2) return null;

  const tabbedLines = lines.filter((line) => line.includes("\t"));
  if (tabbedLines.length < 2) return null;

  if (tabbedLines.length / lines.length < 0.8) return null;

  const parsedRows = lines.map((line) => line.split("\t").map((cell) => cell.trim()));
  const colCount = Math.max(...parsedRows.map((row) => row.length));
  if (colCount < 2) return null;

  const normalizedRows = parsedRows.map((row) => {
    if (row.length >= colCount) return row;
    return [...row, ...Array(colCount - row.length).fill("")];
  });

  const [header, ...rows] = normalizedRows;
  if (!header || rows.length === 0) return null;

  return { header, rows };
}

function markdownUrlTransform(url: string): string {
  if (url.startsWith("data:image/")) {
    return url;
  }
  return defaultUrlTransform(url);
}

function toRenderBlocks(blocks: DocumentPreviewBlock[]): DocumentRenderBlock[] {
  return [...blocks]
    .sort((a, b) => compareOrdinalPath(a.ordinalPath, b.ordinalPath))
    .filter((block) => block.title.trim().length > 0 || block.text.trim().length > 0)
    .map((block) => ({
      ...block,
      headingLevel: headingLevelForNodeType(block.nodeType),
      anchorId: `doc-flow-${block.id}`,
    }));
}

export function DocumentPaneNew({
  document,
  node,
  confidence,
  onSelectNode,
  trace,
  tree,
}: DocumentPaneProps) {
  const contentRef = useRef<HTMLDivElement>(null);
  const [previewDocumentId, setPreviewDocumentId] = useState<string | null>(null);
  const [previewBlocks, setPreviewBlocks] = useState<DocumentPreviewBlock[]>([]);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  const targetDocumentId = node?.documentId ?? document?.id ?? null;

  useEffect(() => {
    if (!targetDocumentId) {
      setPreviewDocumentId(null);
      setPreviewBlocks([]);
      setPreviewError(null);
      setPreviewLoading(false);
      return;
    }

    let cancelled = false;
    setPreviewDocumentId(targetDocumentId);
    setPreviewLoading(true);
    setPreviewError(null);

    void getDocumentPreview(targetDocumentId)
      .then((blocks) => {
        if (cancelled) return;
        setPreviewBlocks(blocks);
      })
      .catch((error) => {
        if (cancelled) return;
        setPreviewBlocks([]);
        setPreviewError(String(error));
      })
      .finally(() => {
        if (cancelled) return;
        setPreviewLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [targetDocumentId]);

  const renderBlocks = useMemo(() => toRenderBlocks(previewBlocks), [previewBlocks]);
  const selectedNodeIsVisible = node ? renderBlocks.some((block) => block.id === node.id) : false;

  useEffect(() => {
    if (!node?.id || !contentRef.current || previewDocumentId !== node.documentId) return;
    const element = contentRef.current.querySelector<HTMLElement>(`[data-node-id="${node.id}"]`);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }, [node?.id, node?.documentId, previewDocumentId, renderBlocks]);

  const relevantSteps = node ? trace.filter((step) => step.nodeRefs.includes(node.id)) : [];
  const parent = node?.parentId ? tree.find((item) => item.id === node.parentId) ?? null : null;
  const childCount = parent ? tree.filter((item) => item.parentId === parent.id).length : 0;

  return (
    <section className="pane document-pane">
      <header className="pane-header">
        <div className="pane-heading-group">
          <h2>CONTEXT</h2>
          <p>{document?.name ?? "No document loaded"}</p>
        </div>
      </header>

      <div className="document-content" ref={contentRef}>
        <article className="preview-reader">
          <div className="reader-rule" />

          {!targetDocumentId && (
            <div className="preview-state">Select a node from the tree to load a document preview.</div>
          )}

          {targetDocumentId && previewLoading && <div className="preview-state">Loading full preview...</div>}

          {targetDocumentId && !previewLoading && previewError && (
            <div className="preview-state">Preview failed to load: {previewError}</div>
          )}

          {targetDocumentId && !previewLoading && !previewError && renderBlocks.length === 0 && (
            <div className="preview-state">No preview blocks available for this document.</div>
          )}

          {targetDocumentId && !previewLoading && !previewError && renderBlocks.length > 0 && (
            <section className="doc-reader-flow" aria-label="Document preview flow">
              {renderBlocks.map((block) => {
                const isActive = node?.id === block.id;
                const bodyText = block.text.trim();
                const tsvTable = bodyText ? parseTsvTable(bodyText) : null;

                return (
                  <section
                    key={block.id}
                    id={block.anchorId}
                    data-node-id={block.id}
                    className={`doc-flow-block kind-${block.nodeType}${isActive ? " doc-flow-active" : ""}`}
                  >
                    <div className="doc-flow-meta">
                      <span>{labelForNodeType(block.nodeType)}</span>
                      <span>{block.ordinalPath}</span>
                    </div>

                    {block.headingLevel === 1 && block.title.trim().length > 0 && (
                      <h1 className="doc-flow-heading level-1">{block.title}</h1>
                    )}
                    {block.headingLevel === 2 && block.title.trim().length > 0 && (
                      <h2 className="doc-flow-heading level-2">{block.title}</h2>
                    )}
                    {block.headingLevel === 3 && block.title.trim().length > 0 && (
                      <h3 className="doc-flow-heading level-3">{block.title}</h3>
                    )}

                    {block.headingLevel === 0 && block.title.trim().length > 0 && (
                      <p className="doc-flow-inline-title">{block.title}</p>
                    )}

                    {tsvTable ? (
                      <div className="doc-flow-markdown">
                        <table>
                          <thead>
                            <tr>
                              {tsvTable.header.map((cell, index) => (
                                <th key={`th-${index}`}>{cell}</th>
                              ))}
                            </tr>
                          </thead>
                          <tbody>
                            {tsvTable.rows.map((row, rowIndex) => (
                              <tr key={`row-${rowIndex}`}>
                                {row.map((cell, cellIndex) => (
                                  <td key={`cell-${rowIndex}-${cellIndex}`}>{cell}</td>
                                ))}
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    ) : (
                      bodyText.length > 0 && (
                        <div className="doc-flow-markdown">
                          <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            urlTransform={markdownUrlTransform}
                            components={{
                              a: ({ node: _node, ...props }) => (
                                <a {...props} target="_blank" rel="noreferrer noopener" />
                              ),
                              img: ({ node: _node, alt, ...props }) => (
                                <img {...props} alt={alt ?? "Document image"} loading="lazy" />
                              ),
                            }}
                          >
                            {bodyText}
                          </ReactMarkdown>
                        </div>
                      )
                    )}
                  </section>
                );
              })}
            </section>
          )}

          {node && !selectedNodeIsVisible && !previewLoading && !previewError && (
            <div className="preview-state">Selected node is outside the loaded preview blocks for this document.</div>
          )}

          {node && (
            <section className="reader-section focus">
              <div className="focus-header">
                <h3>Selected Node</h3>
                <span className="focus-chip">CONFIDENCE {Math.round(confidence * 100)}%</span>
              </div>
              <p className="reader-subtitle">
                {node.nodeType} • {node.ordinalPath}
              </p>
              <div className="doc-flow-markdown">
                <ReactMarkdown remarkPlugins={[remarkGfm]} urlTransform={markdownUrlTransform}>
                  {node.text || "No text in selected node."}
                </ReactMarkdown>
              </div>
            </section>
          )}

          {relevantSteps.length > 0 && (
            <section className="reasoning-context">
              <h4>REASONING CONTEXT</h4>
              {relevantSteps.map((step) => (
                <div key={`${step.runId}-${step.idx}`} className="reasoning-step">
                  <div className="step-meta">
                    <span className="step-type">
                      STEP {String(step.idx).padStart(2, "0")} • {step.stepType.replaceAll("_", " ")}
                    </span>
                    <span className="step-confidence">{Math.round(step.confidence * 100)}%</span>
                  </div>
                  <p className="step-thought">
                    <strong>Thought:</strong> {step.thought}
                  </p>
                  {step.observation && (
                    <p className="step-observation">
                      <strong>Observation:</strong> {step.observation}
                    </p>
                  )}
                </div>
              ))}
            </section>
          )}

          {parent && (
            <section className="parent-context">
              <div className="parent-context-info">
                <h4>PARENT CONTEXT</h4>
                <p>{parent.title || parent.id}</p>
                <span>Contains {childCount} child node{childCount !== 1 ? "s" : ""}</span>
              </div>
              <button
                type="button"
                className="parent-nav-btn"
                onClick={() => onSelectNode(parent.id)}
                aria-label="Navigate to parent"
              >
                &rarr;
              </button>
            </section>
          )}
        </article>
      </div>
    </section>
  );
}
