import { useCallback, useMemo, useState } from "react";

import { depthForOrdinal, nodeTypeLabel } from "../../lib/formatters";
import type { DocNodeSummary } from "../../lib/types";
import { useWorkspaceChrome } from "../navigation/WorkspaceChromeContext";
import { getNodeIcon } from "./getNodeIcon";

interface TreePaneProps {
  nodes: DocNodeSummary[];
  activeNodeId: string | null;
  onSelect: (nodeId: string) => void;
}

function isCollapsible(nodeType: string): boolean {
  const type = nodeType.toLowerCase();
  return type === "section" || type === "subsection" || type === "document";
}

/** Returns which sibling positions are "last child" at each ancestor depth,
 *  used to draw correct vertical guide lines. */
function computeLastFlags(
  nodes: DocNodeSummary[],
  nodeId: string,
  childrenMap: Map<string, DocNodeSummary[]>,
  parentId: string | null,
): boolean[] {
  const flags: boolean[] = [];
  let currentId = nodeId;
  let currentParentId = parentId;

  while (currentParentId !== null) {
    const siblings = childrenMap.get(currentParentId) ?? [];
    const isLast = siblings.at(-1)?.id === currentId;
    flags.unshift(isLast);
    const parent = nodes.find((n) => n.id === currentParentId);
    currentId = currentParentId;
    currentParentId = parent?.parentId ?? null;
  }
  return flags;
}

