import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Background,
  BackgroundVariant,
  Handle,
  Position,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
  type NodeProps,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { compareOrdinalPath, depthForOrdinal, nodeIcon, nodeTypeLabel } from "../../lib/formatters";
import { getGraphLayout, saveGraphLayout } from "../../lib/tauriApi";
import type { DocNodeSummary, GraphNodePosition } from "../../lib/types";
import { GraphInsights } from "./GraphInsights";
import { buildHierarchyLayout } from "./hierarchyLayout";

interface GraphPaneProps {
  documentId: string | null;
  nodes: DocNodeSummary[];
  activeNodeId: string | null;
  onSelect: (nodeId: string) => void;
  onToggleView: () => void;
}

type GraphViewMode = "cluster" | "hierarchy";
type HierarchyScope = "active" | "project";

type DocGraphNode = Node<{
  label: string;
  nodeType: string;
  nodeId: string;
  ordinalPath: string;
  displayMode: GraphViewMode;
  isActive: boolean;
  childCount: number;
}>;

type PersistedPositionMap = Map<string, { x: number; y: number }>;

function DocNode({ data }: NodeProps<DocGraphNode>) {
  const isHierarchy = data.displayMode === "hierarchy";
  return (
    <div className={`graph-node${data.isActive ? " selected" : ""}${isHierarchy ? " hierarchy" : ""}`}>
      <Handle type="target" position={Position.Top} style={{ visibility: "hidden" }} />
      {isHierarchy ? (
        <>
          <div className="graph-node-title">{data.label}</div>
          <div className="graph-node-meta graph-node-meta-compact">
            <span>{nodeTypeLabel(data.nodeType)}</span>
            <span>{data.ordinalPath}</span>
          </div>
        </>
      ) : (
        <>
          <div className="graph-node-header">
            <span className="graph-node-icon">{nodeIcon(data.nodeType)}</span>
            <span className="graph-node-id">{data.nodeId.slice(0, 10)}</span>
            {data.isActive ? <span className="graph-node-selected-badge">SELECTED</span> : null}
          </div>
          <div className="graph-node-title">{data.label}</div>
          <div className="graph-node-meta">
            <span>AST Nodes: {data.childCount}</span>
            <span className="graph-node-status">Processed</span>
          </div>
        </>
      )}
      <Handle type="source" position={Position.Bottom} style={{ visibility: "hidden" }} />
    </div>
  );
}

const nodeTypes = { doc: DocNode };

function toPositionMap(positions: GraphNodePosition[]): PersistedPositionMap {
  const map: PersistedPositionMap = new Map();
  for (const position of positions) {
    map.set(position.nodeId, { x: position.x, y: position.y });
  }
  return map;
}

