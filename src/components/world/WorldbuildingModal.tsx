import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  BookMarked,
  Clock,
  ExternalLink,
  Globe2,
  MapPin,
  Plus,
  ShieldCheck,
  Users,
  X,
} from "lucide-react";
import {
  CharacterItem,
  FactItem,
  LocationItem,
  TimelineEventItem,
  WorldbuildingItem,
  tauriApi,
} from "@/lib/tauri";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

interface WorldbuildingModalProps {
  open: boolean;
  onClose: () => void;
}

type WorldTab = "characters" | "locations" | "worldbuilding" | "timeline" | "facts";

export function WorldbuildingModal({ open, onClose }: WorldbuildingModalProps) {
  const { t } = useTranslation();
  const { activeProject, selectDocument } = useProjectStore();
  const [tab, setTab] = useState<WorldTab>("characters");

  const [characters, setCharacters] = useState<CharacterItem[]>([]);
  const [locations, setLocations] = useState<LocationItem[]>([]);
  const [worldItems, setWorldItems] = useState<WorldbuildingItem[]>([]);
  const [timeline, setTimeline] = useState<TimelineEventItem[]>([]);
  const [facts, setFacts] = useState<FactItem[]>([]);

  const [field1, setField1] = useState("");
  const [field2, setField2] = useState("");
  const [field3, setField3] = useState("");

  const loadAll = async () => {
    if (!activeProject) return;
    const [c, l, w, tl, f] = await Promise.all([
      tauriApi.listCharacters(),
      tauriApi.listLocations(),
      tauriApi.listWorldbuilding(),
      tauriApi.listTimelineEvents(),
      tauriApi.listFacts(),
    ]);
    setCharacters(c);
    setLocations(l);
    setWorldItems(w);
    setTimeline(tl);
    setFacts(f);
  };

  useEffect(() => {
    if (open && activeProject) loadAll();
  }, [open, activeProject?.id]);

  if (!open || !activeProject) return null;

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!field1.trim()) return;
    if (tab === "characters") {
      await tauriApi.upsertCharacter({
        name: field1.trim(),
        role: field2.trim() || undefined,
        profileJson: field3.trim() || undefined,
      });
    } else if (tab === "locations") {
      await tauriApi.upsertLocation({
        name: field1.trim(),
        description: field2.trim() || undefined,
      });
    } else if (tab === "worldbuilding") {
      await tauriApi.upsertWorldbuilding({
        category: field2.trim() || "rule",
        title: field1.trim(),
        content: field3.trim(),
      });
    } else if (tab === "timeline") {
      await tauriApi.upsertTimelineEvent({
        sequenceIndex: timeline.length + 1,
        inStoryTime: field2.trim() || undefined,
        title: field1.trim(),
        summary: field3.trim() || undefined,
      });
    } else if (tab === "facts") {
      await tauriApi.addFact({
        subjectType: "world",
        subjectId: activeProject.id,
        factText: field1.trim(),
        certainty: field2.trim() || "confirmed",
        evidenceQuote: field3.trim() || undefined,
      });
    }
    setField1("");
    setField2("");
    setField3("");
    await loadAll();
  };

  const tabs: Array<{ id: WorldTab; icon: typeof Users; label: string }> = [
    { id: "characters", icon: Users, label: t("world.characters") },
    { id: "locations", icon: MapPin, label: t("world.locations") },
    { id: "worldbuilding", icon: Globe2, label: t("world.worldbuilding") },
    { id: "timeline", icon: Clock, label: t("world.timeline") },
    { id: "facts", icon: ShieldCheck, label: t("world.facts") },
  ];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      onClick={onClose}
    >
      <div
        className="flex max-h-[88vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-800 px-5 py-3.5">
          <div className="flex items-center gap-2 font-semibold text-zinc-100">
            <BookMarked className="h-4 w-4 text-amber-400" />
            <span>{t("world.title")}</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="flex border-b border-zinc-800 bg-zinc-950/40 px-4">
          {tabs.map((tb) => {
            const Icon = tb.icon;
            return (
              <button
                key={tb.id}
                type="button"
                onClick={() => {
                  setTab(tb.id);
                  setField1("");
                  setField2("");
                  setField3("");
                }}
                className={cn(
                  "flex items-center gap-1.5 px-3 py-2.5 text-xs font-medium transition-colors",
                  tab === tb.id
                    ? "border-b-2 border-amber-400 text-amber-300"
                    : "text-zinc-400 hover:text-zinc-200"
                )}
              >
                <Icon className="h-3.5 w-3.5" />
                <span>{tb.label}</span>
              </button>
            );
          })}
        </div>

        <div className="flex-1 overflow-y-auto p-5 space-y-4">
          <form
            onSubmit={handleAdd}
            className="grid grid-cols-1 gap-2 rounded-lg border border-zinc-800 bg-zinc-950/60 p-3 md:grid-cols-4"
          >
            <input
              type="text"
              value={field1}
              onChange={(e) => setField1(e.target.value)}
              placeholder={
                tab === "characters"
                  ? "이름"
                  : tab === "locations"
                  ? "장소명"
                  : tab === "facts"
                  ? "확정 설정 / 사실 내용"
                  : "제목"
              }
              className="rounded border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100"
            />
            <input
              type="text"
              value={field2}
              onChange={(e) => setField2(e.target.value)}
              placeholder={
                tab === "characters"
                  ? "역할 (주인공/조연)"
                  : tab === "worldbuilding"
                  ? "분류 (history/magic/rule)"
                  : tab === "timeline"
                  ? "작중 시점"
                  : tab === "facts"
                  ? "confirmed | inferred | conflict"
                  : "설명"
              }
              className="rounded border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100"
            />
            <input
              type="text"
              value={field3}
              onChange={(e) => setField3(e.target.value)}
              placeholder="상세 내용 / 근거 인용"
              className="rounded border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100"
            />
            <button
              type="submit"
              className="flex items-center justify-center gap-1 rounded bg-amber-500 px-3 py-1.5 text-xs font-medium text-zinc-950 hover:bg-amber-400"
            >
              <Plus className="h-3.5 w-3.5" />
              {t("world.add")}
            </button>
          </form>

          <div className="space-y-2 text-xs">
            {tab === "characters" &&
              characters.map((c) => (
                <div
                  key={c.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3"
                >
                  <div className="font-medium text-zinc-100">
                    {c.name}{" "}
                    {c.role && (
                      <span className="text-amber-400">({c.role})</span>
                    )}
                  </div>
                  {c.profileJson && (
                    <div className="mt-1 text-zinc-400">{c.profileJson}</div>
                  )}
                </div>
              ))}

            {tab === "locations" &&
              locations.map((l) => (
                <div
                  key={l.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3"
                >
                  <div className="font-medium text-zinc-100">{l.name}</div>
                  {l.description && (
                    <div className="mt-1 text-zinc-400">{l.description}</div>
                  )}
                </div>
              ))}

            {tab === "worldbuilding" &&
              worldItems.map((w) => (
                <div
                  key={w.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3"
                >
                  <div className="flex items-center gap-2 font-medium text-zinc-100">
                    <span className="rounded bg-zinc-800 px-1.5 py-0.5 text-[10px] text-amber-300">
                      {w.category}
                    </span>
                    <span>{w.title}</span>
                  </div>
                  <div className="mt-1 text-zinc-400">{w.content}</div>
                </div>
              ))}

            {tab === "timeline" &&
              timeline.map((ev) => (
                <div
                  key={ev.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3"
                >
                  <div className="font-medium text-zinc-100">
                    #{ev.sequenceIndex} {ev.title}{" "}
                    {ev.inStoryTime && (
                      <span className="text-zinc-400">[{ev.inStoryTime}]</span>
                    )}
                  </div>
                  {ev.summary && (
                    <div className="mt-1 text-zinc-400">{ev.summary}</div>
                  )}
                </div>
              ))}

            {tab === "facts" &&
              facts.map((f) => (
                <div
                  key={f.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3 space-y-1"
                >
                  <div className="flex items-center justify-between">
                    <span
                      className={cn(
                        "rounded px-1.5 py-0.5 text-[10px] font-medium uppercase",
                        f.certainty === "confirmed" &&
                          "bg-emerald-500/15 text-emerald-300",
                        f.certainty === "inferred" &&
                          "bg-sky-500/15 text-sky-300",
                        f.certainty === "conflict" &&
                          "bg-red-500/15 text-red-300"
                      )}
                    >
                      {f.certainty}
                    </span>
                    {f.evidenceDocId && (
                      <button
                        type="button"
                        onClick={async () => {
                          await selectDocument(f.evidenceDocId!);
                          onClose();
                        }}
                        className="flex items-center gap-1 text-[11px] text-amber-400 hover:text-amber-300"
                      >
                        <ExternalLink className="h-3 w-3" />
                        {t("world.jumpToEvidence")}
                      </button>
                    )}
                  </div>
                  <div className="text-zinc-100">{f.factText}</div>
                  {f.evidenceQuote && (
                    <div className="italic text-zinc-400">
                      &ldquo;{f.evidenceQuote}&rdquo;
                    </div>
                  )}
                </div>
              ))}
          </div>
        </div>
      </div>
    </div>
  );
}
