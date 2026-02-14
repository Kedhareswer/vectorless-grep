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

import { depthForOrdinal, nodeIcon } from "../../lib/formatters";
import { getGraphLayout, saveGraphLayout } from "../../lib/tauriApi";
import type { DocNodeSummary, GraphNodePosition } from "../../lib/types";
import { GraphInsights } from "./GraphInsights";

interface GraphPaneProps {
  documentId: string | null;
  nodes: DocNodeSummary[];
  activeNodeId: string | null;
  onSelect: (nodeId: string) => void;
  onToggleView: () => void;
}

type DocGraphNode = Node<{
  label: string;
  nodeType: string;
  nodeId: string;
  isActive: boolean;
  childCount: number;
}>;

type PersistedPositionMap = Map<string, { x: number; y: number }>;

function DocNode({ data }: NodeProps<DocGraphNode>) {
  return (
    <div className={`graph-node${data.isActive ? " selected" : ""}`}>
      <Handle type="target" position={Position.Top} style={{ visibility: "hidden" }} />
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
  const flowNodes: DocGraphNode[] = [];
  const edges: Edge[] = [];

  const childCounts = new Map<string, number>();
  for (const node of treeNodes) {
    if (node.parentId) {
      childCounts.set(node.parentId, (childCounts.get(node.parentId) ?? 0) + 1);
    }
  }

  const byDepth = new Map<number, DocNodeSummary[]>();
  for (const node of treeNodes) {
    const depth = depthForOrdinal(node.ordinalPath);
    const list = byDepth.get(depth) ?? [];
    list.push(node);
    byDepth.set(depth, list);
  }

  const NODE_WIDTH = 200;
  const NODE_HEIGHT = 90;
  const H_GAP = 30;
  const V_GAP = 50;

  for (const [depth, siblings] of byDepth) {
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

function GraphPaneInner({ documentId, nodes: treeNodes, activeNodeId, onSelect, onToggleView }: GraphPaneProps) {
  const { fitView, zoomIn, zoomOut, getNodes } = useReactFlow();
  const [viewMode, setViewMode] = useState<"cluster" | "hierarchy">("hierarchy");
  const [graphNodes, setGraphNodes] = useState<DocGraphNode[]>([]);
  const [graphEdges, setGraphEdges] = useState<Edge[]>([]);
  const [persistedPositions, setPersistedPositions] = useState<PersistedPositionMap>(new Map());
  const [layoutPersisted, setLayoutPersisted] = useState(false);
  const dirtyRef = useRef(false);
  const saveTimerRef = useRef<number | null>(null);

  const treeNodeIds = useMemo(() => new Set(treeNodes.map((node) => node.id)), [treeNodes]);

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

  const layout = useMemo(
    () => buildLayout(treeNodes, activeNodeId, persistedPositions),
    [treeNodes, activeNodeId, persistedPositions],
  );

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
    queuePersistLayout();
  }, [queuePersistLayout]);

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

  return (
    <section className="pane graph-pane">
      <header className="pane-header">
        <div className="pane-heading-group">
          <h2>DOC-AST GRAPH</h2>
          <p>
            Interactive node graph
            {layoutPersisted ? <span className="graph-persisted-badge">LAYOUT SAVED</span> : null}
          </p>
        </div>
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
            proOptions={{ hideAttribution: true }}
          >
            <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="var(--graph-bg-dot)" />
          </ReactFlow>
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
            <button type="button" onClick={resetLayout} title="Reset saved layout">
              &#x21BA;
            </button>
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
