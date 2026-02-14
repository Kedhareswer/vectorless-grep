/* eslint-disable react-refresh/only-export-components */
import { createContext, type ReactNode, useContext } from "react";

import type { DocumentSummary, ProjectSummary } from "../../lib/types";

export interface FileStatus {
  name: string;
  state: "queued" | "parsing" | "done" | "error";
  message: string;
}

export interface WorkspaceChromeContextValue {
  projects: ProjectSummary[];
  activeProjectId: string | null;
  setActiveProject: (projectId: string | null) => void;
  createProject: (name: string) => Promise<void>;
  documents: DocumentSummary[];
  activeDocumentId: string | null;
  setActiveDocument: (documentId: string | null) => void;
  uploading: boolean;
  fileStatuses: FileStatus[];
  runIngestion: () => Promise<void>;
  openSettings: () => void;
}

const WorkspaceChromeContext = createContext<WorkspaceChromeContextValue | null>(null);

interface WorkspaceChromeProviderProps {
  value: WorkspaceChromeContextValue;
  children: ReactNode;
}

export function WorkspaceChromeProvider({ value, children }: WorkspaceChromeProviderProps) {
  return (
    <WorkspaceChromeContext.Provider value={value}>
      {children}
    </WorkspaceChromeContext.Provider>
  );
}

export function useWorkspaceChrome(): WorkspaceChromeContextValue {
  const context = useContext(WorkspaceChromeContext);
  if (!context) {
    throw new Error("useWorkspaceChrome must be used within WorkspaceChromeProvider");
  }
  return context;
}
