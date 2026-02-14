import { useCallback, useState } from "react";
import ReactMarkdown from "react-markdown";

import { formatConfidence } from "../../lib/formatters";
import type { AnswerRecord } from "../../lib/types";

interface AnswerCardProps {
  answer: AnswerRecord | null;
  onSelectCitationNode?: (nodeId: string) => boolean;
}

function isNodeCitation(value: string): boolean {
  return /^[a-z]+-[a-z0-9-]{6,}$/i.test(value);
}

export function AnswerCard({ answer, onSelectCitationNode }: AnswerCardProps) {
  const [missingCitation, setMissingCitation] = useState<string | null>(null);

  const handleCopy = useCallback(() => {
    if (answer) {
      navigator.clipboard.writeText(answer.answerMarkdown);
    }
  }, [answer]);

  const handleCitationClick = useCallback(
    (citation: string) => {
      if (!onSelectCitationNode) return;
      const selected = onSelectCitationNode(citation);
      setMissingCitation(selected ? null : citation);
    },
    [onSelectCitationNode],
  );

  if (!answer) {
    return (
      <section className="answer-card empty">
        <h3>Answer</h3>
        <p>No answer.</p>
      </section>
    );
  }

  return (
    <div className="answer-wrapper">
      {answer.grounded && (
        <div className="answer-check" aria-label="Grounded answer">
          &#10003;
        </div>
      )}

      <section className="answer-card">
        <header className="answer-header">
          <h3>Answer</h3>
          <span className="chip">{formatConfidence(answer.confidence)}</span>
        </header>
        <div className="answer-body">
          <ReactMarkdown>{answer.answerMarkdown}</ReactMarkdown>
        </div>
        <div className="citations">
          {answer.citations.map((citation) => (
            isNodeCitation(citation) ? (
              <button
                key={citation}
                type="button"
                className="citation-node-btn"
                onClick={() => handleCitationClick(citation)}
                title="Jump to cited AST node"
              >
                {citation}
              </button>
            ) : (
              <code key={citation}>{citation}</code>
            )
          ))}
        </div>
        {missingCitation ? (
          <p className="citation-warning">Cited node not found in current tree: {missingCitation}</p>
        ) : null}
        <div className="answer-actions">
          <button
            className="answer-action-btn"
            type="button"
            onClick={handleCopy}
          >
            &#128203; Copy
          </button>
        </div>
      </section>
    </div>
  );
}
