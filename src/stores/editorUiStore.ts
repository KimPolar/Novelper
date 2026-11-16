import { create } from "zustand";

interface EditorUiState {
  openTabIds: string[];
  splitDocId: string | null;
  focusMode: boolean;
  typewriterMode: boolean;

  openTab: (docId: string) => void;
  closeTab: (docId: string) => void;
  setSplitDocId: (docId: string | null) => void;
  toggleFocusMode: () => void;
  toggleTypewriterMode: () => void;
}

export const useEditorUiStore = create<EditorUiState>((set) => ({
  openTabIds: [],
  splitDocId: null,
  focusMode: false,
  typewriterMode: false,

  openTab: (docId) =>
    set((s) => ({
      openTabIds: s.openTabIds.includes(docId)
        ? s.openTabIds
        : [...s.openTabIds, docId],
    })),

  closeTab: (docId) =>
    set((s) => ({
      openTabIds: s.openTabIds.filter((id) => id !== docId),
      splitDocId: s.splitDocId === docId ? null : s.splitDocId,
    })),

  setSplitDocId: (docId) => set({ splitDocId: docId }),
  toggleFocusMode: () => set((s) => ({ focusMode: !s.focusMode })),
  toggleTypewriterMode: () => set((s) => ({ typewriterMode: !s.typewriterMode })),
}));
