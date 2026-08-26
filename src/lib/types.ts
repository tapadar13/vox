export type DictationPhase =
  | "idle"
  | "recording"
  | "cancelPending"
  | "transcribing"
  | "delivering"
  | "success"
  | "error";

export type DeliveryMode = "pasted" | "clipboard" | "failed";

export interface AppState {
  phase: DictationPhase;
  elapsedMs: number;
  cancelRemainingMs: number | null;
  audioLevel: number;
  partialTranscript: string | null;
  stableWords: number;
  message: string | null;
  lastTranscript: string | null;
  deliveryMode: DeliveryMode | null;
  activeEngine: string;
  modelReady: boolean;
}

export type LanguageHint = { mode: "auto" } | { mode: "pinned"; language: string };

export interface Settings {
  schemaVersion: number;
  hotkey: string;
  language: LanguageHint;
  engineId: string;
  modelId: string;
  autoPaste: boolean;
  launchAtLogin: boolean;
  trimFillerWords: boolean;
  typingWpm: number;
  maxRecordingSeconds: number;
  onboardingComplete: boolean;
}

export interface TranscriptRecord {
  id: number | null;
  createdAt: string;
  text: string;
  rawText: string;
  language: string;
  durationMs: number;
  latencyMs: number;
  wordCount: number;
  engineId: string;
  delivered: boolean;
}

export interface ActivityDay {
  date: string;
  words: number;
}

export interface StatsSnapshot {
  transcriptionCount: number;
  totalWords: number;
  speakingMs: number;
  timeSavedMs: number;
  speakingWpm: number;
  averageLatencyMs: number;
  streakDays: number;
  activity: ActivityDay[];
}

export interface ManagedModel {
  id: string;
  name: string;
  filename: string;
  sizeBytes: number;
  sha256: string;
  url: string;
  speed: string;
  accuracy: string;
  multilingual: boolean;
  installed: boolean;
  active: boolean;
}

export interface ModelDownloadProgress {
  downloadedBytes: number;
  totalBytes: number;
  fraction: number;
}

export interface CommandError {
  code: string;
  message: string;
}

export const idleState: AppState = {
  phase: "idle",
  elapsedMs: 0,
  cancelRemainingMs: null,
  audioLevel: 0,
  partialTranscript: null,
  stableWords: 0,
  message: null,
  lastTranscript: null,
  deliveryMode: null,
  activeEngine: "whisper-turbo",
  modelReady: false,
};

export const defaultSettings: Settings = {
  schemaVersion: 1,
  hotkey: "CommandOrControl+Shift+Space",
  language: { mode: "auto" },
  engineId: "whisper-turbo",
  modelId: "whisper-large-v3-turbo-q5_0",
  autoPaste: true,
  launchAtLogin: false,
  trimFillerWords: false,
  typingWpm: 40,
  maxRecordingSeconds: 300,
  onboardingComplete: false,
};

export const emptyStats: StatsSnapshot = {
  transcriptionCount: 0,
  totalWords: 0,
  speakingMs: 0,
  timeSavedMs: 0,
  speakingWpm: 0,
  averageLatencyMs: 0,
  streakDays: 0,
  activity: [],
};