function buildLayout(
  treeNodes: DocNodeSummary[],
  activeNodeId: string | null,
  persistedPositions: PersistedPositionMap,
): { nodes: DocGraphNode[]; edges: Edge[] } {
  const sortedNodes = [...treeNodes].sort((a, b) => {
    const depthCompare = depthForOrdinal(a.ordinalPath) - depthForOrdinal(b.ordinalPath);
    if (depthCompare !== 0) return depthCompare;
    const ordinalCompare = compareOrdinalPath(a.ordinalPath, b.ordinalPath);
    if (ordinalCompare !== 0) return ordinalCompare;
    return (a.title || "").localeCompare(b.title || "", undefined, { numeric: true, sensitivity: "base" });
  });

  const flowNodes: DocGraphNode[] = [];
  const edges: Edge[] = [];

  const childCounts = new Map<string, number>();
  for (const node of sortedNodes) {
    if (node.parentId) {
      childCounts.set(node.parentId, (childCounts.get(node.parentId) ?? 0) + 1);
    }
  }

  const byDepth = new Map<number, DocNodeSummary[]>();
  for (const node of sortedNodes) {
    const depth = depthForOrdinal(node.ordinalPath);
    const list = byDepth.get(depth) ?? [];
    list.push(node);
    byDepth.set(depth, list);
  }

  const NODE_WIDTH = 200;
  const NODE_HEIGHT = 90;
  const H_GAP = 30;
  const V_GAP = 50;

  const depthLevels = [...byDepth.keys()].sort((a, b) => a - b);
  for (const depth of depthLevels) {
    const siblings = [...(byDepth.get(depth) ?? [])].sort((a, b) => {
      const ordinalCompare = compareOrdinalPath(a.ordinalPath, b.ordinalPath);
      if (ordinalCompare !== 0) return ordinalCompare;
      return (a.title || "").localeCompare(b.title || "", undefined, { numeric: true, sensitivity: "base" });
    });
    const totalWidth = siblings.length * NODE_WIDTH + (siblings.length - 1) * H_GAP;
    const startX = -totalWidth / 2;

    for (let i = 0; i < siblings.length; i++) {
      const node = siblings[i];
      const persisted = persistedPositions.get(node.id);
      flowNodes.push({
        id: node.id,
        type: "doc",
        position: persisted ?? {
          x: startX + i * (NODE_WIDTH + H_GAP),
          y: depth * (NODE_HEIGHT + V_GAP),
        },
        data: {
          label: node.title || node.ordinalPath,
          nodeType: node.nodeType,
          nodeId: node.id,
          ordinalPath: node.ordinalPath,
          displayMode: "cluster",
          isActive: node.id === activeNodeId,
          childCount: childCounts.get(node.id) ?? 0,
        },
      });

      if (node.parentId) {
        edges.push({
          id: `e-${node.parentId}-${node.id}`,
          source: node.parentId,
          target: node.id,
          type: "default",
          label: "contains",
          labelStyle: { fontSize: "var(--font-size-xs)", fill: "var(--text-2)" },
          style: { stroke: "var(--line-soft)" },
        });
      }
    }
  }

  return { nodes: flowNodes, edges };
}

function buildHierarchyFlow(
  treeNodes: DocNodeSummary[],
  activeNodeId: string | null,
): { nodes: DocGraphNode[]; edges: Edge[] } {
  const layout = buildHierarchyLayout(treeNodes);
  const childCounts = new Map<string, number>();
  for (const edge of layout.edges) {
    childCounts.set(edge.source, (childCounts.get(edge.source) ?? 0) + 1);
  }

  const flowNodes: DocGraphNode[] = layout.orderedNodes.map((node) => ({
    id: node.id,
    type: "doc",
    position: layout.positions.get(node.id) ?? { x: 0, y: 0 },
    data: {
      label: node.title || node.ordinalPath,
      nodeType: node.nodeType,
      nodeId: node.id,
      ordinalPath: node.ordinalPath,
      displayMode: "hierarchy",
      isActive: node.id === activeNodeId,
      childCount: childCounts.get(node.id) ?? 0,
    },
  }));

  const edges: Edge[] = layout.edges.map((edge) => ({
    id: `e-${edge.source}-${edge.target}`,
    source: edge.source,
    target: edge.target,
    type: "default",
    style: { stroke: "var(--line-soft)" },
  }));

  return { nodes: flowNodes, edges };
}

