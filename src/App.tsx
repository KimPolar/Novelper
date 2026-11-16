import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { BookOpen, Cpu, Globe, History, Search, Sparkles, X } from "lucide-react";
import { AIAssistantDrawer } from "@/components/ai/AIAssistantDrawer";
import { AISettingsModal } from "@/components/ai/AISettingsModal";
import { NovelEditor } from "@/components/editor/NovelEditor";
import { SearchModal } from "@/components/editor/SearchModal";
import { VersionHistoryDrawer } from "@/components/editor/VersionHistoryDrawer";
import { ProjectSidebar } from "@/components/tree/ProjectSidebar";
import { useEditorUiStore } from "@/stores/editorUiStore";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

export function App() {
  const { t, i18n } = useTranslation();
  const { documents, activeDocument, isSaving, loadProjects, selectDocument } =
    useProjectStore();
  const { openTabIds, focusMode, closeTab } = useEditorUiStore();
  const [searchOpen, setSearchOpen] = useState(false);
  const [versionsOpen, setVersionsOpen] = useState(false);
  const [aiSettingsOpen, setAiSettingsOpen] = useState(false);
  const [aiDrawerOpen, setAiDrawerOpen] = useState(false);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

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
          <div className="flex items-center gap-2">
            {activeDocument && (
              <span className="mr-2 text-xs text-zinc-500">
                {isSaving ? t("editor.saving") : t("editor.saved")}
              </span>
            )}
            <button
              type="button"
              onClick={() => setSearchOpen(true)}
              className="flex items-center gap-1.5 rounded-md border border-zinc-800 bg-zinc-900 px-2.5 py-1 text-xs text-zinc-300 hover:border-zinc-700 hover:bg-zinc-800 transition-colors"
            >
              <Search className="h-3.5 w-3.5" />
              <span>{t("search.button")}</span>
            </button>
            {activeDocument && (
              <button
                type="button"
                onClick={() => setVersionsOpen((v) => !v)}
                className={cn(
                  "flex items-center gap-1.5 rounded-md border border-zinc-800 px-2.5 py-1 text-xs transition-colors",
                  versionsOpen
                    ? "bg-amber-500/20 text-amber-300 border-amber-500/40"
                    : "bg-zinc-900 text-zinc-300 hover:border-zinc-700 hover:bg-zinc-800"
                )}
              >
                <History className="h-3.5 w-3.5" />
                <span>{t("versions.button")}</span>
              </button>
            )}
            <button
              type="button"
              onClick={() => setAiDrawerOpen((v) => !v)}
              className={cn(
                "flex items-center gap-1.5 rounded-md border border-zinc-800 px-2.5 py-1 text-xs transition-colors",
                aiDrawerOpen
                  ? "bg-amber-500/20 text-amber-300 border-amber-500/40"
                  : "bg-zinc-900 text-zinc-300 hover:border-zinc-700 hover:bg-zinc-800"
              )}
            >
              <Sparkles className="h-3.5 w-3.5 text-amber-400" />
              <span>{t("aiPanel.button")}</span>
            </button>
            <button
              type="button"
              onClick={() => setAiSettingsOpen(true)}
              className="flex items-center gap-1.5 rounded-md border border-zinc-800 bg-zinc-900 px-2.5 py-1 text-xs text-zinc-300 hover:border-zinc-700 hover:bg-zinc-800 transition-colors"
            >
              <Cpu className="h-3.5 w-3.5 text-amber-400" />
              <span>{t("aiSettings.button")}</span>
            </button>
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

        {!focusMode && (
          <VersionHistoryDrawer
            open={versionsOpen}
            onClose={() => setVersionsOpen(false)}
          />
        )}
        {!focusMode && (
          <AIAssistantDrawer
            open={aiDrawerOpen}
            onClose={() => setAiDrawerOpen(false)}
          />
        )}
      </div>

      <SearchModal open={searchOpen} onClose={() => setSearchOpen(false)} />
      <AISettingsModal
        open={aiSettingsOpen}
        onClose={() => setAiSettingsOpen(false)}
      />
    </div>
  );
}

export default App;
