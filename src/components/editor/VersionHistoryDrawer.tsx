import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { History, RotateCcw, Save, X } from "lucide-react";
import { DiffLine, DocumentVersionItem, tauriApi } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

interface VersionHistoryDrawerProps {
  open: boolean;
  onClose: () => void;
}

export function VersionHistoryDrawer({
  open,
  onClose,
}: VersionHistoryDrawerProps) {
  const { t } = useTranslation();
  const { activeDocument, selectDocument, refreshDocuments } = useProjectStore();
  const [versions, setVersions] = useState<DocumentVersionItem[]>([]);
  const [selectedVersion, setSelectedVersion] =
    useState<DocumentVersionItem | null>(null);
  const [diffLines, setDiffLines] = useState<DiffLine[]>([]);
  const [note, setNote] = useState("");

  const loadVersions = async (docId: string) => {
    const list = await tauriApi.listDocumentVersions(docId);
    setVersions(list);
    if (list.length > 0 && !selectedVersion) {
      setSelectedVersion(list[0]);
    }
  };

  useEffect(() => {
    if (!open || !activeDocument) return;
    setSelectedVersion(null);
    loadVersions(activeDocument.id);
  }, [open, activeDocument?.id]);

  useEffect(() => {
    if (!selectedVersion || !activeDocument) {
      setDiffLines([]);
      return;
    }
    tauriApi
      .computeLineDiff(selectedVersion.plainText, activeDocument.plainText)
      .then(setDiffLines);
  }, [selectedVersion?.id, activeDocument?.plainText]);

  if (!open || !activeDocument) return null;

  const handleSaveSnapshot = async (e: React.FormEvent) => {
    e.preventDefault();
    await tauriApi.createDocumentVersion(
      activeDocument.id,
      "manual",
      note.trim() || undefined
    );
    setNote("");
    await loadVersions(activeDocument.id);
  };

  const handleRestore = async (versionId: string) => {
    await tauriApi.restoreDocumentVersion(versionId);
    await refreshDocuments();
    await selectDocument(activeDocument.id);
    await loadVersions(activeDocument.id);
  };

  return (
    <aside className="flex w-96 shrink-0 flex-col border-l border-zinc-800 bg-zinc-900/60">
      <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
          <History className="h-4 w-4 text-amber-400" />
          <span>{t("versions.title")}</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <form
        onSubmit={handleSaveSnapshot}
        className="flex gap-2 border-b border-zinc-800 p-3"
      >
        <input
          type="text"
          value={note}
          onChange={(e) => setNote(e.target.value)}
          placeholder={t("versions.notePlaceholder")}
          className="flex-1 rounded border border-zinc-800 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
        />
        <button
          type="submit"
          className="flex items-center gap-1 rounded bg-amber-500 px-2.5 py-1.5 text-xs font-medium text-zinc-950 hover:bg-amber-400"
        >
          <Save className="h-3.5 w-3.5" />
          {t("versions.createSnapshot")}
        </button>
      </form>

      <div className="max-h-56 overflow-y-auto border-b border-zinc-800 p-2 space-y-1">
        {versions.length === 0 ? (
          <div className="py-6 text-center text-xs text-zinc-500">
            {t("versions.empty")}
          </div>
        ) : (
          versions.map((v) => (
            <div
              key={v.id}
              onClick={() => setSelectedVersion(v)}
              className={cn(
                "flex cursor-pointer items-center justify-between rounded-md px-2.5 py-2 text-xs transition-colors",
                selectedVersion?.id === v.id
                  ? "bg-amber-500/15 text-amber-300"
                  : "text-zinc-300 hover:bg-zinc-800/70"
              )}
            >
              <div className="min-w-0">
                <div className="font-medium">
                  rev.{v.revision} · {v.snapshotType}
                </div>
                {v.note && (
                  <div className="truncate text-zinc-400">{v.note}</div>
                )}
              </div>
              <button
                type="button"
                title={t("versions.restore")}
                onClick={(e) => {
                  e.stopPropagation();
                  handleRestore(v.id);
                }}
                className="rounded p-1 text-zinc-400 hover:bg-zinc-700 hover:text-emerald-400"
              >
                <RotateCcw className="h-3.5 w-3.5" />
              </button>
            </div>
          ))
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        <div className="mb-2 text-xs font-medium text-zinc-400">
          {t("versions.diffTitle")}
        </div>
        <div className="space-y-1 font-mono text-xs">
          {diffLines.map((line, idx) => (
            <div
              key={idx}
              className={cn(
                "rounded px-2 py-0.5 whitespace-pre-wrap",
                line.kind === "insert" && "bg-emerald-500/15 text-emerald-300",
                line.kind === "delete" && "bg-red-500/15 text-red-300 line-through",
                line.kind === "equal" && "text-zinc-400"
              )}
            >
              {line.kind === "insert" ? "+ " : line.kind === "delete" ? "- " : "  "}
              {line.text}
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
}
