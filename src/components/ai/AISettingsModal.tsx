import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  CheckCircle2,
  Cpu,
  Plus,
  RefreshCw,
  Shield,
  Trash2,
  X,
  XCircle,
} from "lucide-react";
import { ConnectionTestResult, ProtocolKind } from "@/lib/tauri";
import { useAiStore } from "@/stores/aiStore";

interface AISettingsModalProps {
  open: boolean;
  onClose: () => void;
}

const PROTOCOL_OPTIONS: Array<{ value: ProtocolKind; label: string; defaultUrl: string; local: boolean }> = [
  { value: "openai_responses", label: "OpenAI Responses API", defaultUrl: "https://api.openai.com/v1", local: false },
  { value: "openai_chat", label: "OpenAI Chat Completions", defaultUrl: "https://api.openai.com/v1", local: false },
  { value: "anthropic_messages", label: "Anthropic Claude Messages", defaultUrl: "https://api.anthropic.com/v1", local: false },
  { value: "gemini_interactions", label: "Google Gemini Interactions", defaultUrl: "https://generativelanguage.googleapis.com/v1beta", local: false },
  { value: "gemini_generate_content", label: "Google Gemini generateContent", defaultUrl: "https://generativelanguage.googleapis.com/v1beta", local: false },
  { value: "ollama_native", label: "Ollama Native (Local)", defaultUrl: "http://localhost:11434", local: true },
  { value: "openai_compat", label: "OpenAI-Compatible (LM Studio / OpenRouter / vLLM)", defaultUrl: "http://localhost:1234/v1", local: true },
];

const TASK_TYPES = [
  "draft",
  "proofread",
  "consistency_check",
  "embedding",
  "rerank",
  "chat",
];

