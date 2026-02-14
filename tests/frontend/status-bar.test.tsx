import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { StatusBar } from "../../src/features/status/StatusBar";
import { useVectorlessStore } from "../../src/lib/state";

describe("StatusBar upload progress", () => {
  it("shows upload segment only when active", () => {
    useVectorlessStore.getState().reset();

    const { rerender } = render(<StatusBar />);
    expect(screen.queryByText(/Upload:/i)).not.toBeInTheDocument();

    rerender(
      <StatusBar uploadProgress={{ active: true, processed: 2, total: 5 }} />,
    );
    expect(screen.getByText("Upload: 2/5")).toBeInTheDocument();

    rerender(
      <StatusBar uploadProgress={{ active: false, processed: 5, total: 5 }} />,
    );
    expect(screen.queryByText(/Upload:/i)).not.toBeInTheDocument();
  });
});
