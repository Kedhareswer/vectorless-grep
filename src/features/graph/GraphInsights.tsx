import { useMemo } from "react";
import { useShallow } from "zustand/react/shallow";

import { useVectorlessStore } from "../../lib/state";
import type { DocNodeSummary } from "../../lib/types";

interface EntityLink {
  name: string;
  count: number;
  maxCount: number;
  variant: "default" | "danger" | "ok";
}

function extractTopEntities(tree: DocNodeSummary[]): EntityLink[] {
  // Extract the most referenced entity types by counting nodes per section
  const sectionCounts = new Map<string, number>();
  for (const node of tree) {
    if (node.nodeType === "section" || node.nodeType === "subsection") {
      const label = node.title || node.ordinalPath;
      const children = tree.filter((n) => n.parentId === node.id).length;
      sectionCounts.set(label, children);
    }
  }

  const sorted = [...sectionCounts.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4);

  const maxCount = sorted[0]?.[1] ?? 1;
  const variants: ("default" | "danger" | "ok")[] = ["danger", "default", "ok", "default"];

  return sorted.map(([name, count], i) => ({
    name,
    count,
    maxCount,
    variant: variants[i % variants.length],
  }));
}

export function GraphInsights() {
  const { documents, tree, recentQueries } = useVectorlessStore(
    useShallow((s) => ({
      documents: s.documents,
      tree: s.tree,
      recentQueries: s.recentQueries,
    })),
  );

  const totalDocs = documents.length;
  const connections = tree.filter((n) => n.parentId !== null).length;
  const entities = useMemo(() => extractTopEntities(tree), [tree]);

  const handleExportGraph = () => {
    const lines = tree.map((node) =>
      JSON.stringify({
        id: node.id,
        parentId: node.parentId,
        type: node.nodeType,
        title: node.title,
        ordinalPath: node.ordinalPath,
      }),
    );
    const blob = new Blob([lines.join("\n")], { type: "application/jsonl" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "vectorless-graph.jsonl";
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <aside className="graph-insights">
      <h3 className="insights-heading">GLOBAL GRAPH INSIGHTS</h3>

      <div className="insights-stats">
        <div className="insights-stat-card">
          <span className="insights-stat-label">TOTAL DOCS</span>
          <span className="insights-stat-value">{totalDocs.toLocaleString()}</span>
        </div>
        <div className="insights-stat-card ok">
          <span className="insights-stat-label">CONNECTIONS</span>
          <span className="insights-stat-value">{connections.toLocaleString()}</span>
        </div>
      </div>

      <section className="insights-section">
        <h4>CROSS-DOCUMENT SYNTHESIS</h4>
        <p>
          {totalDocs > 1
            ? `LLM Reasoning Agent detected structural patterns across ${totalDocs} documents.`
            : "No cross-document synthesis."}
        </p>
      </section>

      {entities.length > 0 ? (
        <section className="insights-section">
          <h4>TOP ENTITY LINKS</h4>
          {entities.map((entity) => (
            <div key={entity.name} className="entity-link">
              <span className="entity-label">
                {entity.name}
                <span className="entity-count">{entity.count} nodes</span>
              </span>
              <div className="entity-bar">
                <div
                  className={`entity-bar-fill${entity.variant !== "default" ? ` ${entity.variant}` : ""}`}
                  ref={(el) => {
                    if (el) {
                      el.style.setProperty("--_fill", `${Math.round((entity.count / entity.maxCount) * 100)}%`);
                    }
                  }}
                />
              </div>
            </div>
          ))}
        </section>
      ) : null}

      <section className="insights-section">
        <h4>RECENT EXPLORATIONS</h4>
        {recentQueries.length > 0 ? (
          <ul className="recent-list">
            {recentQueries.map((query, i) => (
              <li key={`${query}-${i}`}>{query}</li>
            ))}
          </ul>
        ) : (
          <p>No recent explorations yet.</p>
        )}
      </section>

      <div className="insights-footer">
        <button type="button" className="doc-action-btn secondary" onClick={handleExportGraph}>
          EXPORT GRAPH (JSONL)
        </button>
      </div>
    </aside>
  );
}
