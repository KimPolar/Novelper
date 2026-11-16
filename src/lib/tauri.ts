import { invoke } from "@tauri-apps/api/core";

export interface ProjectSummary {
  id: string;
  title: string;
  language: string;
  genre?: string | null;
  synopsis?: string | null;
  path: string;
  updatedAt: string;
}

export interface ProjectDetail {
  id: string;
  title: string;
  language: string;
  genre?: string | null;
  synopsis?: string | null;
  styleGuideJson?: string | null;
  dailyGoalCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentTreeItem {
  id: string;
  projectId: string;
  parentId?: string | null;
  docType: "part" | "chapter" | "scene" | "note" | string;
  title: string;
  sortOrder: number;
  status: "draft" | "writing" | "revising" | "done" | string;
  revision: number;
  charsWithSpace: number;
  charsNoSpace: number;
  wordCount: number;
  isTrashed: boolean;
  updatedAt: string;
}

export interface DocumentDetail extends DocumentTreeItem {
  povCharacterId?: string | null;
  locationId?: string | null;
  timelineLabel?: string | null;
  contentJson: string;
  plainText: string;
}

export interface FtsSearchHit {
  docId: string;
  title: string;
  snippet: string;
}

export const tauriApi = {
  listProjects: () => invoke<ProjectSummary[]>("list_projects"),
  createProject: (input: {
    title: string;
    language: string;
    genre?: string;
    synopsis?: string;
  }) => invoke<ProjectDetail>("create_project", { input }),
  openProject: (dbPath: string) =>
    invoke<ProjectDetail>("open_project", { dbPath }),
  listDocuments: () => invoke<DocumentTreeItem[]>("list_documents"),
  getDocument: (docId: string) =>
    invoke<DocumentDetail>("get_document", { docId }),
  createDocument: (input: {
    parentId?: string | null;
    docType: string;
    title: string;
  }) => invoke<DocumentDetail>("create_document", { input }),
  updateDocument: (input: {
    id: string;
    title?: string;
    status?: string;
    contentJson?: string;
    plainText?: string;
    expectedRevision?: number;
  }) => invoke<DocumentDetail>("update_document", { input }),
  reorderDocuments: (
    items: Array<{ id: string; parentId?: string | null; sortOrder: number }>
  ) => invoke<void>("reorder_documents", { items }),
  setDocumentTrashed: (docId: string, trashed: boolean) =>
    invoke<void>("set_document_trashed", { docId, trashed }),
  searchDocumentsFts: (query: string) =>
    invoke<FtsSearchHit[]>("search_documents_fts", { query }),
};