export function AISettingsModal({ open, onClose }: AISettingsModalProps) {
  const { t } = useTranslation();
  const {
    providers,
    models,
    taskBindings,
    localOnlyMode,
    loadAiConfig,
    saveProvider,
    deleteProvider,
    testConnection,
    syncModels,
    addCustomModel,
    saveTaskBinding,
    setLocalOnlyMode,
  } = useAiStore();

  const [name, setName] = useState("");
  const [protocol, setProtocol] = useState<ProtocolKind>("openai_compat");
  const [baseUrl, setBaseUrl] = useState("http://localhost:1234/v1");
  const [apiKey, setApiKey] = useState("");
  const [isLocal, setIsLocal] = useState(true);
  const [testResults, setTestResults] = useState<Record<string, ConnectionTestResult>>({});
  const [customModelId, setCustomModelId] = useState<Record<string, string>>({});

  useEffect(() => {
    if (open) loadAiConfig();
  }, [open, loadAiConfig]);

  if (!open) return null;

  const handleProtocolChange = (next: ProtocolKind) => {
    setProtocol(next);
    const meta = PROTOCOL_OPTIONS.find((p) => p.value === next);
    if (meta) {
      setBaseUrl(meta.defaultUrl);
      setIsLocal(meta.local);
    }
  };

  const handleAddProvider = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    await saveProvider({
      name: name.trim(),
      protocol,
      baseUrl: baseUrl.trim() || undefined,
      apiKey: apiKey.trim() || undefined,
      isLocal,
      enabled: true,
    });
    setName("");
    setApiKey("");
  };

  const handleTest = async (providerId: string) => {
    const res = await testConnection(providerId);
    setTestResults((prev) => ({ ...prev, [providerId]: res }));
  };

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
            <Cpu className="h-4 w-4 text-amber-400" />
            <span>{t("aiSettings.title")}</span>
          </div>
          <div className="flex items-center gap-3">
            <label className="flex items-center gap-1.5 text-xs text-zinc-300 cursor-pointer">
              <input
                type="checkbox"
                checked={localOnlyMode}
                onChange={(e) => setLocalOnlyMode(e.target.checked)}
                className="accent-amber-500"
              />
              <Shield className="h-3.5 w-3.5 text-emerald-400" />
              <span>{t("aiSettings.localOnly")}</span>
            </label>
            <button
              type="button"
              onClick={onClose}
              className="rounded p-1 text-zinc-400 hover:bg-zinc-800"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-5 space-y-6">
          {/* Add Provider Form */}
          <form
            onSubmit={handleAddProvider}
            className="rounded-lg border border-zinc-800 bg-zinc-950/60 p-4 space-y-3"
          >
            <div className="text-xs font-medium text-zinc-300">
              {t("aiSettings.addProvider")}
            </div>
            <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("aiSettings.providerNamePlaceholder")}
                className="rounded border border-zinc-800 bg-zinc-900 px-3 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              />
              <select
                value={protocol}
                onChange={(e) => handleProtocolChange(e.target.value as ProtocolKind)}
                className="rounded border border-zinc-800 bg-zinc-900 px-3 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              >
                {PROTOCOL_OPTIONS.map((p) => (
                  <option key={p.value} value={p.value}>
                    {p.label}
                  </option>
                ))}
              </select>
              <input
                type="text"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="Base URL"
                className="rounded border border-zinc-800 bg-zinc-900 px-3 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              />
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={t("aiSettings.apiKeyPlaceholder")}
                className="rounded border border-zinc-800 bg-zinc-900 px-3 py-1.5 text-xs text-zinc-100 focus:border-amber-500 focus:outline-none"
              />
            </div>
            <div className="flex items-center justify-between">
              <label className="flex items-center gap-2 text-xs text-zinc-400">
                <input
                  type="checkbox"
                  checked={isLocal}
                  onChange={(e) => setIsLocal(e.target.checked)}
                  className="accent-amber-500"
                />
                {t("aiSettings.isLocal")}
              </label>
              <button
                type="submit"
                className="flex items-center gap-1 rounded bg-amber-500 px-3 py-1.5 text-xs font-medium text-zinc-950 hover:bg-amber-400"
              >
                <Plus className="h-3.5 w-3.5" />
                {t("aiSettings.saveProvider")}
              </button>
            </div>
          </form>

          {/* Configured Providers */}
          <div className="space-y-3">
            <div className="text-xs font-medium text-zinc-300">
              {t("aiSettings.configuredProviders")}
            </div>
            {providers.map((p) => {
              const pModels = models.filter((m) => m.providerId === p.id);
              const testRes = testResults[p.id];
              return (
                <div
                  key={p.id}
                  className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-4 space-y-3"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
                        <span>{p.name}</span>
                        <span className="rounded bg-zinc-800 px-1.5 py-0.5 text-[10px] text-zinc-300">
                          {p.protocol}
                        </span>
                        {p.isLocal && (
                          <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 text-[10px] text-emerald-300">
                            LOCAL
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-zinc-500">{p.baseUrl}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        type="button"
                        onClick={() => handleTest(p.id)}
                        className="rounded border border-zinc-700 px-2.5 py-1 text-xs text-zinc-300 hover:bg-zinc-800"
                      >
                        {t("aiSettings.testConnection")}
                      </button>
                      <button
                        type="button"
                        onClick={() => syncModels(p.id)}
                        className="flex items-center gap-1 rounded border border-zinc-700 px-2.5 py-1 text-xs text-zinc-300 hover:bg-zinc-800"
                      >
                        <RefreshCw className="h-3 w-3" />
                        {t("aiSettings.syncModels")}
                      </button>
                      <button
                        type="button"
                        onClick={() => deleteProvider(p.id)}
                        className="rounded p-1 text-zinc-500 hover:bg-zinc-800 hover:text-red-400"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  </div>

                  {testRes && (
                    <div
                      className={`flex items-center gap-1.5 text-xs ${
                        testRes.ok ? "text-emerald-400" : "text-red-400"
                      }`}
                    >
                      {testRes.ok ? (
                        <CheckCircle2 className="h-3.5 w-3.5" />
                      ) : (
                        <XCircle className="h-3.5 w-3.5" />
                      )}
                      <span>
                        {testRes.message} ({testRes.latencyMs}ms)
                      </span>
                    </div>
                  )}

                  <div className="flex flex-wrap gap-1.5">
                    {pModels.map((m) => (
                      <span
                        key={m.id}
                        className="rounded border border-zinc-800 bg-zinc-900 px-2 py-0.5 text-xs text-zinc-300"
                      >
                        {m.displayName}
                      </span>
                    ))}
                  </div>

                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={customModelId[p.id] ?? ""}
                      onChange={(e) =>
                        setCustomModelId((prev) => ({
                          ...prev,
                          [p.id]: e.target.value,
                        }))
                      }
                      placeholder={t("aiSettings.customModelPlaceholder")}
                      className="flex-1 rounded border border-zinc-800 bg-zinc-900 px-2.5 py-1 text-xs text-zinc-100"
                    />
                    <button
                      type="button"
                      onClick={async () => {
                        const mid = (customModelId[p.id] ?? "").trim();
                        if (!mid) return;
                        await addCustomModel(p.id, mid, mid);
                        setCustomModelId((prev) => ({ ...prev, [p.id]: "" }));
                      }}
                      className="rounded border border-zinc-700 px-2.5 py-1 text-xs text-zinc-300 hover:bg-zinc-800"
                    >
                      {t("aiSettings.addCustomModel")}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Task Routing */}
          <div className="space-y-2">
            <div className="text-xs font-medium text-zinc-300">
              {t("aiSettings.taskRouting")}
            </div>
            <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
              {TASK_TYPES.map((taskType) => {
                const binding = taskBindings.find((b) => b.taskType === taskType);
                return (
                  <div
                    key={taskType}
                    className="flex items-center justify-between rounded border border-zinc-800 bg-zinc-950/40 px-3 py-2 text-xs"
                  >
                    <span className="font-medium text-zinc-300">
                      {t(`aiSettings.tasks.${taskType}`)}
                    </span>
                    <select
                      value={binding?.primaryModelId ?? ""}
                      onChange={(e) =>
                        saveTaskBinding({
                          taskType,
                          primaryModelId: e.target.value || null,
                          fallbackModelIds: binding?.fallbackModelIds ?? [],
                          allowExternalFallback: binding?.allowExternalFallback ?? false,
                        })
                      }
                      className="max-w-48 rounded border border-zinc-800 bg-zinc-900 px-2 py-1 text-xs text-zinc-200"
                    >
                      <option value="">--</option>
                      {models.map((m) => (
                        <option key={m.id} value={m.id}>
                          {m.displayName}
                        </option>
                      ))}
                    </select>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
