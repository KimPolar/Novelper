import { create } from "zustand";
import {
  AIModelInfo,
  AIProviderConfig,
  ConnectionTestResult,
  ProtocolKind,
  TaskBinding,
  tauriApi,
} from "@/lib/tauri";

interface AIState {
  providers: AIProviderConfig[];
  models: AIModelInfo[];
  taskBindings: TaskBinding[];
  localOnlyMode: boolean;

  loadAiConfig: () => Promise<void>;
  saveProvider: (input: {
    id?: string;
    name: string;
    protocol: ProtocolKind;
    baseUrl?: string;
    apiKey?: string;
    isLocal: boolean;
    enabled: boolean;
  }) => Promise<AIProviderConfig>;
  deleteProvider: (providerId: string) => Promise<void>;
  testConnection: (providerId: string) => Promise<ConnectionTestResult>;
  syncModels: (providerId: string) => Promise<void>;
  addCustomModel: (
    providerId: string,
    modelId: string,
    displayName: string
  ) => Promise<void>;
  saveTaskBinding: (binding: TaskBinding) => Promise<void>;
  setLocalOnlyMode: (enabled: boolean) => void;
}

export const useAiStore = create<AIState>((set, get) => ({
  providers: [],
  models: [],
  taskBindings: [],
  localOnlyMode: false,

  loadAiConfig: async () => {
    const [providers, models, taskBindings] = await Promise.all([
      tauriApi.listAiProviders(),
      tauriApi.listAiModels(),
      tauriApi.listTaskBindings(),
    ]);
    set({ providers, models, taskBindings });
  },

  saveProvider: async (input) => {
    const saved = await tauriApi.saveAiProvider(input);
    await get().loadAiConfig();
    return saved;
  },

  deleteProvider: async (providerId) => {
    await tauriApi.deleteAiProvider(providerId);
    await get().loadAiConfig();
  },

  testConnection: (providerId) =>
    tauriApi.testAiProviderConnection(providerId),

  syncModels: async (providerId) => {
    await tauriApi.syncProviderModels(providerId);
    await get().loadAiConfig();
  },

  addCustomModel: async (providerId, modelId, displayName) => {
    await tauriApi.addCustomModel(providerId, modelId, displayName);
    await get().loadAiConfig();
  },

  saveTaskBinding: async (binding) => {
    await tauriApi.saveTaskBinding(binding);
    const taskBindings = await tauriApi.listTaskBindings();
    set({ taskBindings });
  },

  setLocalOnlyMode: (enabled) => set({ localOnlyMode: enabled }),
}));
