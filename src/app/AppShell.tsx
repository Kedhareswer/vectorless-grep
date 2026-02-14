import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { UtilityRail } from "../features/navigation/UtilityRail";
import {
  WorkspaceChromeProvider,
  type FileStatus,
} from "../features/navigation/WorkspaceChromeContext";
import { StatusBar } from "../features/status/StatusBar";
import { createProject, ingestDocument, listDocuments, listProjects, pickDocumentFiles } from "../lib/tauriApi";
import { useVectorlessStore } from "../lib/state";

interface AppShellProps {
  path: "/" | "/settings";
  navigate: (path: "/" | "/settings") => void;
  children: ReactNode;
}

type UploadNotice = {
  type: "success" | "error";
  message: string;
};

function inferMimeType(filePath: string): string {
  const ext = filePath.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "pdf":   return "application/pdf";
    case "pptx":  return "application/vnd.openxmlformats-officedocument.presentationml.presentation";
    case "docx":  return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    case "xlsx":  return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
    case "xls":   return "application/vnd.ms-excel";
    case "csv":   return "text/csv";
    case "txt":   return "text/plain";
    case "md":    return "text/markdown";
    case "png":   return "image/png";
    case "jpg":
    case "jpeg":  return "image/jpeg";
    case "webp":  return "image/webp";
    case "tiff":  return "image/tiff";
    default:      return "application/octet-stream";
  }
}

function fileNameFromPath(filePath: string): string {
  const normalized = filePath.replaceAll("\\", "/");
  const name = normalized.split("/").at(-1);
  return name && name.length > 0 ? name : filePath;
}

