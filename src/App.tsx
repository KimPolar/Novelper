import { useTranslation } from "react-i18next";
import { BookOpen, Globe, Settings, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";

export function App() {
  const { t, i18n } = useTranslation();

  const toggleLanguage = () => {
    const next = i18n.language === "ko" ? "en" : "ko";
    i18n.changeLanguage(next);
  };

  return (
    <div className="flex h-screen w-screen flex-col bg-zinc-950 text-zinc-100">
      <header className="flex h-12 items-center justify-between border-b border-zinc-800 px-4">
        <div className="flex items-center gap-2.5">
          <BookOpen className="h-5 w-5 text-amber-400" />
          <span className="font-semibold tracking-tight">{t("app.name")}</span>
          <span className="text-xs text-zinc-500">{t("app.subtitle")}</span>
        </div>
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
      </header>

      <div className="flex flex-1 overflow-hidden">
        <aside className="flex w-56 flex-col gap-1 border-r border-zinc-800 bg-zinc-900/40 p-3">
          <div className="flex items-center gap-2 rounded-md bg-zinc-800/80 px-3 py-2 text-sm font-medium text-zinc-100">
            <BookOpen className="h-4 w-4 text-amber-400" />
            {t("nav.manuscript")}
          </div>
          <div className="flex items-center gap-2 rounded-md px-3 py-2 text-sm text-zinc-400">
            <Sparkles className="h-4 w-4" />
            {t("nav.aiChat")}
          </div>
          <div className="flex items-center gap-2 rounded-md px-3 py-2 text-sm text-zinc-400">
            <Settings className="h-4 w-4" />
            {t("nav.settings")}
          </div>
        </aside>

        <main className="flex flex-1 items-center justify-center p-8">
          <div className="max-w-md text-center">
            <h1 className="text-xl font-semibold text-zinc-100">
              {t("editor.untitledScene")}
            </h1>
            <p className="mt-2 text-sm text-zinc-400">
              {t("editor.placeholder")}
            </p>
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
