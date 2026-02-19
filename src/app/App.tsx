import { useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";

import { DocumentPaneNew } from "../features/document/DocumentPaneNew";
import { GraphPane } from "../features/graph/GraphPane";
import { useWorkspaceChrome } from "../features/navigation/WorkspaceChromeContext";
import { TracePane } from "../features/trace/TracePane";
import { TreePane } from "../features/tree/TreePane";
import {
  deleteDocument,
  getNode,
  getProjectTree,
  getRun,
  listDocuments,
  listProjects,
  onReasoningComplete,
  onReasoningError,
  onReasoningStep,
  runReasoningQuery,
} from "../lib/tauriApi";
import { useVectorlessStore } from "../lib/state";
import type { ReasoningRun, ReasoningStep } from "../lib/types";

export function App() {
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const { documents, activeDocumentId } = useWorkspaceChrome();

  const {
    activeProjectId,
    tree,
    trace,
    answer,
    activeNodeId,
    nodeDetail,
    pending,
    queryText,
    currentRun,
    centerView,
  } = useVectorlessStore(
    useShallow((state) => ({
      activeProjectId: state.activeProjectId,
      tree: state.tree,
      trace: state.trace,
      answer: state.answer,
      activeNodeId: state.activeNodeId,
      nodeDetail: state.nodeDetail,
      pending: state.pending,
      queryText: state.queryText,
      currentRun: state.currentRun,
      centerView: state.centerView,
    })),
  );

  const {
    setProjects,
    setActiveProject,
    setDocuments,
    setActiveDocument,
    setTree,
    setNodeDetail,
    setTrace,
    appendTraceStep,
    setAnswer,
    setPending,
    setQueryText,
    selectNode,
    setCurrentRun,
    setIndexingStatus,
    setCenterView,
    addRecentQuery,
  } = useVectorlessStore(
    useShallow((state) => ({
      setProjects: state.setProjects,
      setActiveProject: state.setActiveProject,
      setDocuments: state.setDocuments,
      setActiveDocument: state.setActiveDocument,
      setTree: state.setTree,
      setNodeDetail: state.setNodeDetail,
      setTrace: state.setTrace,
      appendTraceStep: state.appendTraceStep,
      setAnswer: state.setAnswer,
      setPending: state.setPending,
      setQueryText: state.setQueryText,
      selectNode: state.selectNode,
      setCurrentRun: state.setCurrentRun,
      setIndexingStatus: state.setIndexingStatus,
      setCenterView: state.setCenterView,
      addRecentQuery: state.addRecentQuery,
    })),
  );

  const handleDeleteDocument = async (documentId: string) => {
    if (!activeProjectId) return;
    const confirmed = window.confirm("Delete this document from the project?");
    if (!confirmed) return;
    try {
      await deleteDocument(documentId);
      const docs = await listDocuments(activeProjectId);
      setDocuments(docs);
      if (activeDocumentId === documentId) {
        setActiveDocument(docs[0]?.id ?? null);
      }
      const nodes = await getProjectTree(activeProjectId, 6);
      setTree(nodes);
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  const activeDocument = useMemo(
    () => documents.find((item) => item.id === activeDocumentId) ?? null,
    [documents, activeDocumentId],
  );
  const activeTraceConfidence = useMemo(
    () => trace.at(-1)?.confidence ?? answer?.confidence ?? 0,
    [answer?.confidence, trace],
  );

  // Load projects on mount
  useEffect(() => {
    void listProjects().then((projects) => {
      setProjects(projects);
      if (projects.length > 0 && !activeProjectId) {
        setActiveProject(projects[0].id);
      }
    }).catch((error) => setErrorMessage(String(error)));
  }, [setProjects, setActiveProject, activeProjectId]);

  // Load project-wide tree for active project (always multi-document)
  useEffect(() => {
    if (!activeProjectId) {
      setTree([]);
      return;
    }

    setIndexingStatus("indexing");
    void getProjectTree(activeProjectId, 6)
      .then((nodes) => setTree(nodes))
      .catch((error) => setErrorMessage(String(error)));
  }, [activeProjectId, activeDocumentId, documents.length, setTree, setIndexingStatus]);

  useEffect(() => {
    if (!activeNodeId) {
      setNodeDetail(null);
      return;
    }
    let cancelled = false;
    void getNode(activeNodeId)
      .then((node) => { if (!cancelled) setNodeDetail(node); })
      .catch((error) => { if (!cancelled) setErrorMessage(String(error)); });
    return () => { cancelled = true; };
  }, [activeNodeId, setNodeDetail]);

  useEffect(() => {
    const stepPromise = onReasoningStep((event) => {
      if (!activeRunId || event.runId === activeRunId) {
        const mapped: ReasoningStep = {
          runId: event.runId,
          idx: event.stepIndex,
          stepType: event.stepType,
          thought: event.thought,
          action: event.action,
          observation: event.observation,
          nodeRefs: event.nodeRefs,
          confidence: event.confidence,
          latencyMs: event.latencyMs,
        };
        appendTraceStep(mapped);
      }
    });

    const completePromise = onReasoningComplete((event) => {
      if (activeRunId && event.runId !== activeRunId) {
        return;
      }
      setPending(false);
      void getRun(event.runId).then((payload) => {
        setTrace(payload.steps);
        setCurrentRun(payload.run);
        if (payload.answer) {
          setAnswer(payload.answer);
        }
      });
    });

    const errorPromise = onReasoningError((event) => {
      if (activeRunId && event.runId !== activeRunId) {
        return;
      }
      setPending(false);
      const message = event.code === "QUALITY_GATE_FAILED"
        ? `Answer withheld by quality policy: ${event.message}`
        : event.message;
      setErrorMessage(message);
      setCurrentRun({
        id: event.runId,
        projectId: activeProjectId ?? "",
        documentId: activeDocumentId,
        query: queryText,
        status: "failed",
        phase: "failed",
        startedAt: new Date().toISOString(),
        endedAt: new Date().toISOString(),
        totalLatencyMs: null,
        tokenUsageJson: {},
        costUsd: 0,
      } satisfies ReasoningRun);
    });

    return () => {
      void stepPromise.then((off) => off());
      void completePromise.then((off) => off());
      void errorPromise.then((off) => off());
    };
  }, [activeRunId, activeDocumentId, activeProjectId, queryText, appendTraceStep, setAnswer, setPending, setTrace, setCurrentRun]);

  const runQuery = async () => {
    if (!activeProjectId || !queryText.trim()) {
      return;
    }
    setErrorMessage(null);
    setPending(true);
    setTrace([]);
    setAnswer(null);
    setCurrentRun(null);
    selectNode(null);
    addRecentQuery(queryText.trim());
    try {
      const response = await runReasoningQuery(activeProjectId, queryText, 6, activeDocumentId);
      setActiveRunId(response.runId);
    } catch (error) {
      setPending(false);
      setErrorMessage(String(error));
    }
  };

  return (
    <>
      {errorMessage ? <p className="error-banner">{errorMessage}</p> : null}

      <main className="workspace-grid">
        <TreePane
          nodes={tree}
          activeNodeId={activeNodeId}
          onSelect={selectNode}
          onDeleteDocument={handleDeleteDocument}
        />
        {centerView === "trace" ? (
          <TracePane
            steps={trace}
            running={pending}
            answer={answer}
            tree={tree}
            run={currentRun}
            queryText={queryText}
            onQueryChange={setQueryText}
            onSubmit={() => {
              void runQuery();
            }}
            onSelectNode={selectNode}
            onRerun={() => {
              void runQuery();
            }}
            onToggleView={() => setCenterView("graph")}
          />
        ) : (
          <GraphPane
            documentId={activeDocumentId}
            nodes={tree}
            activeNodeId={activeNodeId}
            onSelect={selectNode}
            onToggleView={() => setCenterView("trace")}
          />
        )}
        <section className="document-column">
          <DocumentPaneNew
            document={activeDocument}
            node={nodeDetail}
            confidence={activeTraceConfidence}
            onSelectNode={selectNode}
            tree={tree}
          />
        </section>
      </main>
    </>
  );
}