export function AppShell({ path, navigate, children }: AppShellProps) {
  const [fileStatuses, setFileStatuses] = useState<FileStatus[]>([]);
  const [uploading, setUploading] = useState(false);
  const [uploadNotice, setUploadNotice] = useState<UploadNotice | null>(null);
  const [railExpanded, setRailExpanded] = useState(false);
  const fileStatusClearTimerRef = useRef<number | null>(null);
  const noticeClearTimerRef = useRef<number | null>(null);

  const projects = useVectorlessStore((state) => state.projects);
  const activeProjectId = useVectorlessStore((state) => state.activeProjectId);
  const documents = useVectorlessStore((state) => state.documents);
  const activeDocumentId = useVectorlessStore((state) => state.activeDocumentId);
  const pending = useVectorlessStore((state) => state.pending);
  const setProjects = useVectorlessStore((state) => state.setProjects);
  const setActiveProject = useVectorlessStore((state) => state.setActiveProject);
  const setDocuments = useVectorlessStore((state) => state.setDocuments);
  const setActiveDocument = useVectorlessStore((state) => state.setActiveDocument);
  const setActiveRoute = useVectorlessStore((state) => state.setActiveRoute);

  const onSettingsPage = useMemo(() => path === "/settings", [path]);

  useEffect(() => {
    setActiveRoute(onSettingsPage ? "/settings" : "/");
  }, [onSettingsPage, setActiveRoute]);

  useEffect(
    () => () => {
      if (fileStatusClearTimerRef.current !== null) {
        window.clearTimeout(fileStatusClearTimerRef.current);
      }
      if (noticeClearTimerRef.current !== null) {
        window.clearTimeout(noticeClearTimerRef.current);
      }
    },
    [],
  );

  // Load projects on mount
  useEffect(() => {
    void listProjects().then((projects) => {
      setProjects(projects);
      if (projects.length > 0 && !activeProjectId) {
        setActiveProject(projects[0].id);
      }
    });
  }, [setProjects, setActiveProject, activeProjectId]);

  // Load documents for active project
  useEffect(() => {
    if (!activeProjectId) {
      setDocuments([]);
      return;
    }
    void listDocuments(activeProjectId).then((items) => {
      setDocuments(items);
      if (items.length > 0 && !activeDocumentId) {
        setActiveDocument(items[0].id);
      }
    });
  }, [activeProjectId, activeDocumentId, setActiveDocument, setDocuments]);

  const updateStatus = useCallback((name: string, patch: Partial<FileStatus>) => {
    setFileStatuses((prev) =>
      prev.map((s) => (s.name === name ? { ...s, ...patch } : s)),
    );
  }, []);

  const handleCreateProject = useCallback(async (name: string) => {
    try {
      const newProject = await createProject(name);
      const updatedProjects = await listProjects();
      setProjects(updatedProjects);
      setActiveProject(newProject.id);
    } catch (error) {
      console.error("Failed to create project:", error);
      throw error;
    }
  }, [setProjects, setActiveProject]);

  const runIngestion = useCallback(async () => {
    if (pending || uploading) {
      return;
    }

    if (fileStatusClearTimerRef.current !== null) {
      window.clearTimeout(fileStatusClearTimerRef.current);
      fileStatusClearTimerRef.current = null;
    }
    if (noticeClearTimerRef.current !== null) {
      window.clearTimeout(noticeClearTimerRef.current);
      noticeClearTimerRef.current = null;
    }
    setUploadNotice(null);

    const paths = await pickDocumentFiles();
    if (paths.length === 0) return;

    const initial: FileStatus[] = paths.map((p) => ({
      name: fileNameFromPath(p),
      state: "queued",
      message: "Queued",
    }));
    setFileStatuses(initial);
    setUploading(true);

    // Ingest all files in parallel
    const outcomes = await Promise.allSettled(
      paths.map(async (filePath) => {
        const name = fileNameFromPath(filePath);
        const mimeType = inferMimeType(filePath);
        updateStatus(name, { state: "parsing", message: "Parsing\u2026" });
        try {
          const result = await ingestDocument({ 
            filePath, 
            mimeType, 
            displayName: name,
            projectId: activeProjectId || "project-default"
          });
          updateStatus(name, { state: "done", message: `${result.nodeCount} nodes` });
          return { ok: true as const, documentId: result.documentId };
        } catch (err) {
          updateStatus(name, { state: "error", message: String(err).slice(0, 60) });
          return { ok: false as const };
        }
      }),
    );

    let successCount = 0;
    let failureCount = 0;
    let lastDocumentId: string | null = null;
    for (const outcome of outcomes) {
      if (outcome.status !== "fulfilled") {
        failureCount += 1;
        continue;
      }
      if (outcome.value.ok) {
        successCount += 1;
        lastDocumentId = outcome.value.documentId;
      } else {
        failureCount += 1;
      }
    }

    if (activeProjectId) {
      const docs = await listDocuments(activeProjectId);
      setDocuments(docs);
      if (lastDocumentId) setActiveDocument(lastDocumentId);
    }

    setUploading(false);
    const totalCount = paths.length;
    if (failureCount === 0) {
      setUploadNotice({
        type: "success",
        message: `Upload complete (${successCount}/${totalCount})`,
      });
    } else {
      setUploadNotice({
        type: "error",
        message: `Upload complete with errors (${successCount}/${totalCount})`,
      });
    }

    noticeClearTimerRef.current = window.setTimeout(() => {
      setUploadNotice(null);
      noticeClearTimerRef.current = null;
    }, failureCount === 0 ? 2000 : 4000);

    fileStatusClearTimerRef.current = window.setTimeout(() => {
      setFileStatuses([]);
      fileStatusClearTimerRef.current = null;
    }, 4000);
  }, [pending, setActiveDocument, setDocuments, updateStatus, uploading, activeProjectId]);

  const openSettings = useCallback(() => {
    setRailExpanded(false);
    navigate("/settings");
  }, [navigate]);

  const chromeValue = useMemo(
    () => ({
      projects,
      activeProjectId,
      setActiveProject,
      createProject: handleCreateProject,
      documents,
      activeDocumentId,
      setActiveDocument,
      uploading,
      fileStatuses,
      runIngestion,
      openSettings,
    }),
    [
      activeDocumentId,
      activeProjectId,
      documents,
      fileStatuses,
      handleCreateProject,
      openSettings,
      projects,
      runIngestion,
      setActiveDocument,
      setActiveProject,
      uploading,
    ],
  );

  const uploadProgress = useMemo(() => {
    const total = fileStatuses.length;
    const processed = fileStatuses.filter((status) => status.state === "done" || status.state === "error").length;
    return {
      active: uploading && total > 0,
      processed,
      total,
    };
  }, [fileStatuses, uploading]);

  return (
    <div className="app-shell">
      <WorkspaceChromeProvider value={chromeValue}>
        {onSettingsPage ? (
          children
        ) : (
          <div className={`workspace-shell${railExpanded ? " rail-expanded" : ""}`}>
            <UtilityRail onExpandedChange={setRailExpanded} uploadNotice={uploadNotice} />
            {children}
          </div>
        )}
      </WorkspaceChromeProvider>
      <StatusBar uploadProgress={uploadProgress} />
    </div>
  );
}
