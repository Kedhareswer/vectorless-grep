import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { UtilityRail } from "../../src/features/navigation/UtilityRail";
import {
  WorkspaceChromeProvider,
  type WorkspaceChromeContextValue,
} from "../../src/features/navigation/WorkspaceChromeContext";

const contextValue: WorkspaceChromeContextValue = {
  documents: [],
  activeDocumentId: null,
  setActiveDocument: vi.fn(),
  uploading: false,
  fileStatuses: [],
  runIngestion: vi.fn(async () => {}),
  openSettings: vi.fn(),
};

describe("UtilityRail", () => {
  it("fires upload and settings actions", () => {
    render(
      <WorkspaceChromeProvider value={contextValue}>
        <UtilityRail />
      </WorkspaceChromeProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Upload documents" }));
    fireEvent.click(screen.getByRole("button", { name: "Open settings" }));

    expect(contextValue.runIngestion).toHaveBeenCalledTimes(1);
    expect(contextValue.openSettings).toHaveBeenCalledTimes(1);
  });

  it("renders completion notice near upload button", () => {
    render(
      <WorkspaceChromeProvider value={contextValue}>
        <UtilityRail uploadNotice={{ type: "success", message: "Upload complete (2/2)" }} />
      </WorkspaceChromeProvider>,
    );

    expect(screen.getByText("Upload complete (2/2)")).toBeInTheDocument();
  });
});
