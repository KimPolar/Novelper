import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  FileText,
  FolderOpen,
  Plus,
  RotateCcw,
  StickyNote,
  Trash2,
} from "lucide-react";
import { DocumentTreeItem } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

export function ProjectSidebar() {
  const { t, i18n } = useTranslation();
  const {
    projects,
    activeProject,
    documents,
    activeDocument,
    createProject,
    openProject,
    selectDocument,
    createDocument,
    setDocumentTrashed,
  } = useProjectStore();

  const [showNewProject, setShowNewProject] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newLang, setNewLang] = useState<"ko" | "en">(
    i18n.language === "en" ? "en" : "ko"
  );
  const [showTrash, setShowTrash] = useState(false);

  const handleCreateProject = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTitle.trim()) return;
    await createProject(newTitle.trim(), newLang);
    setNewTitle("");
    setShowNewProject(false);
  };

  const activeDocs = documents.filter((d) => !d.isTrashed);
  const trashedDocs = documents.filter((d) => d.isTrashed);
  const roots = activeDocs.filter((d) => !d.parentId);

  const renderNode = (doc: DocumentTreeItem, depth = 0) => {
    const children = activeDocs.filter((d) => d.parentId === doc.id);
    const isSelected = activeDocument?.id === doc.id;
    const isContainer = doc.docType === "part" || doc.docType === "chapter";

    return (
      <div key={doc.id}>
        <div
          className={cn(
            "group flex items-center justify-between rounded-md px-2 py-1.5 text-sm cursor-pointer transition-colors",
            isSelected
              ? "bg-amber-500/15 text-amber-300"
              : "text-zinc-300 hover:bg-zinc-800/70"
          )}
          style={{ paddingLeft: `${depth * 12 + 8}px` }}
          onClick={() => selectDocument(doc.id)}
        >
          <div className="flex min-w-0 items-center gap-2">
            {isContainer ? (
              <FolderOpen className="h-3.5 w-3.5 shrink-0 text-amber-400" />
            ) : doc.docType === "note" ? (
              <StickyNote className="h-3.5 w-3.5 shrink-0 text-sky-400" />
            ) : (
              <FileText className="h-3.5 w-3.5 shrink-0 text-zinc-400" />
            )}
            <span className="truncate">{doc.title}</span>
          </div>
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100">
            {isContainer && (
              <button
                type="button"
                title={t("project.addScene")}
                onClick={(e) => {
                  e.stopPropagation();
                  createDocument(
                    "scene",
                    i18n.language === "en" ? "New Scene" : "새 장면",
                    doc.id
                  );
                }}
                className="rounded p-0.5 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100"
              >
                <Plus className="h-3 w-3" />
              </button>
            )}
            <button
              type="button"
              title={t("project.moveToTrash")}
              onClick={(e) => {
                e.stopPropagation();
                setDocumentTrashed(doc.id, true);
              }}
              className="rounded p-0.5 text-zinc-400 hover:bg-zinc-700 hover:text-red-400"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </div>
        </div>
        {children.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-zinc-800 bg-zinc-900/40">
      <div className="border-b border-zinc-800 p-3">
        <div className="flex items-center justify-between gap-2">
          <select
            value={activeProject?.id ?? ""}
            onChange={(e) => {
              const found = projects.find((p) => p.id === e.target.value);
              if (found) openProject(found.path);
            }}
            className="w-full rounded-md border border-zinc-800 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-200 focus:border-amber-500 focus:outline-none"
          >
            <option value="" disabled>
              {t("project.selectOrCreate")}
            </option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.title} ({p.language.toUpperCase()})
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => setShowNewProject((v) => !v)}
            title={t("project.newProject")}
            className="rounded-md border border-zinc-800 bg-zinc-900 p-1.5 text-zinc-300 hover:bg-zinc-800"
          >
            <Plus className="h-3.5 w-3.5" />
          </button>
        </div>

        {showNewProject && (
          <form onSubmit={handleCreateProject} className="mt-2 space-y-2 rounded-md border border-zinc-800 bg-zinc-950 p-2">
            <input
              type="text"
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              placeholder={t("project.titlePlaceholder")}
              className="w-full rounded border border-zinc-800 bg-zinc-900 px-2 py-1 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              autoFocus
            />
            <div className="flex items-center justify-between gap-2">
              <select
                value={newLang}
                onChange={(e) => setNewLang(e.target.value as "ko" | "en")}
                className="rounded border border-zinc-800 bg-zinc-900 px-2 py-1 text-xs text-zinc-200"
              >
                <option value="ko">한국어</option>
                <option value="en">English</option>
              </select>
              <div className="flex gap-1">
                <button
                  type="button"
                  onClick={() => setShowNewProject(false)}
                  className="rounded px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200"
                >
                  {t("project.cancel")}
                </button>
                <button
                  type="submit"
                  className="rounded bg-amber-500 px-2 py-1 text-xs font-medium text-zinc-950 hover:bg-amber-400"
                >
                  {t("project.create")}
                </button>
              </div>
            </div>
          </form>
        )}
      </div>

      {activeProject ? (
        <>
          <div className="flex items-center gap-1 border-b border-zinc-800/80 px-3 py-2">
            <button
              type="button"
              onClick={() =>
                createDocument(
                  "chapter",
                  i18n.language === "en" ? "New Chapter" : "새 장"
                )
              }
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
            >
              <Plus className="h-3 w-3" />
              {t("project.addChapter")}
            </button>
            <button
              type="button"
              onClick={() =>
                createDocument(
                  "note",
                  i18n.language === "en" ? "New Note" : "새 노트"
                )
              }
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
            >
              <StickyNote className="h-3 w-3" />
              {t("project.addNote")}
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            {roots.map((doc) => renderNode(doc))}
          </div>

          <div className="border-t border-zinc-800 p-2">
            <button
              type="button"
              onClick={() => setShowTrash((v) => !v)}
              className="flex w-full items-center justify-between rounded px-2 py-1 text-xs text-zinc-500 hover:bg-zinc-800/60 hover:text-zinc-300"
            >
              <span className="flex items-center gap-1.5">
                <Trash2 className="h-3.5 w-3.5" />
                {t("project.trash")}
              </span>
              <span>{trashedDocs.length}</span>
            </button>
            {showTrash && trashedDocs.length > 0 && (
              <div className="mt-1 max-h-32 space-y-1 overflow-y-auto pl-2">
                {trashedDocs.map((doc) => (
                  <div
                    key={doc.id}
                    className="flex items-center justify-between rounded px-2 py-1 text-xs text-zinc-400 hover:bg-zinc-800/50"
                  >
                    <span className="truncate">{doc.title}</span>
                    <button
                      type="button"
                      title={t("project.restore")}
                      onClick={() => setDocumentTrashed(doc.id, false)}
                      className="rounded p-0.5 text-zinc-400 hover:text-emerald-400"
                    >
                      <RotateCcw className="h-3 w-3" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </>
      ) : (
        <div className="flex flex-1 items-center justify-center p-4 text-center text-xs text-zinc-500">
          {t("project.emptyProject")}
        </div>
      )}
    </aside>
  );
}
