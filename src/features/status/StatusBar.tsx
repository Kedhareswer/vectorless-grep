import { useShallow } from "zustand/react/shallow";

import { useVectorlessStore } from "../../lib/state";

interface UploadProgress {
  active: boolean;
  processed: number;
  total: number;
}

interface StatusBarProps {
  uploadProgress?: UploadProgress;
}

export function StatusBar({ uploadProgress }: StatusBarProps) {
  const { tree, indexingStatus } = useVectorlessStore(
    useShallow((s) => ({
      tree: s.tree,
      indexingStatus: s.indexingStatus,
    })),
  );

  const nodeCount = tree.length;
  const maxDepth = tree.reduce((acc, node) => {
    const depth =
      node.ordinalPath === "root"
        ? 0
        : node.ordinalPath.split(".").length - 1;
    return Math.max(acc, depth);
  }, 0);

  const dotClass =
    indexingStatus === "complete"
      ? "ok"
      : indexingStatus === "indexing"
        ? "indexing"
        : "idle";

  const statusLabel =
    indexingStatus === "complete"
      ? "Indexed"
      : indexingStatus === "indexing"
        ? "Indexing..."
        : "Idle";

  return (
    <footer className="status-bar">
      <span className="status-bar-item">
        <span className={`status-dot ${dotClass}`} />
        {statusLabel}
      </span>

      <span className="status-separator" />

      <span className="status-bar-item">Nodes: {nodeCount}</span>

      <span className="status-separator" />

      <span className="status-bar-item">Depth: {maxDepth}</span>

      {uploadProgress?.active ? (
        <>
          <span className="status-separator" />
          <span className="status-bar-item">Upload: {uploadProgress.processed}/{uploadProgress.total}</span>
        </>
      ) : null}

    </footer>
  );
}