export function TreePane({
  nodes,
  activeNodeId,
  onSelect,
}: TreePaneProps) {
  const { projects, activeProjectId, setActiveProject, createProject } = useWorkspaceChrome();
  const [creatingProject, setCreatingProject] = useState(false);
  
  // Build children map for computing "is last child" flags
  const childrenMap = useMemo(() => {
    const map = new Map<string, DocNodeSummary[]>();
    for (const node of nodes) {
      if (node.parentId !== null) {
        const list = map.get(node.parentId) ?? [];
        list.push(node);
        map.set(node.parentId, list);
      }
    }
    return map;
  }, [nodes]);

  // Build initial expanded set: top 2 depth levels
  const initialExpanded = useMemo(() => {
    const ids = new Set<string>();
    for (const node of nodes) {
      if (depthForOrdinal(node.ordinalPath) < 2 && isCollapsible(node.nodeType)) {
        ids.add(node.id);
      }
    }
    return ids;
  }, [nodes]);

  const [expandedIds, setExpandedIds] = useState<Set<string>>(initialExpanded);

  // Keep expandedIds in sync when nodes change (new document loaded)
  const [prevNodes, setPrevNodes] = useState(nodes);
  if (nodes !== prevNodes) {
    setPrevNodes(nodes);
    const ids = new Set<string>();
    for (const node of nodes) {
      if (depthForOrdinal(node.ordinalPath) < 2 && isCollapsible(node.nodeType)) {
        ids.add(node.id);
      }
    }
    setExpandedIds(ids);
  }

  const toggleExpand = useCallback((nodeId: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(nodeId)) {
        next.delete(nodeId);
      } else {
        next.add(nodeId);
      }
      return next;
    });
  }, []);

  // Build a parentId lookup
  const parentMap = useMemo(() => {
    const map = new Map<string, string | null>();
    for (const node of nodes) {
      map.set(node.id, node.parentId);
    }
    return map;
  }, [nodes]);

  const isHiddenByCollapse = useCallback(
    (node: DocNodeSummary): boolean => {
      let currentParentId = node.parentId;
      while (currentParentId !== null) {
        if (!expandedIds.has(currentParentId)) return true;
        currentParentId = parentMap.get(currentParentId) ?? null;
      }
      return false;
    },
    [expandedIds, parentMap],
  );

  const filtered = nodes.filter((node) => !isHiddenByCollapse(node));

  const hasChildren = useMemo(() => {
    const set = new Set<string>();
    for (const node of nodes) {
      if (node.parentId !== null) set.add(node.parentId);
    }
    return set;
  }, [nodes]);

  const maxDepth = useMemo(
    () => nodes.reduce((acc, node) => Math.max(acc, depthForOrdinal(node.ordinalPath)), 0),
    [nodes],
  );

  const rootNode = useMemo(
    () => nodes.find((node) => node.parentId === null) ?? null,
    [nodes],
  );

  const visibleNodes = useMemo(
    () => filtered.filter((node) => node.id !== rootNode?.id),
    [filtered, rootNode],
  );

  const handleCreateProject = useCallback(async () => {
    const projectName = prompt("Enter project name:");
    if (!projectName || !projectName.trim()) return;
    
    setCreatingProject(true);
    try {
      await createProject(projectName.trim());
    } catch (error) {
      alert(`Failed to create project: ${error}`);
    } finally {
      setCreatingProject(false);
    }
  }, [createProject]);

  return (
    <section className="pane tree-pane">
      <header className="pane-header">
        <div className="pane-heading-group">
          <h2>EXPLORER</h2>
          {projects.length > 0 && (
            <select
              className="tree-doc-select"
              value={activeProjectId || ""}
              onChange={(e) => setActiveProject(e.target.value)}
              aria-label="Active project"
            >
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
          )}
        </div>
        <div className="tree-header-actions">
          <button
            type="button"
            className="tree-filter-btn"
            onClick={handleCreateProject}
            disabled={creatingProject}
            aria-label="New project"
            title="New project"
          >
            +
          </button>
        </div>
      </header>

      <div className="tree-content">
        {visibleNodes.map((node) => {
          const depth = depthForOrdinal(node.ordinalPath);
          const collapsible = isCollapsible(node.nodeType) && hasChildren.has(node.id);
          const expanded = expandedIds.has(node.id);
          const isLast = (() => {
            if (node.parentId === null) return true;
            const siblings = childrenMap.get(node.parentId) ?? [];
            return siblings.at(-1)?.id === node.id;
          })();
          const ancestorLastFlags = computeLastFlags(nodes, node.id, childrenMap, node.parentId);

          return (
            <div
              key={node.id}
              className={`tree-row${node.id === activeNodeId ? " active" : ""} tree-depth-${depth}`}
            >
              {/* Vertical guide lines for ancestor levels */}
              {depth > 0 &&
                Array.from({ length: depth }).map((_, i) => (
                  <span
                    key={i}
                    className={`tree-guide tree-guide-col-${i}${ancestorLastFlags[i] ? " last" : ""}`}
                  />
                ))}

              {/* The actual node button */}
              <button
                className="tree-node-btn"
                type="button"
                onClick={() => onSelect(node.id)}
              >
                {/* Elbow connector */}
                {depth > 0 && (
                  <span className={`tree-elbow${isLast ? " last" : ""}`} />
                )}

                {/* Expand/collapse chevron or leaf spacer */}
                {collapsible ? (
                  <span
                    className={`tree-chevron${expanded ? " open" : ""}`}
                    onClick={(e: React.MouseEvent) => {
                      e.stopPropagation();
                      toggleExpand(node.id);
                    }}
                  >
                    ▶
                  </span>
                ) : (
                  <span className="tree-leaf-spacer" />
                )}

                <span className="tree-node-icon">{getNodeIcon(node.nodeType)}</span>

                <span className="tree-node-body">
                  <span className="tree-node-title">
                    {node.title || `${nodeTypeLabel(node.nodeType)} ${node.ordinalPath}`}
                  </span>
                </span>
              </button>
            </div>
          );
        })}

        {visibleNodes.length === 0 && (
          <div className="tree-empty">
            <p>No nodes yet. Upload a document.</p>
          </div>
        )}
      </div>

      <footer className="pane-footer">
        <span>Nodes: {nodes.length}</span>
        <span>Depth: {maxDepth}</span>
      </footer>
    </section>
  );
}
