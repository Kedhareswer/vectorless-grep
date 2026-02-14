import { formatLatency } from "../../lib/formatters";
import type { AnswerRecord, DocNodeSummary, ReasoningRun, ReasoningStep } from "../../lib/types";
import { useWorkspaceChrome } from "../navigation/WorkspaceChromeContext";
import { AnswerCard } from "../answer/AnswerCard";

interface TracePaneProps {
  steps: ReasoningStep[];
  running: boolean;
  answer: AnswerRecord | null;
  tree: DocNodeSummary[];
  run: ReasoningRun | null;
  queryText: string;
  onQueryChange: (value: string) => void;
  onSubmit: () => void;
  onSelectNode: (nodeId: string) => void;
  onRerun: () => void;
  onToggleView: () => void;
}

function stepTypeClass(stepType: string): string {
  const lower = stepType.toLowerCase();
  if (lower.includes("retr")) return "is-retrieval";
  if (lower.includes("extract")) return "is-extract";
  if (lower.includes("synth")) return "is-synthesize";
  if (lower.includes("scan")) return "is-scan";
  if (lower.includes("nav") || lower.includes("select") || lower.includes("drill")) return "is-navigate";
  return "";
}

function confidenceChipClass(confidence: number): string {
  if (confidence >= 0.7) return "chip high";
  if (confidence >= 0.4) return "chip medium";
  return "chip low";
}

export function TracePane({
  steps,
  running,
  answer,
  tree,
  run,
  queryText,
  onQueryChange,
  onSubmit,
  onSelectNode,
  onRerun,
  onToggleView,
}: TracePaneProps) {
  const { documents } = useWorkspaceChrome();
  const onSelectCitationNode = (nodeId: string): boolean => {
    const exists = tree.some((node) => node.id === nodeId);
    if (exists) {
      onSelectNode(nodeId);
    }
    return exists;
  };

  const nodeById = new Map(tree.map((item) => [item.id, item]));
  const hasDocuments = documents.length > 0;
  const hasRunData = steps.length > 0 || !!answer || !!run;
  const showQueryCard = hasRunData && queryText.trim().length > 0;

  return (
    <section className="pane trace-pane">
      <header className="pane-header">
        <div className="pane-heading-group">
          <h2>Reasoning Timeline</h2>
          <p>Plan &rarr; Act &rarr; Observe &rarr; Refine</p>
        </div>
        <div className="center-view-toggle">
          <button type="button" className="active">Timeline</button>
          <button type="button" onClick={onToggleView}>Graph</button>
        </div>
      </header>

      {showQueryCard ? (
        <article className="query-card">
          <span className="query-kicker">User Query</span>
          <p>{queryText.trim()}</p>
        </article>
      ) : null}

      {run && run.status !== "running" ? (
        <div className="run-summary">
          <div className="run-summary-top">
            <span className={`run-badge ${run.status === "completed" ? "success" : "failed"}`}>
              {run.status === "completed" ? "SUCCESS" : "FAILED"}
            </span>
            <span className="run-id">ID: {run.id.slice(0, 8)}</span>
          </div>
          <div className="run-metrics-row">
            <div className="run-metric-card">
              <span className="run-metric-label">Latency</span>
              <span className="run-metric-value">{formatLatency(run.totalLatencyMs ?? 0)}</span>
            </div>
            <div className="run-metric-card">
              <span className="run-metric-label">Tokens</span>
              <span className="run-metric-value">
                {typeof run.tokenUsageJson === "object" && run.tokenUsageJson !== null
                  ? (run.tokenUsageJson as Record<string, unknown>).total?.toString() ?? "--"
                  : "--"}
              </span>
            </div>
            <div className="run-metric-card">
              <span className="run-metric-label">Cost</span>
              <span className="run-metric-value">${run.costUsd.toFixed(3)}</span>
            </div>
          </div>
        </div>
      ) : null}

      <div className="trace-timeline">
        {!hasDocuments ? (
          <div className="trace-empty-state">
            <h3>Get Started</h3>
            <p>Create a project and upload files to get started.</p>
          </div>
        ) : null}
        {hasDocuments && !hasRunData ? (
          <div className="trace-empty-state">
            <h3>Ask Your First Query</h3>
            <p>Ask a question to generate a grounded answer from your uploaded documents.</p>
          </div>
        ) : null}

        {hasRunData ? (
          <div className="timeline-stream">
            {steps.map((step) => {
            const firstNodeRef = step.nodeRefs[0];
            const nodeTitle = firstNodeRef ? (nodeById.get(firstNodeRef)?.title || firstNodeRef) : null;
            return (
              <article
                key={`${step.runId}-${step.idx}`}
                className={`timeline-step ${stepTypeClass(step.stepType)}`}
              >
                <span className="timeline-marker" aria-hidden="true" />
                <div className="timeline-step-main">
                  <div className="timeline-step-header">
                    <span className="timeline-step-index">STEP {String(step.idx).padStart(2, "0")}</span>
                    <span className="timeline-step-name">{step.stepType.replaceAll("_", " ")}</span>
                    <span className="timeline-step-latency">{formatLatency(step.latencyMs)}</span>
                  </div>
                  <div className="timeline-step-card">
                    <p className="trace-thought">{step.thought}</p>
                    {step.observation ? (
                      <p className="trace-observation">{step.observation}</p>
                    ) : null}
                    {step.action ? (
                      <pre className="trace-snippet"><code>{step.action}</code></pre>
                    ) : null}
                    <div className="trace-footer">
                      {nodeTitle ? (
                        <button
                          className="timeline-node-chip"
                          type="button"
                          onClick={() => onSelectNode(firstNodeRef!)}
                        >
                          Found: {nodeTitle}
                        </button>
                      ) : <span />}
                      {step.confidence >= 0.7 ? (
                        <span className="score-chip">
                          Score: {step.confidence.toFixed(2)}
                        </span>
                      ) : (
                        <span className={confidenceChipClass(step.confidence)}>
                          Score: {step.confidence.toFixed(2)}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              </article>
            );
            })}
          </div>
        ) : null}
        {answer ? (
          <AnswerCard answer={answer} onSelectCitationNode={onSelectCitationNode} />
        ) : null}

        {hasRunData ? (
          <div className="trace-actions">
            <button type="button" className="rerun-btn" onClick={onRerun} disabled={running}>
              Re-run Trace
            </button>
            <button
              type="button"
              className="download-btn"
              onClick={() => {
                const blob = new Blob(
                  [JSON.stringify({ run, steps, answer }, null, 2)],
                  { type: "application/json" },
                );
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = `trace-${run?.id?.slice(0, 8) ?? "export"}.json`;
                a.click();
                URL.revokeObjectURL(url);
              }}
            >
              Download Trace
            </button>
          </div>
        ) : null}
      </div>

      <form
        className="query-bar"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="query-bar-input-row">
          <textarea
            value={queryText}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="Ask a question about the document structure or content..."
            disabled={running}
            onKeyDown={(event) => {
              if (event.key === "Enter" && event.ctrlKey) {
                event.preventDefault();
                onSubmit();
              }
            }}
          />
          <button
            type="submit"
            className="query-send-btn"
            disabled={running || !queryText.trim()}
            aria-label="Send query"
          >
            &#x25B6;
          </button>
        </div>
      </form>
    </section>
  );
}
