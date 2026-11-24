export type EventCategory = "SFX" | "MUSIC";
export type BakeMode = "modding" | "standalone";

export interface AppConfig {
  mode: BakeMode;
  fmodCompilerPath: string | null;
  outputDirectory: string | null;
  buildMasterBanks: boolean;
  exportGuids: boolean;
  keepTempProject: boolean;
  suppressUpdatePopup?: boolean;
  isDebugBuild?: boolean;
}

export interface GithubReleaseInfo {
  tagName: string;
  version: string;
  name: string;
  body: string;
  htmlUrl: string;
}

export interface CompilerInfo {
  available: boolean;
  bundledAvailable?: boolean;
  path: string | null;
  source: "configured" | "bundled" | "system" | "not_found" | string;
  version: string | null;
  message: string;
}

export interface AudioFileInfo {
  path: string;
  filename: string;
  suggestedEventName: string;
  sizeBytes: number;
  extension: string;
  durationSeconds: number | null;
}

export interface AudioEventCardItem {
  id: string;
  filePath: string;
  filename: string;
  eventName: string;
  category: EventCategory;
  looping: boolean;
  volumeDb: number;
  customLoopEnabled: boolean;
  loopStart: number;
  loopEnd: number | null;
  duration: number;
  sizeBytes: number;
  showWaveform: boolean;
  showAdvanced: boolean;
  audioBlobUrl?: string;
}

export interface BakeEventItemPayload {
  id: string;
  filePath: string;
  filename: string;
  eventName: string;
  category: EventCategory;
  looping: boolean;
  volumeDb: number;
  customLoopEnabled: boolean;
  loopStart: number;
  loopEnd: number | null;
  duration: number;
}

export interface BakeBankRequest {
  bankName: string;
  events: BakeEventItemPayload[];
  mode?: BakeMode;
  outputDirectory?: string | null;
  fmodCompilerPath?: string | null;
  buildMasterBanks?: boolean;
  exportGuids?: boolean;
}

export interface BuiltFileEntry {
  filename: string;
  path: string;
  sizeBytes: number;
}

export interface BakeBankResult {
  success: boolean;
  bankName: string;
  outputDirectory: string;
  builtFiles: BuiltFileEntry[];
  guidsContent: string | null;
  compilerStdout: string;
  compilerStderr: string;
  generatedScript: string;
  durationMs: number;
  message: string;
}
