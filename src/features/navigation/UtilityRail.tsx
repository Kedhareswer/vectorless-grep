import { useState } from "react";

import { useWorkspaceChrome } from "./WorkspaceChromeContext";

interface UtilityRailProps {
  onExpandedChange?: (expanded: boolean) => void;
  uploadNotice?: { type: "success" | "error"; message: string } | null;
}

export function UtilityRail({ onExpandedChange, uploadNotice }: UtilityRailProps) {
  const [expanded, setExpanded] = useState(false);
  const { uploading, runIngestion, openSettings } = useWorkspaceChrome();
  const setRailExpanded = (next: boolean) => {
    setExpanded(next);
    onExpandedChange?.(next);
  };

  return (
    <aside
      className={`utility-rail${expanded ? " expanded" : ""}`}
      onMouseEnter={() => setRailExpanded(true)}
      onMouseLeave={() => setRailExpanded(false)}
      onFocusCapture={() => setRailExpanded(true)}
      onBlurCapture={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
          setRailExpanded(false);
        }
      }}
      aria-label="Workspace actions"
    >
      <button
        type="button"
        className="utility-rail-btn primary"
        onClick={() => void runIngestion()}
        aria-label="Upload documents"
        title="Upload documents"
        disabled={uploading}
      >
        <span className="utility-rail-icon" aria-hidden="true">&#8682;</span>
        <span className="utility-rail-label">
          {uploading ? "Uploading..." : "Upload"}
        </span>
      </button>

      <button
        type="button"
        className="utility-rail-btn"
        onClick={openSettings}
        aria-label="Open settings"
        title="Open settings"
      >
        <span className="utility-rail-icon" aria-hidden="true">&#9881;</span>
        <span className="utility-rail-label">Settings</span>
      </button>

      {uploadNotice ? (
        <div
          className={`utility-rail-notice ${uploadNotice.type}`}
          role="status"
          aria-live="polite"
        >
          {uploadNotice.message}
        </div>
      ) : null}
    </aside>
  );
}
