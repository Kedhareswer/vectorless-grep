import { describe, expect, it } from "vitest";

import { useVectorlessStore } from "../../src/lib/state";
import type { DocNodeSummary, ReasoningStep } from "../../src/lib/types";

const makeNode = (id: string): DocNodeSummary => ({
  id,
  documentId: "doc-1",
  parentId: null,
  nodeType: "section",
  title: `Node ${id}`,
  text: "",
  ordinalPath: id,
  pageStart: 1,
  pageEnd: 1,
});

const makeStep = (idx: number): ReasoningStep => ({
  runId: "run-1",
  idx,
  stepType: "scan_root",
  thought: "Scan root",
  action: "scan",
  observation: "ok",
  nodeRefs: idx === 1 ? ["n-2"] : ["n-1"],
  confidence: 0.9,
  latencyMs: 120,
});

describe("vectorless state synchronization", () => {
  it("keeps active node aligned across tree and reasoning trace", () => {
    const store = useVectorlessStore.getState();
    store.reset();

    useVectorlessStore.getState().setTree([makeNode("n-1"), makeNode("n-2")]);
    useVectorlessStore.getState().setTrace([makeStep(1)]);

    expect(useVectorlessStore.getState().activeNodeId).toBe("n-2");

    useVectorlessStore.getState().selectNode("n-1");
    expect(useVectorlessStore.getState().activeNodeId).toBe("n-1");
  });
});
