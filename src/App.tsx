import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BookOpen, Globe, X } from "lucide-react";
import { NovelEditor } from "@/components/editor/NovelEditor";
import { ProjectSidebar } from "@/components/tree/ProjectSidebar";
import { useEditorUiStore } from "@/stores/editorUiStore";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

export function App() {
  const { t, i18n } = useTranslation();
  const { documents, activeDocument, isSaving, loadProjects, selectDocument } =
    useProjectStore();
  const { openTabIds, focusMode, closeTab } = useEditorUiStore();

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const toggleLanguage = () => {
    const next = i18n.language === "ko" ? "en" : "ko";
    i18n.changeLanguage(next);
  };

  return (
    <div className="flex h-screen w-screen flex-col bg-zinc-950 text-zinc-100">
      {!focusMode && (
        <header className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-800 px-4">
          <div className="flex items-center gap-2.5">
            <BookOpen className="h-5 w-5 text-amber-400" />
            <span className="font-semibold tracking-tight">{t("app.name")}</span>
            <span className="text-xs text-zinc-500">{t("app.subtitle")}</span>
          </div>
          <div className="flex items-center gap-3">
            {activeDocument && (
              <span className="text-xs text-zinc-500">
                {isSaving ? t("editor.saving") : t("editor.saved")}
              </span>
            )}
            <button
              type="button"
              onClick={toggleLanguage}
              className={cn(
                "flex items-center gap-1.5 rounded-md border border-zinc-800 bg-zinc-900 px-2.5 py-1 text-xs text-zinc-300",
                "hover:border-zinc-700 hover:bg-zinc-800 transition-colors"
              )}
            >
              <Globe className="h-3.5 w-3.5" />
              <span>
                {i18n.language === "ko" ? t("language.ko") : t("language.en")}
              </span>
            </button>
          </div>
        </header>
      )}

      <div className="flex flex-1 overflow-hidden">
        {!focusMode && <ProjectSidebar />}

        <main className="flex flex-1 flex-col overflow-hidden">
          {!focusMode && openTabIds.length > 0 && (
            <div className="flex items-center gap-1 overflow-x-auto border-b border-zinc-800 bg-zinc-900/30 px-2 py-1">
              {openTabIds.map((tabId) => {
                const doc = documents.find((d) => d.id === tabId);
                if (!doc) return null;
                const isActive = activeDocument?.id === tabId;
                return (
                  <div
                    key={tabId}
                    onClick={() => selectDocument(tabId)}
                    className={cn(
                      "flex cursor-pointer items-center gap-2 rounded-t px-3 py-1 text-xs transition-colors",
                      isActive
                        ? "bg-zinc-900 text-amber-300 border-b-2 border-amber-400"
                        : "text-zinc-400 hover:bg-zinc-800/60 hover:text-zinc-200"
                    )}
                  >
                    <span className="max-w-36 truncate">{doc.title}</span>
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        closeTab(tabId);
                      }}
                      className="rounded p-0.5 hover:bg-zinc-700"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </div>
                );
              })}
            </div>
          )}

          {activeDocument ? (
            <NovelEditor document={activeDocument} />
          ) : (
            <div className="flex flex-1 items-center justify-center">
              <div className="max-w-md text-center">
                <h1 className="text-xl font-semibold text-zinc-100">
                  {t("editor.untitledScene")}
                </h1>
                <p className="mt-2 text-sm text-zinc-400">
                  {t("project.emptyProject")}
                </p>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
