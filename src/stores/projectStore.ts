import { create } from "zustand";
import {
  DocumentDetail,
  DocumentTreeItem,
  ProjectDetail,
  ProjectSummary,
  tauriApi,
} from "@/lib/tauri";
import { useEditorUiStore } from "@/stores/editorUiStore";

interface ProjectState {
  projects: ProjectSummary[];
  activeProject: ProjectDetail | null;
  documents: DocumentTreeItem[];
  activeDocument: DocumentDetail | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;

  loadProjects: () => Promise<void>;
  createProject: (title: string, language: string) => Promise<void>;
  openProject: (dbPath: string) => Promise<void>;
  refreshDocuments: () => Promise<void>;
  selectDocument: (docId: string) => Promise<void>;
  createDocument: (
    docType: "part" | "chapter" | "scene" | "note",
    title: string,
    parentId?: string | null
  ) => Promise<void>;
  saveDocument: (
    id: string,
    patch: { title?: string; status?: string; contentJson?: string; plainText?: string }
  ) => Promise<void>;
  setDocumentTrashed: (docId: string, trashed: boolean) => Promise<void>;
  reorderDocuments: (
    items: Array<{ id: string; parentId?: string | null; sortOrder: number }>
  ) => Promise<void>;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  projects: [],
  activeProject: null,
  documents: [],
  activeDocument: null,
  isLoading: false,
  isSaving: false,
  error: null,

  loadProjects: async () => {
    set({ isLoading: true, error: null });
    try {
      const projects = await tauriApi.listProjects();
      set({ projects, isLoading: false });
      if (projects.length > 0 && !get().activeProject) {
        await get().openProject(projects[0].path);
      }
    } catch (err) {
      set({ isLoading: false, error: String(err) });
    }
  },

  createProject: async (title, language) => {
    set({ isLoading: true, error: null });
    try {
      const activeProject = await tauriApi.createProject({ title, language });
      const [projects, documents] = await Promise.all([
        tauriApi.listProjects(),
        tauriApi.listDocuments(),
      ]);
      const firstScene = documents.find((d) => d.docType === "scene" && !d.isTrashed);
      const activeDocument = firstScene
        ? await tauriApi.getDocument(firstScene.id)
        : null;
      set({ projects, activeProject, documents, activeDocument, isLoading: false });
    } catch (err) {
      set({ isLoading: false, error: String(err) });
    }
  },

  openProject: async (dbPath) => {
    set({ isLoading: true, error: null });
    try {
      const activeProject = await tauriApi.openProject(dbPath);
      const documents = await tauriApi.listDocuments();
      const firstScene = documents.find((d) => d.docType === "scene" && !d.isTrashed);
      const activeDocument = firstScene
        ? await tauriApi.getDocument(firstScene.id)
        : null;
      set({ activeProject, documents, activeDocument, isLoading: false });
    } catch (err) {
      set({ isLoading: false, error: String(err) });
    }
  },

  refreshDocuments: async () => {
    const documents = await tauriApi.listDocuments();
    set({ documents });
  },

  selectDocument: async (docId) => {
    try {
      const activeDocument = await tauriApi.getDocument(docId);
      useEditorUiStore.getState().openTab(docId);
      set({ activeDocument, error: null });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  createDocument: async (docType, title, parentId = null) => {
    try {
      const created = await tauriApi.createDocument({ docType, title, parentId });
      const documents = await tauriApi.listDocuments();
      set({
        documents,
        activeDocument: docType === "scene" || docType === "note" ? created : get().activeDocument,
      });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  saveDocument: async (id, patch) => {
    const current = get().activeDocument;
    set({ isSaving: true, error: null });
    try {
      const updated = await tauriApi.updateDocument({
        id,
        ...patch,
        expectedRevision: current && current.id === id ? current.revision : undefined,
      });
      const documents = get().documents.map((d) =>
        d.id === id
          ? {
              ...d,
              title: updated.title,
              status: updated.status,
              revision: updated.revision,
              charsWithSpace: updated.charsWithSpace,
              charsNoSpace: updated.charsNoSpace,
              wordCount: updated.wordCount,
              updatedAt: updated.updatedAt,
            }
          : d
      );
      set({
        documents,
        activeDocument: current && current.id === id ? updated : current,
        isSaving: false,
      });
    } catch (err) {
      set({ isSaving: false, error: String(err) });
    }
  },

  setDocumentTrashed: async (docId, trashed) => {
    await tauriApi.setDocumentTrashed(docId, trashed);
    const documents = await tauriApi.listDocuments();
    const current = get().activeDocument;
    set({
      documents,
      activeDocument: trashed && current?.id === docId ? null : current,
    });
  },

  reorderDocuments: async (items) => {
    await tauriApi.reorderDocuments(items);
    const documents = await tauriApi.listDocuments();
    set({ documents });
  },
}));