function GraphPaneInner({ documentId, nodes: treeNodes, activeNodeId, onSelect, onToggleView }: GraphPaneProps) {
  const { fitView, zoomIn, zoomOut, getNodes } = useReactFlow();
  const [viewMode, setViewMode] = useState<GraphViewMode>("hierarchy");
  const [hierarchyScope, setHierarchyScope] = useState<HierarchyScope>("active");
  const [graphNodes, setGraphNodes] = useState<DocGraphNode[]>([]);
  const [graphEdges, setGraphEdges] = useState<Edge[]>([]);
  const [persistedPositions, setPersistedPositions] = useState<PersistedPositionMap>(new Map());
  const [layoutPersisted, setLayoutPersisted] = useState(false);
  const dirtyRef = useRef(false);
  const saveTimerRef = useRef<number | null>(null);
  const resetInFlightRef = useRef(new Set<string>());

  const treeNodeIds = useMemo(() => new Set(treeNodes.map((node) => node.id)), [treeNodes]);
  const nodeById = useMemo(() => new Map(treeNodes.map((node) => [node.id, node])), [treeNodes]);
  const selectedNodeDocumentId = activeNodeId ? nodeById.get(activeNodeId)?.documentId ?? null : null;
  const targetHierarchyDocumentId = selectedNodeDocumentId ?? documentId;
  const hierarchyNodes = useMemo(() => {
    if (hierarchyScope === "active") {
      if (!targetHierarchyDocumentId) return [];
      return treeNodes.filter((node) => node.documentId === targetHierarchyDocumentId);
    }
    return treeNodes;
  }, [hierarchyScope, targetHierarchyDocumentId, treeNodes]);
  const hierarchyDocumentIds = useMemo(() => {
    if (hierarchyScope === "active") {
      return targetHierarchyDocumentId ? [targetHierarchyDocumentId] : [];
    }
    return [...new Set(treeNodes.map((node) => node.documentId))];
  }, [hierarchyScope, targetHierarchyDocumentId, treeNodes]);

  useEffect(() => {
    let cancelled = false;
    if (!documentId) {
      setPersistedPositions(new Map());
      setLayoutPersisted(false);
      return;
    }
    void getGraphLayout(documentId)
      .then((positions) => {
        if (cancelled) return;
        setPersistedPositions(toPositionMap(positions));
        setLayoutPersisted(positions.length > 0);
      })
      .catch(() => {
        if (!cancelled) {
          setPersistedPositions(new Map());
          setLayoutPersisted(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [documentId]);

  const clusterLayout = useMemo(
    () => buildLayout(treeNodes, activeNodeId, persistedPositions),
    [treeNodes, activeNodeId, persistedPositions],
  );
  const hierarchyLayout = useMemo(
    () => buildHierarchyFlow(hierarchyNodes, activeNodeId),
    [hierarchyNodes, activeNodeId],
  );

  const layout = viewMode === "cluster" ? clusterLayout : hierarchyLayout;

  useEffect(() => {
    setGraphNodes(layout.nodes);
    setGraphEdges(layout.edges);
  }, [layout]);

  useEffect(() => {
    setGraphNodes((prev) =>
      prev.map((node) => ({
        ...node,
        data: {
          ...node.data,
          isActive: node.id === activeNodeId,
        },
      })),
    );
  }, [activeNodeId]);

  const persistCurrentLayout = useCallback(() => {
    if (!documentId || !dirtyRef.current) return;
    const positions = getNodes()
      .filter((node) => treeNodeIds.has(node.id))
      .map((node) => ({ nodeId: node.id, x: node.position.x, y: node.position.y }));
    if (positions.length === 0) return;
    void saveGraphLayout(documentId, positions)
      .then(() => {
        setLayoutPersisted(true);
        dirtyRef.current = false;
      })
      .catch(() => {});
  }, [documentId, getNodes, treeNodeIds]);

  const queuePersistLayout = useCallback(() => {
    dirtyRef.current = true;
    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
    }
    saveTimerRef.current = window.setTimeout(() => {
      persistCurrentLayout();
      saveTimerRef.current = null;
    }, 500);
  }, [persistCurrentLayout]);

  useEffect(
    () => () => {
      if (saveTimerRef.current !== null) {
        window.clearTimeout(saveTimerRef.current);
      }
    },
    [],
  );

  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: DocGraphNode) => {
      onSelect(node.id);
    },
    [onSelect],
  );

  const onNodeDragStop = useCallback(() => {
    if (viewMode !== "cluster") return;
    queuePersistLayout();
  }, [queuePersistLayout, viewMode]);

  const resetLayout = useCallback(() => {
    if (!documentId) return;
    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
    dirtyRef.current = false;
    setPersistedPositions(new Map());
    setLayoutPersisted(false);
    void saveGraphLayout(documentId, []).finally(() => {
      void fitView({ duration: 240 });
    });
  }, [documentId, fitView]);

  useEffect(() => {
    if (viewMode !== "hierarchy") return;
    for (const docId of hierarchyDocumentIds) {
      if (!docId || resetInFlightRef.current.has(docId)) continue;
      const key = `graph-layout-reset-v2:${docId}`;
      if (window.localStorage.getItem(key) === "1") continue;
      resetInFlightRef.current.add(docId);
      void saveGraphLayout(docId, [])
        .then(() => {
          window.localStorage.setItem(key, "1");
        })
        .finally(() => {
          resetInFlightRef.current.delete(docId);
        });
    }
  }, [hierarchyDocumentIds, viewMode]);

  return (
    <section className="pane graph-pane">
      <header className="pane-header">
        <div className="pane-heading-group">
          <h2>DOC-AST GRAPH</h2>
          <p>
            Interactive node graph
            {viewMode === "cluster" && layoutPersisted ? <span className="graph-persisted-badge">LAYOUT SAVED</span> : null}
          </p>
        </div>
        <div className="graph-header-controls">
          <div className="center-view-toggle">
            <button type="button" onClick={onToggleView}>Timeline</button>
            <button
              type="button"
              className={viewMode === "cluster" ? "active" : ""}
              onClick={() => setViewMode("cluster")}
            >
              Global Cluster
            </button>
            <button
              type="button"
              className={viewMode === "hierarchy" ? "active" : ""}
              onClick={() => setViewMode("hierarchy")}
            >
              AST Hierarchy
            </button>
          </div>
          {viewMode === "hierarchy" ? (
            <div className="graph-scope-toggle" role="group" aria-label="Hierarchy scope">
              <button
                type="button"
                className={hierarchyScope === "active" ? "active" : ""}
                onClick={() => setHierarchyScope("active")}
              >
                Active Document
              </button>
              <button
                type="button"
                className={hierarchyScope === "project" ? "active" : ""}
                onClick={() => setHierarchyScope("project")}
              >
                Whole Project
              </button>
            </div>
          ) : null}
        </div>
      </header>
      <div className={`graph-layout${viewMode === "cluster" ? " with-sidebar" : ""}`}>
        <div className="graph-container">
          <ReactFlow
            nodes={graphNodes}
            edges={graphEdges}
            nodeTypes={nodeTypes}
            onNodeClick={onNodeClick}
            onNodeDragStop={onNodeDragStop}
            fitView
            minZoom={0.1}
            maxZoom={2}
            nodesDraggable={viewMode === "cluster"}
            proOptions={{ hideAttribution: true }}
          >
            <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="var(--graph-bg-dot)" />
          </ReactFlow>
          {viewMode === "hierarchy" && hierarchyNodes.length === 0 ? (
            <div className="graph-empty-state">No AST nodes available for this scope.</div>
          ) : null}
          <div className="graph-controls">
            <button type="button" onClick={() => void zoomIn({ duration: 220 })} title="Zoom in">
              +
            </button>
            <button type="button" onClick={() => void zoomOut({ duration: 220 })} title="Zoom out">
              -
            </button>
            <button type="button" onClick={() => void fitView({ duration: 240 })} title="Fit to view">
              &#x2922;
            </button>
            {viewMode === "cluster" ? (
              <button type="button" onClick={resetLayout} title="Reset saved layout">
                &#x21BA;
              </button>
            ) : null}
          </div>
        </div>
        {viewMode === "cluster" ? <GraphInsights /> : null}
      </div>
    </section>
  );
}

export function GraphPane(props: GraphPaneProps) {
  return (
    <ReactFlowProvider>
      <GraphPaneInner {...props} />
    </ReactFlowProvider>
  );
}
