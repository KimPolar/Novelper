import { Channel, invoke } from "@tauri-apps/api/core";

export type ProtocolKind =
  | "openai_chat"
  | "openai_responses"
  | "anthropic_messages"
  | "gemini_interactions"
  | "gemini_generate_content"
  | "ollama_native"
  | "openai_compat";

export interface ModelCapabilities {
  streaming: boolean;
  vision: boolean;
  toolCalling: boolean;
  structuredOutput: boolean;
  reasoning: boolean;
  promptCaching: boolean;
  contextWindow?: number;
  maxOutputTokens?: number;
}

export interface AIProviderConfig {
  id: string;
  name: string;
  protocol: ProtocolKind;
  baseUrl?: string | null;
  secretRef?: string | null;
  extraHeaders?: Record<string, string> | null;
  timeoutMs: number;
  maxRetries: number;
  isLocal: boolean;
  enabled: boolean;
}

export interface AIModelInfo {
  id: string;
  providerId: string;
  modelId: string;
  displayName: string;
  capabilities: ModelCapabilities;
  defaultOptions?: Record<string, unknown> | null;
  isCustom: boolean;
}

export interface TaskBinding {
  taskType: string;
  primaryModelId?: string | null;
  fallbackModelIds: string[];
  allowExternalFallback: boolean;
}

export interface ConnectionTestResult {
  ok: boolean;
  latencyMs: number;
  modelCount: number;
  message: string;
}

export interface AIProposal {
  id: string;
  documentId: string;
  baseRevision: number;
  targetBlockId: string;
  originalText: string;
  proposedText: string;
  rationale?: string | null;
  status: "pending" | "accepted" | "rejected" | "stale" | string;
  isStale: boolean;
  createdAt: string;
}

export interface CharacterItem {
  id: string;
  projectId: string;
  name: string;
  role?: string | null;
  profileJson?: string | null;
  updatedAt: string;
}

export interface LocationItem {
  id: string;
  projectId: string;
  name: string;
  description?: string | null;
  updatedAt: string;
}

export interface WorldbuildingItem {
  id: string;
  projectId: string;
  category: string;
  title: string;
  content: string;
  updatedAt: string;
}

export interface TimelineEventItem {
  id: string;
  projectId: string;
  documentId?: string | null;
  sequenceIndex: number;
  inStoryTime?: string | null;
  title: string;
  summary?: string | null;
}

export interface FactItem {
  id: string;
  projectId: string;
  subjectType: string;
  subjectId: string;
  factText: string;
  certainty: "confirmed" | "inferred" | "conflict" | string;
  evidenceDocId?: string | null;
  evidenceBlockId?: string | null;
  evidenceQuote?: string | null;
  createdAt: string;
}

export interface BuiltContext {
  contextText: string;
  estimatedTokens: number;
  includedSections: string[];
}

export type AIEvent =
  | { type: "text_delta"; delta: string }
  | { type: "reasoning_delta"; delta: string }
  | { type: "tool_call"; id: string; name: string; arguments_json: string }
  | { type: "usage"; input_tokens: number; output_tokens: number }
  | { type: "done"; finish_reason: string; provider_meta: unknown }
  | { type: "error"; code: string; message: string; retryable: boolean };

export interface ProjectSummary {
  id: string;
  title: string;
  language: string;
  genre?: string | null;
  synopsis?: string | null;
  path: string;
  updatedAt: string;
}

