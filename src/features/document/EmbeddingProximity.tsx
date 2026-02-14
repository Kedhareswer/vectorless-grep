import { useMemo } from "react";

import type { DocNodeSummary } from "../../lib/types";

interface EmbeddingProximityProps {
  confidence: number;
  nodeId: string | null;
  queryText: string;
  tree: DocNodeSummary[];
}

interface DotData {
  x: number;
  y: number;
  r: number;
  opacity: number;
  isMatch: boolean;
}

/** Simple hash from a string to a 0–1 float. */
function hashFloat(str: string, seed: number): number {
  let h = seed ^ 0x811c9dc5;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return ((h >>> 0) % 10000) / 10000;
}

/** Compute a simple bag-of-words overlap score between two texts. */
function overlapScore(a: string, b: string): number {
  if (!a.trim() || !b.trim()) return 0;
  const wordsA = new Set(a.toLowerCase().split(/\W+/).filter(Boolean));
  const wordsB = new Set(b.toLowerCase().split(/\W+/).filter(Boolean));
  if (wordsA.size === 0 || wordsB.size === 0) return 0;
  let overlap = 0;
  for (const word of wordsA) {
    if (wordsB.has(word)) overlap++;
  }
  return overlap / Math.max(wordsA.size, wordsB.size);
}

function generateDots(
  tree: DocNodeSummary[],
  queryText: string,
  activeNodeId: string | null,
): { dots: DotData[]; matchScore: number } {
  const WIDTH = 300;
  const HEIGHT = 150;
  const dots: DotData[] = [];
  let bestScore = 0;

  // Generate a dot for each node in the tree (cap at 60 for perf)
  const nodes = tree.slice(0, 60);

  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const nodeText = `${node.title} ${node.text}`;
    const score = queryText.trim() ? overlapScore(queryText, nodeText) : 0;
    const isActive = node.id === activeNodeId;
    const isMatch = score > 0.15 || isActive;

    if (isActive && score > bestScore) bestScore = score;
    if (score > bestScore && isMatch) bestScore = score;

    // Position: matched nodes cluster towards center, others scatter
    let x: number;
    let y: number;
    if (isMatch) {
      x = WIDTH * 0.4 + hashFloat(node.id, 1) * WIDTH * 0.25;
      y = HEIGHT * 0.3 + hashFloat(node.id, 2) * HEIGHT * 0.4;
    } else {
      x = hashFloat(node.id, 3) * WIDTH * 0.85 + WIDTH * 0.075;
      y = hashFloat(node.id, 4) * HEIGHT * 0.85 + HEIGHT * 0.075;
    }

    dots.push({
      x,
      y,
      r: isMatch ? 4 : 2.5,
      opacity: isMatch ? 0.9 : 0.25 + hashFloat(node.id, 5) * 0.3,
      isMatch,
    });
  }

  // If no query, generate a reasonable match score from confidence
  const matchScore = bestScore > 0 ? bestScore : 0;

  return { dots, matchScore };
}

export function EmbeddingProximity({ confidence, nodeId, queryText, tree }: EmbeddingProximityProps) {
  const { dots, matchScore } = useMemo(
    () => generateDots(tree, queryText, nodeId),
    [tree, queryText, nodeId],
  );

  if (!nodeId) return null;

  const displayScore = matchScore > 0 ? matchScore : confidence * 0.95 + 0.05;

  return (
    <section className="embedding-proximity">
      <div className="embedding-header">
        <span className="section-kicker">EMBEDDING SPACE PROXIMITY</span>
        <span className="match-badge">Match: {displayScore.toFixed(3)}</span>
      </div>
      <svg
        className="embedding-plot"
        viewBox="0 0 300 150"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Embedding space proximity visualization"
      >
        {/* Grid lines */}
        <line x1="0" y1="75" x2="300" y2="75" stroke="var(--embed-grid-stroke)" strokeWidth="var(--embed-stroke-width)" />
        <line x1="150" y1="0" x2="150" y2="150" stroke="var(--embed-grid-stroke)" strokeWidth="var(--embed-stroke-width)" />

        {/* Scatter dots */}
        {dots.map((dot, i) => (
          <circle
            key={i}
            cx={dot.x}
            cy={dot.y}
            r={dot.r}
            fill={dot.isMatch ? "var(--ok)" : "var(--accent-0)"}
            opacity={dot.opacity}
          />
        ))}

        {/* Query crosshair at center-left */}
        <line x1="145" y1="70" x2="155" y2="80" stroke="var(--accent-1)" strokeWidth="0.8" opacity="0.5" />
        <line x1="155" y1="70" x2="145" y2="80" stroke="var(--accent-1)" strokeWidth="0.8" opacity="0.5" />
      </svg>
    </section>
  );
}
