import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Archive, Download, FileText, X } from "lucide-react";
import { tauriApi } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";

interface ExportModalProps {
  open: boolean;
  onClose: () => void;
}

export function ExportModal({ open, onClose }: ExportModalProps) {
  const { t } = useTranslation();
  const { activeProject } = useProjectStore();
  const [resultPath, setResultPath] = useState<string | null>(null);

  if (!open || !activeProject) return null;

  const handleExport = async (fmt: "docx" | "epub" | "html" | "markdown") => {
    const path = await tauriApi.exportManuscript(fmt);
    setResultPath(path);
  };

  const handleBackup = async () => {
    const path = await tauriApi.backupActiveProject();
    setResultPath(path);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-xl border border-zinc-800 bg-zinc-900 p-5 shadow-2xl space-y-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
          <div className="flex items-center gap-2 font-semibold text-zinc-100">
            <Download className="h-4 w-4 text-amber-400" />
            <span>{t("export.title")}</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="grid grid-cols-2 gap-2">
          {(["docx", "epub", "html", "markdown"] as const).map((fmt) => (
            <button
              key={fmt}
              type="button"
              onClick={() => handleExport(fmt)}
              className="flex items-center justify-center gap-1.5 rounded-lg border border-zinc-800 bg-zinc-950 py-2.5 text-xs font-medium text-zinc-200 hover:border-amber-500/40 hover:text-amber-300"
            >
              <FileText className="h-3.5 w-3.5" />
              <span>{fmt === "html" ? "PDF / Print HTML" : fmt.toUpperCase()}</span>
            </button>
          ))}
        </div>

        <button
          type="button"
          onClick={handleBackup}
          className="flex w-full items-center justify-center gap-2 rounded-lg bg-amber-500 py-2 text-xs font-medium text-zinc-950 hover:bg-amber-400"
        >
          <Archive className="h-3.5 w-3.5" />
          <span>{t("export.backupNow")}</span>
        </button>

        {resultPath && (
          <div className="rounded bg-emerald-500/15 px-3 py-2 text-xs text-emerald-300 break-all">
            {t("export.done")}
            {resultPath}
          </div>
        )}
      </div>
    </div>
  );
}
