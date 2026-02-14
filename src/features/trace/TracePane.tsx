import { formatLatency } from "../../lib/formatters";
import type { AnswerRecord, DocNodeSummary, ReasoningRun, ReasoningStep } from "../../lib/types";
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
  const onSelectCitationNode = (nodeId: string): boolean => {
    const exists = tree.some((node) => node.id === nodeId);
    if (exists) {
      onSelectNode(nodeId);
    }
    return exists;
  };

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

      <article className="query-card">
        <span className="query-kicker">User Query</span>
        <p>{queryText.trim() || "No query."}</p>
      </article>

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
        {steps.map((step) => (
          <article
            key={`${step.runId}-${step.idx}`}
            className={`trace-card ${stepTypeClass(step.stepType)}`}
          >
            <div className="trace-meta">
              <span className="step-type">
                STEP {String(step.idx).padStart(2, "0")} &middot; {step.stepType.replaceAll("_", " ")}
              </span>
              <span className="timing-badge">{formatLatency(step.latencyMs)}</span>
            </div>
            <p className="trace-thought">
              <strong>Thought:</strong> {step.thought}
            </p>
            {step.action ? (
              <>
                <p className="trace-action">
                  <strong>Action:</strong>
                </p>
                <pre className="trace-snippet"><code>{step.action}</code></pre>
              </>
            ) : null}
            {step.observation ? (
              <p className="trace-observation">
                <strong>Observation:</strong> {step.observation}
              </p>
            ) : null}
            <div className="trace-footer">
              {step.confidence >= 0.7 ? (
                <span className="score-chip">
                  Score: {step.confidence.toFixed(2)}
                </span>
              ) : (
                <span className={confidenceChipClass(step.confidence)}>
                  Score: {step.confidence.toFixed(2)}
                </span>
              )}
              {step.nodeRefs[0] ? (
                <button
                  className="link-btn"
                  type="button"
                  onClick={() => onSelectNode(step.nodeRefs[0]!)}
                >
                  Focus {step.nodeRefs[0]}
                </button>
              ) : null}
            </div>
          </article>
        ))}
        {steps.length === 0 ? (
          <div className="trace-empty">
            <p>No trace.</p>
          </div>
        ) : null}

        <AnswerCard answer={answer} onSelectCitationNode={onSelectCitationNode} />

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
