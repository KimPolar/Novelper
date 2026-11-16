import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BookOpen, Globe } from "lucide-react";
import { ProjectSidebar } from "@/components/tree/ProjectSidebar";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

export function App() {
  const { t, i18n } = useTranslation();
  const { activeDocument, isSaving, loadProjects } = useProjectStore();

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const toggleLanguage = () => {
    const next = i18n.language === "ko" ? "en" : "ko";
    i18n.changeLanguage(next);
  };

  return (
    <div className="flex h-screen w-screen flex-col bg-zinc-950 text-zinc-100">
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
            <span>{i18n.language === "ko" ? t("language.ko") : t("language.en")}</span>
          </button>
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden">
        <ProjectSidebar />

        <main className="flex flex-1 flex-col overflow-hidden p-8">
          {activeDocument ? (
            <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col">
              <div className="mb-4 border-b border-zinc-800 pb-3">
                <h1 className="text-2xl font-semibold text-zinc-100">
                  {activeDocument.title}
                </h1>
                <div className="mt-1 flex gap-4 text-xs text-zinc-500">
                  <span>
                    {t("editor.charsWithSpace")}: {activeDocument.charsWithSpace}
                  </span>
                  <span>
                    {t("editor.charsNoSpace")}: {activeDocument.charsNoSpace}
                  </span>
                  <span>
                    {t("editor.words")}: {activeDocument.wordCount}
                  </span>
                </div>
              </div>
              <p className="text-sm text-zinc-400">
                {activeDocument.plainText || t("editor.placeholder")}
              </p>
            </div>
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