export interface ProjectDetail {
  id: string;
  title: string;
  language: string;
  genre?: string | null;
  synopsis?: string | null;
  styleGuideJson?: string | null;
  dailyGoalCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentTreeItem {
  id: string;
  projectId: string;
  parentId?: string | null;
  docType: "part" | "chapter" | "scene" | "note" | string;
  title: string;
  sortOrder: number;
  status: "draft" | "writing" | "revising" | "done" | string;
  revision: number;
  charsWithSpace: number;
  charsNoSpace: number;
  wordCount: number;
  isTrashed: boolean;
  updatedAt: string;
}

export interface DocumentDetail extends DocumentTreeItem {
  povCharacterId?: string | null;
  locationId?: string | null;
  timelineLabel?: string | null;
  contentJson: string;
  plainText: string;
}

export interface FtsSearchHit {
  docId: string;
  title: string;
  snippet: string;
}

export interface DocumentVersionItem {
  id: string;
  documentId: string;
  revision: number;
  snapshotType: string;
  note?: string | null;
  plainText: string;
  createdAt: string;
}

export interface DiffLine {
  kind: "equal" | "insert" | "delete" | string;
  text: string;
}

export interface ProjectStats {
  totalCharsWithSpace: number;
  totalCharsNoSpace: number;
  totalWordCount: number;
  sceneCount: number;
}

export const tauriApi = {
  listProjects: () => invoke<ProjectSummary[]>("list_projects"),
  createProject: (input: {
    title: string;
    language: string;
    genre?: string;
    synopsis?: string;
  }) => invoke<ProjectDetail>("create_project", { input }),
  openProject: (dbPath: string) =>
    invoke<ProjectDetail>("open_project", { dbPath }),
  listDocuments: () => invoke<DocumentTreeItem[]>("list_documents"),
  getDocument: (docId: string) =>
    invoke<DocumentDetail>("get_document", { docId }),
  createDocument: (input: {
    parentId?: string | null;
    docType: string;
    title: string;
  }) => invoke<DocumentDetail>("create_document", { input }),
  updateDocument: (input: {
    id: string;
    title?: string;
    status?: string;
    contentJson?: string;
    plainText?: string;
    expectedRevision?: number;
  }) => invoke<DocumentDetail>("update_document", { input }),
  reorderDocuments: (
    items: Array<{ id: string; parentId?: string | null; sortOrder: number }>
  ) => invoke<void>("reorder_documents", { items }),
  setDocumentTrashed: (docId: string, trashed: boolean) =>
    invoke<void>("set_document_trashed", { docId, trashed }),
  searchDocumentsFts: (query: string) =>
    invoke<FtsSearchHit[]>("search_documents_fts", { query }),
  createDocumentVersion: (
    docId: string,
    snapshotType = "manual",
    note?: string
  ) =>
    invoke<DocumentVersionItem>("create_document_version", {
      docId,
      snapshotType,
      note,
    }),
  listDocumentVersions: (docId: string) =>
    invoke<DocumentVersionItem[]>("list_document_versions", { docId }),
  restoreDocumentVersion: (versionId: string) =>
    invoke<DocumentDetail>("restore_document_version", { versionId }),
  computeLineDiff: (oldText: string, newText: string) =>
    invoke<DiffLine[]>("compute_line_diff", { oldText, newText }),
  getProjectStats: () => invoke<ProjectStats>("get_project_stats"),

  // AI Provider & Model Management
  listAiProviders: () => invoke<AIProviderConfig[]>("list_ai_providers"),
  saveAiProvider: (input: {
    id?: string;
    name: string;
    protocol: ProtocolKind;
    baseUrl?: string;
    apiKey?: string;
    timeoutMs?: number;
    maxRetries?: number;
    isLocal: boolean;
    enabled: boolean;
  }) => invoke<AIProviderConfig>("save_ai_provider", { input }),
  deleteAiProvider: (providerId: string) =>
    invoke<void>("delete_ai_provider", { providerId }),
  testAiProviderConnection: (providerId: string) =>
    invoke<ConnectionTestResult>("test_ai_provider_connection", { providerId }),
  syncProviderModels: (providerId: string) =>
    invoke<AIModelInfo[]>("sync_provider_models", { providerId }),
  addCustomModel: (
    providerId: string,
    modelId: string,
    displayName: string,
    capabilities?: ModelCapabilities
  ) =>
    invoke<AIModelInfo>("add_custom_model", {
      providerId,
      modelId,
      displayName,
      capabilities,
    }),
  listAiModels: (providerId?: string) =>
    invoke<AIModelInfo[]>("list_ai_models", { providerId }),
  deleteAiModel: (modelId: string) =>
    invoke<void>("delete_ai_model", { modelId }),
  listTaskBindings: () => invoke<TaskBinding[]>("list_task_bindings"),
  saveTaskBinding: (binding: TaskBinding) =>
    invoke<void>("save_task_binding", { binding }),
  streamAiChat: (
    params: {
      requestId: string;
      modelRecordId: string;
      taskType: string;
      request: {
        modelId: string;
        system?: string;
        messages: Array<{ role: string; content: string }>;
        temperature?: number;
        maxTokens?: number;
        tools: unknown[];
        providerOptions?: Record<string, unknown>;
      };
    },
    onEvent: (ev: AIEvent) => void
  ) => {
    const channel = new Channel<AIEvent>();
    channel.onmessage = onEvent;
    return invoke<void>("stream_ai_chat", { ...params, onEvent: channel });
  },
  cancelAiRequest: (requestId: string) =>
    invoke<boolean>("cancel_ai_request", { requestId }),
  createAiProposal: (params: {
    docId: string;
    targetBlockId: string;
    originalText: string;
    proposedText: string;
    rationale?: string;
  }) => invoke<AIProposal>("create_ai_proposal", params),
  listAiProposals: (docId: string) =>
    invoke<AIProposal[]>("list_ai_proposals", { docId }),
  applyAiProposal: (proposalId: string) =>
    invoke<DocumentDetail>("apply_ai_proposal", { proposalId }),
  rejectAiProposal: (proposalId: string) =>
    invoke<void>("reject_ai_proposal", { proposalId }),

  // Worldbuilding & Context Engine
  listCharacters: () => invoke<CharacterItem[]>("list_characters"),
  upsertCharacter: (params: {
    id?: string;
    name: string;
    role?: string;
    profileJson?: string;
  }) => invoke<CharacterItem>("upsert_character", params),
  listLocations: () => invoke<LocationItem[]>("list_locations"),
  upsertLocation: (params: {
    id?: string;
    name: string;
    description?: string;
  }) => invoke<LocationItem>("upsert_location", params),
  listWorldbuilding: () => invoke<WorldbuildingItem[]>("list_worldbuilding"),
  upsertWorldbuilding: (params: {
    id?: string;
    category: string;
    title: string;
    content: string;
  }) => invoke<WorldbuildingItem>("upsert_worldbuilding", params),
  listTimelineEvents: () => invoke<TimelineEventItem[]>("list_timeline_events"),
  upsertTimelineEvent: (params: {
    id?: string;
    documentId?: string;
    sequenceIndex: number;
    inStoryTime?: string;
    title: string;
    summary?: string;
  }) => invoke<TimelineEventItem>("upsert_timeline_event", params),
  listFacts: () => invoke<FactItem[]>("list_facts"),
  addFact: (params: {
    subjectType: string;
    subjectId: string;
    factText: string;
    certainty: string;
    evidenceDocId?: string;
    evidenceBlockId?: string;
    evidenceQuote?: string;
  }) => invoke<FactItem>("add_fact", params),
  buildSceneContext: (params: {
    docId?: string;
    query?: string;
    tokenBudget?: number;
  }) => invoke<BuiltContext>("build_scene_context_cmd", params),

  // Export & Backup
  exportManuscript: (format: "docx" | "epub" | "html" | "markdown") =>
    invoke<string>("export_manuscript", { format }),
  backupActiveProject: () => invoke<string>("backup_active_project"),
};
