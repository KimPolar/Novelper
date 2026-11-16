import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";
import { FtsSearchHit, tauriApi } from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";

interface SearchModalProps {
  open: boolean;
  onClose: () => void;
}

export function SearchModal({ open, onClose }: SearchModalProps) {
  const { t } = useTranslation();
  const { selectDocument } = useProjectStore();
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<FtsSearchHit[]>([]);

  useEffect(() => {
    if (!open) return;
    const trimmed = query.trim();
    if (!trimmed) {
      setHits([]);
      return;
    }
    const timer = window.setTimeout(async () => {
      try {
        const res = await tauriApi.searchDocumentsFts(trimmed);
        setHits(res);
      } catch {
        setHits([]);
      }
    }, 180);
    return () => window.clearTimeout(timer);
  }, [query, open]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-24"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg rounded-xl border border-zinc-800 bg-zinc-900 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b border-zinc-800 px-4 py-3">
          <Search className="h-4 w-4 text-zinc-400" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("search.placeholder")}
            className="flex-1 bg-transparent text-sm text-zinc-100 focus:outline-none"
            autoFocus
          />
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="max-h-80 overflow-y-auto p-2">
          {hits.length === 0 ? (
            <div className="py-8 text-center text-xs text-zinc-500">
              {t("search.noResults")}
            </div>
          ) : (
            hits.map((hit) => (
              <button
                key={hit.docId}
                type="button"
                onClick={async () => {
                  await selectDocument(hit.docId);
                  onClose();
                }}
                className="w-full rounded-lg px-3 py-2 text-left hover:bg-zinc-800/80 transition-colors"
              >
                <div className="text-sm font-medium text-amber-300">
                  {hit.title}
                </div>
                <div className="mt-0.5 line-clamp-2 text-xs text-zinc-400">
                  {hit.snippet}
                </div>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
