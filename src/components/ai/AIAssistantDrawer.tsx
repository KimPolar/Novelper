import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertTriangle,
  Check,
  FileDiff,
  Send,
  ShieldAlert,
  Sparkles,
  Square,
  X,
} from "lucide-react";
import { AIProposal, tauriApi } from "@/lib/tauri";
import { useAiStore } from "@/stores/aiStore";
import { useProjectStore } from "@/stores/projectStore";
import { cn } from "@/lib/utils";

interface AIAssistantDrawerProps {
  open: boolean;
  onClose: () => void;
}

interface ChatMsg {
  role: "user" | "assistant";
  content: string;
}

export function AIAssistantDrawer({ open, onClose }: AIAssistantDrawerProps) {
  const { t } = useTranslation();
  const { activeProject, activeDocument, selectDocument, refreshDocuments } =
    useProjectStore();
  const { providers, models, taskBindings, localOnlyMode, loadAiConfig } =
    useAiStore();

  const [tab, setTab] = useState<"chat" | "proposals">("chat");
  const [selectedModelId, setSelectedModelId] = useState("");
  const [prompt, setPrompt] = useState("");
  const [messages, setMessages] = useState<ChatMsg[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [proposals, setProposals] = useState<AIProposal[]>([]);

  useEffect(() => {
    if (!open) return;
    loadAiConfig();
  }, [open, loadAiConfig]);

  useEffect(() => {
    if (!selectedModelId && models.length > 0) {
      const chatBinding = taskBindings.find((b) => b.taskType === "chat");
      setSelectedModelId(chatBinding?.primaryModelId || models[0].id);
    }
  }, [models, taskBindings, selectedModelId]);

  const loadProposals = async (docId: string) => {
    const list = await tauriApi.listAiProposals(docId);
    setProposals(list);
  };

  useEffect(() => {
    if (open && activeDocument) {
      loadProposals(activeDocument.id);
    }
  }, [open, activeDocument?.id, activeDocument?.revision]);

  if (!open) return null;

  const selectedModel = models.find((m) => m.id === selectedModelId);
  const selectedProvider = providers.find(
    (p) => p.id === selectedModel?.providerId
  );
  const isCloudProvider = selectedProvider ? !selectedProvider.isLocal : false;

  const handleSend = async (customPrompt?: string) => {
    const text = (customPrompt ?? prompt).trim();
    if (!text || !selectedModel) return;
    setErrorMsg(null);

    if (localOnlyMode && isCloudProvider) {
      setErrorMsg(t("aiPanel.localBlocked"));
      return;
    }

    const userMsg: ChatMsg = { role: "user", content: text };
    const nextMessages = [...messages, userMsg, { role: "assistant" as const, content: "" }];
    setMessages(nextMessages);
    if (!customPrompt) setPrompt("");

    const reqId = crypto.randomUUID();
    setActiveRequestId(reqId);
    setIsStreaming(true);

    const projectLang = activeProject?.language === "en" ? "English" : "Korean";
    const sceneContext = activeDocument
      ? `\n\n[Current Scene: ${activeDocument.title}]\n${activeDocument.plainText}`
      : "";

    try {
      await tauriApi.streamAiChat(
        {
          requestId: reqId,
          modelRecordId: selectedModel.id,
          taskType: "chat",
          request: {
            modelId: selectedModel.modelId,
            system: `You are Novelper, an expert novel writing assistant. Respond in ${projectLang}.${sceneContext}`,
            messages: [...messages, userMsg],
            tools: [],
          },
        },
        (ev) => {
          if (ev.type === "text_delta") {
            setMessages((prev) => {
              const copy = [...prev];
              const last = copy[copy.length - 1];
              if (last && last.role === "assistant") {
                copy[copy.length - 1] = {
                  ...last,
                  content: last.content + ev.delta,
                };
              }
              return copy;
            });
          } else if (ev.type === "error") {
            setErrorMsg(ev.message);
          }
        }
      );
    } catch (err) {
      setErrorMsg(String(err));
    } finally {
      setIsStreaming(false);
      setActiveRequestId(null);
    }
  };

  const handleStop = async () => {
    if (activeRequestId) {
      await tauriApi.cancelAiRequest(activeRequestId);
    }
  };

  const handleSaveAsProposal = async (proposedText: string) => {
    if (!activeDocument) return;
    let firstBlockId = "";
    let firstOriginal = activeDocument.plainText.split("\n")[0] ?? "";
    try {
      const parsed = JSON.parse(activeDocument.contentJson);
      const firstNode = parsed?.content?.[0];
      if (firstNode?.attrs?.blockId) {
        firstBlockId = firstNode.attrs.blockId;
      }
    } catch {
      // ignore
    }

    await tauriApi.createAiProposal({
      docId: activeDocument.id,
      targetBlockId: firstBlockId,
      originalText: firstOriginal,
      proposedText: proposedText.trim(),
      rationale: "AI Assistant suggestion",
    });
    await loadProposals(activeDocument.id);
    setTab("proposals");
  };

  const handleApplyProposal = async (proposalId: string) => {
    try {
      await tauriApi.applyAiProposal(proposalId);
      if (activeDocument) {
        await refreshDocuments();
        await selectDocument(activeDocument.id);
        await loadProposals(activeDocument.id);
      }
    } catch (err) {
      setErrorMsg(String(err));
      if (activeDocument) await loadProposals(activeDocument.id);
    }
  };

  const handleRejectProposal = async (proposalId: string) => {
    await tauriApi.rejectAiProposal(proposalId);
    if (activeDocument) await loadProposals(activeDocument.id);
  };

  return (
    <aside className="flex w-96 shrink-0 flex-col border-l border-zinc-800 bg-zinc-900/60">
      <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
          <Sparkles className="h-4 w-4 text-amber-400" />
          <span>{t("aiPanel.button")}</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="flex border-b border-zinc-800 text-xs">
        <button
          type="button"
          onClick={() => setTab("chat")}
          className={cn(
            "flex-1 py-2 font-medium transition-colors",
            tab === "chat"
              ? "border-b-2 border-amber-400 text-amber-300"
              : "text-zinc-400 hover:text-zinc-200"
          )}
        >
          {t("aiPanel.tabChat")}
        </button>
        <button
          type="button"
          onClick={() => setTab("proposals")}
          className={cn(
            "flex-1 py-2 font-medium transition-colors",
            tab === "proposals"
              ? "border-b-2 border-amber-400 text-amber-300"
              : "text-zinc-400 hover:text-zinc-200"
          )}
        >
          {t("aiPanel.tabProposals")} ({proposals.filter((p) => p.status === "pending" || p.status === "stale").length})
        </button>
      </div>

      {tab === "chat" ? (
        <>
          <div className="border-b border-zinc-800 p-3 space-y-2">
            <select
              value={selectedModelId}
              onChange={(e) => setSelectedModelId(e.target.value)}
              className="w-full rounded border border-zinc-800 bg-zinc-950 px-2.5 py-1.5 text-xs text-zinc-200"
            >
              {models.length === 0 && (
                <option value="">{t("aiPanel.noModel")}</option>
              )}
              {models.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.displayName}
                </option>
              ))}
            </select>

            {isCloudProvider && (
              <div className="flex items-start gap-1.5 rounded bg-amber-500/10 px-2.5 py-1.5 text-[11px] text-amber-300">
                <ShieldAlert className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                <span>{t("aiPanel.cloudNotice")}</span>
              </div>
            )}

            <div className="flex flex-wrap gap-1">
              {[
                { key: "quickNext", prompt: "이 장면에 이어질 다음 문단 2~3문장을 자연스럽게 제안해 주세요." },
                { key: "quickExpand", prompt: "현재 장면의 감각적 묘사와 긴장감을 확장해 주세요." },
                { key: "quickPolish", prompt: "현재 장면의 문체를 더 매끄럽고 문학적으로 다듬어 주세요." },
                { key: "quickProofread", prompt: "현재 장면의 맞춤법, 띄어쓰기, 어색한 표현을 교정해 주세요." },
              ].map((q) => (
                <button
                  key={q.key}
                  type="button"
                  disabled={isStreaming || models.length === 0}
                  onClick={() => handleSend(q.prompt)}
                  className="rounded border border-zinc-800 bg-zinc-950 px-2 py-1 text-[11px] text-zinc-300 hover:border-amber-500/40 hover:text-amber-300 disabled:opacity-40"
                >
                  {t(`aiPanel.${q.key}`)}
                </button>
              ))}
            </div>
          </div>

          {errorMsg && (
            <div className="mx-3 mt-2 rounded bg-red-500/15 px-3 py-2 text-xs text-red-300">
              {errorMsg}
            </div>
          )}

          <div className="flex-1 overflow-y-auto p-3 space-y-3">
            {messages.map((m, idx) => (
              <div
                key={idx}
                className={cn(
                  "rounded-lg p-3 text-xs whitespace-pre-wrap",
                  m.role === "user"
                    ? "ml-6 bg-zinc-800 text-zinc-100"
                    : "mr-4 border border-zinc-800 bg-zinc-950/70 text-zinc-200"
                )}
              >
                <div>{m.content}</div>
                {m.role === "assistant" && m.content && !isStreaming && activeDocument && (
                  <button
                    type="button"
                    onClick={() => handleSaveAsProposal(m.content)}
                    className="mt-2 flex items-center gap-1 text-[11px] text-amber-400 hover:text-amber-300"
                  >
                    <FileDiff className="h-3 w-3" />
                    {t("aiPanel.saveProposal")}
                  </button>
                )}
              </div>
            ))}
          </div>

          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleSend();
            }}
            className="flex gap-2 border-t border-zinc-800 p-3"
          >
            <input
              type="text"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder={t("aiPanel.promptPlaceholder")}
              className="flex-1 rounded border border-zinc-800 bg-zinc-950 px-3 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
            />
            {isStreaming ? (
              <button
                type="button"
                onClick={handleStop}
                className="flex items-center gap-1 rounded bg-red-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-red-400"
              >
                <Square className="h-3 w-3" />
                {t("aiPanel.stop")}
              </button>
            ) : (
              <button
                type="submit"
                disabled={!prompt.trim() || models.length === 0}
                className="flex items-center gap-1 rounded bg-amber-500 px-3 py-1.5 text-xs font-medium text-zinc-950 hover:bg-amber-400 disabled:opacity-40"
              >
                <Send className="h-3 w-3" />
                {t("aiPanel.send")}
              </button>
            )}
          </form>
        </>
      ) : (
        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          {proposals.length === 0 ? (
            <div className="py-8 text-center text-xs text-zinc-500">
              {t("aiPanel.noProposals")}
            </div>
          ) : (
            proposals.map((p) => (
              <div
                key={p.id}
                className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-3 space-y-2 text-xs"
              >
                <div className="flex items-center justify-between text-[11px] text-zinc-400">
                  <span>base rev.{p.baseRevision}</span>
                  <span className="uppercase">{p.status}</span>
                </div>
                {p.isStale && (
                  <div className="flex items-center gap-1.5 rounded bg-amber-500/15 px-2 py-1 text-[11px] text-amber-300">
                    <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
                    <span>{t("aiPanel.staleWarning")}</span>
                  </div>
                )}
                <div className="rounded bg-red-500/10 p-2 text-red-300 line-through">
                  {p.originalText}
                </div>
                <div className="rounded bg-emerald-500/10 p-2 text-emerald-300">
                  {p.proposedText}
                </div>
                {(p.status === "pending" || p.status === "stale") && (
                  <div className="flex justify-end gap-2 pt-1">
                    <button
                      type="button"
                      onClick={() => handleRejectProposal(p.id)}
                      className="rounded border border-zinc-700 px-2.5 py-1 text-zinc-300 hover:bg-zinc-800"
                    >
                      {t("aiPanel.reject")}
                    </button>
                    <button
                      type="button"
                      disabled={p.isStale}
                      onClick={() => handleApplyProposal(p.id)}
                      className="flex items-center gap-1 rounded bg-emerald-500 px-2.5 py-1 font-medium text-zinc-950 hover:bg-emerald-400 disabled:opacity-40"
                    >
                      <Check className="h-3 w-3" />
                      {t("aiPanel.apply")}
                    </button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}
    </aside>
  );
}
