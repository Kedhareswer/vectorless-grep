import { useEffect, useRef } from "react";
import type { DocNodeDetail, DocNodeSummary, DocumentSummary, ReasoningStep } from "../../lib/types";

interface DocumentPaneProps {
  document: DocumentSummary | null;
  node: DocNodeDetail | null;
  confidence: number;
  onSelectNode: (nodeId: string) => void;
  trace: ReasoningStep[];
  tree: DocNodeSummary[];
  queryText: string;
}

function formatMarkdown(text: string): string {
  // Simple markdown formatting
  return text
    .replace(/^# (.*$)/gm, '<h1>$1</h1>')
    .replace(/^## (.*$)/gm, '<h2>$1</h2>')
    .replace(/^### (.*$)/gm, '<h3>$1</h3>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/\n\n/g, '<br><br>')
    .replace(/\n/g, '<br>');
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
  const selectedNodeRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to selected node
  useEffect(() => {
    if (selectedNodeRef.current && contentRef.current) {
      selectedNodeRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, [node?.id]);

  const relevantSteps = node
    ? trace.filter((step) => step.nodeRefs.includes(node.id))
    : [];

  const parent = node?.parentId
    ? tree.find((n) => n.id === node.parentId) ?? null
    : null;

  const childCount = parent
    ? tree.filter((n) => n.parentId === parent.id).length
    : 0;

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
          
          {node ? (
            <>
              <div className="breadcrumb-row">
                Document / {node.ordinalPath}
              </div>

              <div className="extracted-content-header">
                <span className="section-kicker">FULL CONTEXT</span>
              </div>

              <h2>{node.title || document?.name || "No node selected."}</h2>
              <p className="reader-subtitle">
                {node.nodeType} • {node.ordinalPath}
              </p>

              <section className="reader-section focus" ref={selectedNodeRef}>
                <div className="focus-header">
                  <h3>Selected Node</h3>
                  <span className="focus-chip">CONFIDENCE {Math.round(confidence * 100)}%</span>
                </div>

                <div 
                  className="markdown-content"
                  dangerouslySetInnerHTML={{ __html: formatMarkdown(node.text) }}
                />
              </section>

              {relevantSteps.length > 0 && (
                <section className="reasoning-context">
                  <h4>REASONING CONTEXT</h4>
                  {relevantSteps.map((step) => (
                    <div key={`${step.runId}-${step.idx}`} className="reasoning-step">
                      <div className="step-meta">
                        <span className="step-type">
                          STEP {String(step.idx).padStart(2, "0")} • {step.stepType.replaceAll("_", " ")}
                        </span>
                        <span className="step-confidence">
                          {Math.round(step.confidence * 100)}%
                        </span>
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
            </>
          ) : (
            <div className="no-selection">
              <p>Select a node from the tree to view its content.</p>
            </div>
          )}
        </article>
      </div>
    </section>
  );
}