import { compareOrdinalPath } from "../../lib/formatters";
import type { DocNodeSummary } from "../../lib/types";

export interface HierarchyLayoutEdge {
  source: string;
  target: string;
}

export interface HierarchyLayoutPosition {
  x: number;
  y: number;
}

export interface HierarchyLayoutResult {
  orderedNodes: DocNodeSummary[];
  edges: HierarchyLayoutEdge[];
  positions: Map<string, HierarchyLayoutPosition>;
}

interface HierarchyLayoutOptions {
  nodeWidth?: number;
  nodeHeight?: number;
  horizontalGap?: number;
  verticalGap?: number;
  rootGap?: number;
}

const DEFAULT_OPTIONS: Required<HierarchyLayoutOptions> = {
  nodeWidth: 220,
  nodeHeight: 84,
  horizontalGap: 28,
  verticalGap: 72,
  rootGap: 72,
};

export function buildHierarchyLayout(
  inputNodes: DocNodeSummary[],
  options: HierarchyLayoutOptions = {},
): HierarchyLayoutResult {
  if (inputNodes.length === 0) {
    return {
      orderedNodes: [],
      edges: [],
      positions: new Map(),
    };
  }

  const config = { ...DEFAULT_OPTIONS, ...options };
  const indexById = new Map(inputNodes.map((node, index) => [node.id, index]));
  const nodeById = new Map(inputNodes.map((node) => [node.id, node]));
  const childrenMap = new Map<string, DocNodeSummary[]>();
  const roots: DocNodeSummary[] = [];

  for (const node of inputNodes) {
    if (node.parentId && nodeById.has(node.parentId)) {
      const list = childrenMap.get(node.parentId) ?? [];
      list.push(node);
      childrenMap.set(node.parentId, list);
      continue;
    }
    roots.push(node);
  }

  const siblingComparator = (a: DocNodeSummary, b: DocNodeSummary): number => {
    if (a.documentId !== b.documentId) {
      return (indexById.get(a.id) ?? 0) - (indexById.get(b.id) ?? 0);
    }
    const ordinalCompare = compareOrdinalPath(a.ordinalPath, b.ordinalPath);
    if (ordinalCompare !== 0) return ordinalCompare;
    const titleCompare = (a.title || "").localeCompare(b.title || "", undefined, {
      numeric: true,
      sensitivity: "base",
    });
    if (titleCompare !== 0) return titleCompare;
    return (indexById.get(a.id) ?? 0) - (indexById.get(b.id) ?? 0);
  };

  for (const [parentId, children] of childrenMap.entries()) {
    childrenMap.set(parentId, [...children].sort(siblingComparator));
  }
  roots.sort(siblingComparator);

  const subtreeWidth = new Map<string, number>();
  const computeSubtreeWidth = (node: DocNodeSummary): number => {
    const children = childrenMap.get(node.id) ?? [];
    if (children.length === 0) {
      subtreeWidth.set(node.id, config.nodeWidth);
      return config.nodeWidth;
    }
    let total = 0;
    for (let i = 0; i < children.length; i += 1) {
      total += computeSubtreeWidth(children[i]);
      if (i < children.length - 1) total += config.horizontalGap;
    }
    const resolved = Math.max(config.nodeWidth, total);
    subtreeWidth.set(node.id, resolved);
    return resolved;
  };

  for (const root of roots) {
    computeSubtreeWidth(root);
  }

  const positions = new Map<string, HierarchyLayoutPosition>();
  const orderedNodes: DocNodeSummary[] = [];
  const edges: HierarchyLayoutEdge[] = [];

  const placeNode = (node: DocNodeSummary, depth: number, centerX: number): void => {
    positions.set(node.id, {
      x: centerX - config.nodeWidth / 2,
      y: depth * (config.nodeHeight + config.verticalGap),
    });
    orderedNodes.push(node);

    const children = childrenMap.get(node.id) ?? [];
    if (children.length === 0) return;

    let totalChildrenWidth = 0;
    for (let i = 0; i < children.length; i += 1) {
      totalChildrenWidth += subtreeWidth.get(children[i].id) ?? config.nodeWidth;
      if (i < children.length - 1) totalChildrenWidth += config.horizontalGap;
    }

    let cursor = centerX - totalChildrenWidth / 2;
    for (const child of children) {
      const childWidth = subtreeWidth.get(child.id) ?? config.nodeWidth;
      const childCenter = cursor + childWidth / 2;
      edges.push({ source: node.id, target: child.id });
      placeNode(child, depth + 1, childCenter);
      cursor += childWidth + config.horizontalGap;
    }
  };

  let rootCursor = 0;
  for (let i = 0; i < roots.length; i += 1) {
    const root = roots[i];
    const width = subtreeWidth.get(root.id) ?? config.nodeWidth;
    const rootCenter = rootCursor + width / 2;
    placeNode(root, 0, rootCenter);
    rootCursor += width;
    if (i < roots.length - 1) rootCursor += config.rootGap;
  }

  const totalWidth = rootCursor;
  const shift = totalWidth / 2;
  for (const [nodeId, point] of positions.entries()) {
    positions.set(nodeId, { x: point.x - shift, y: point.y });
  }

  return {
    orderedNodes,
    edges,
    positions,
  };
}
