import { create } from "zustand";

import type {
  AnswerRecord,
  DocNodeDetail,
  DocNodeSummary,
  DocumentSummary,
  ProjectSummary,
  ReasoningRun,
  ReasoningStep,
} from "./types";

interface VectorlessState {
  activeRoute: "/" | "/settings";
  projects: ProjectSummary[];
  activeProjectId: string | null;
  documents: DocumentSummary[];
  activeDocumentId: string | null;
  leftPaneWidth: number;
  centerView: "trace" | "graph";
  tree: DocNodeSummary[];
  activeNodeId: string | null;
  nodeDetail: DocNodeDetail | null;
  trace: ReasoningStep[];
  currentRun: ReasoningRun | null;
  answer: AnswerRecord | null;
  pending: boolean;
  queryText: string;
  indexingStatus: "idle" | "indexing" | "complete";
  graphViewMode: "cluster" | "hierarchy";
  recentQueries: string[];
  setActiveRoute: (route: "/" | "/settings") => void;
  setProjects: (projects: ProjectSummary[]) => void;
  setActiveProject: (projectId: string | null) => void;
  setDocuments: (documents: DocumentSummary[]) => void;
  setActiveDocument: (documentId: string | null) => void;
  setLeftPaneWidth: (width: number) => void;
  setCenterView: (view: "trace" | "graph") => void;
  setTree: (nodes: DocNodeSummary[]) => void;
  setNodeDetail: (node: DocNodeDetail | null) => void;
  setTrace: (steps: ReasoningStep[]) => void;
  appendTraceStep: (step: ReasoningStep) => void;
  setCurrentRun: (run: ReasoningRun | null) => void;
  setAnswer: (answer: AnswerRecord | null) => void;
  setPending: (value: boolean) => void;
  setQueryText: (query: string) => void;
  selectNode: (nodeId: string | null) => void;
  setIndexingStatus: (status: "idle" | "indexing" | "complete") => void;
  setGraphViewMode: (mode: "cluster" | "hierarchy") => void;
  addRecentQuery: (query: string) => void;
  reset: () => void;
}

const initialState = {
  activeRoute: "/" as "/" | "/settings",
  projects: [] as ProjectSummary[],
  activeProjectId: null as string | null,
  documents: [] as DocumentSummary[],
  activeDocumentId: null as string | null,
  leftPaneWidth: 280,
  centerView: "trace" as "trace" | "graph",
  tree: [] as DocNodeSummary[],
  activeNodeId: null as string | null,
  nodeDetail: null as DocNodeDetail | null,
  trace: [] as ReasoningStep[],
  currentRun: null as ReasoningRun | null,
  answer: null as AnswerRecord | null,
  pending: false,
  queryText: "",
  indexingStatus: "idle" as "idle" | "indexing" | "complete",
  graphViewMode: "cluster" as "cluster" | "hierarchy",
  recentQueries: [] as string[],
};

export const useVectorlessStore = create<VectorlessState>((set) => ({
  ...initialState,
  setActiveRoute: (activeRoute) => set({ activeRoute }),
  setProjects: (projects) => set({ projects }),
  setActiveProject: (activeProjectId) =>
    set({
      activeProjectId,
      documents: [],
      activeDocumentId: null,
      tree: [],
      trace: [],
      answer: null,
      currentRun: null,
      activeNodeId: null,
      nodeDetail: null,
    }),
  setDocuments: (documents) => set({ documents }),
  setActiveDocument: (activeDocumentId) =>
    set({
      activeDocumentId,
      tree: [],
      trace: [],
      answer: null,
      currentRun: null,
      activeNodeId: null,
      nodeDetail: null,
    }),
  setLeftPaneWidth: (leftPaneWidth) => set({ leftPaneWidth }),
  setCenterView: (centerView) => set({ centerView }),
  setTree: (tree) =>
    set((state) => {
      const preservedActiveNodeId = state.activeNodeId && tree.some((node) => node.id === state.activeNodeId)
        ? state.activeNodeId
        : null;
      const fallbackRootNodeId = tree.find((node) => node.parentId === null)?.id ?? null;
      return {
        tree,
        activeNodeId: preservedActiveNodeId ?? fallbackRootNodeId,
        indexingStatus: "complete",
      };
    }),
  setNodeDetail: (nodeDetail) => set({ nodeDetail }),
  // Fix: only update activeNodeId if the trace has a concrete node ref,
  // otherwise preserve the user's current tree selection.
  setTrace: (trace) => {
    const lastNodeRef = trace.at(-1)?.nodeRefs?.[0] ?? null;
    set((state) => ({
      trace,
      activeNodeId: lastNodeRef ?? state.activeNodeId,
    }));
  },
  appendTraceStep: (step) =>
    set((state) => ({
      trace: [...state.trace, step],
      activeNodeId: step.nodeRefs[0] ?? state.activeNodeId,
    })),
  setCurrentRun: (currentRun) => set({ currentRun }),
  setAnswer: (answer) => set({ answer }),
  setPending: (pending) => set({ pending }),
  setQueryText: (queryText) => set({ queryText }),
  selectNode: (activeNodeId) => set({ activeNodeId }),
  setIndexingStatus: (indexingStatus) => set({ indexingStatus }),
  setGraphViewMode: (graphViewMode) => set({ graphViewMode }),
  addRecentQuery: (query) =>
    set((state) => ({
      recentQueries: [query, ...state.recentQueries.filter((q) => q !== query)].slice(0, 5),
    })),
  reset: () => set({ ...initialState }),
}));
