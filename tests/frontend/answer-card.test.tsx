import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AnswerCard } from "../../src/features/answer/AnswerCard";

describe("AnswerCard", () => {
  it("renders answer text and citations", () => {
    render(
      <AnswerCard
        answer={{
          runId: "run-1",
          answerMarkdown: "Latency is **50ms** at p99.",
          citations: ["node-a", "node-b"],
          confidence: 0.91,
          grounded: true,
        }}
      />,
    );

    expect(screen.getByText("Answer")).toBeInTheDocument();
    expect(screen.getByText(/50ms/i)).toBeInTheDocument();
    expect(screen.getByText("node-a")).toBeInTheDocument();
    expect(screen.getByText("node-b")).toBeInTheDocument();
  });

  it("navigates when node citation is clicked", () => {
    const selected: string[] = [];
    render(
      <AnswerCard
        answer={{
          runId: "run-1",
          answerMarkdown: "See citations.",
          citations: ["root-12345678", "https://doi.org/abc"],
          confidence: 0.91,
          grounded: true,
        }}
        onSelectCitationNode={(nodeId) => {
          selected.push(nodeId);
          return true;
        }}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "root-12345678" }));
    expect(selected).toEqual(["root-12345678"]);
    expect(screen.getByText("https://doi.org/abc")).toBeInTheDocument();
  });
});
