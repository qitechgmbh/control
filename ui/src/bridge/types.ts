export type ThemeMode = "dark" | "light" | "system";

export interface EnvironmentInfo {
  qitechOs: boolean;
  qitechOsGitTimestamp?: Date;
  qitechOsGitCommit?: string;
  qitechOsGitAbbreviation?: string;
  qitechOsGitUrl?: string;
}

export interface NixOSGeneration {
  id: string;
  name: string;
  version: string;
  current: boolean;
  date: string;
  path: string;
  kernelVersion?: string;
  description?: string;
}

export interface UpdateExecuteParams {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
}

export interface UpdateStepParams {
  stepId: string;
  status: "pending" | "in-progress" | "completed" | "error";
  progress?: number;
}

export interface NativeBridge {
  theme: {
    current: () => Promise<ThemeMode>;
    toggle: () => Promise<boolean>;
    dark: () => Promise<void>;
    light: () => Promise<void>;
    system: () => Promise<boolean>;
  };
  window: {
    minimize: () => Promise<void>;
    maximize: () => Promise<void>;
    fullscreen: (value: boolean) => Promise<void>;
    close: () => Promise<void>;
  };
  environment: {
    getInfo: () => Promise<EnvironmentInfo>;
  };
  update: {
    execute: (params: UpdateExecuteParams) => Promise<void>;
    cancel: () => Promise<{ success: boolean; error?: string }>;
    onLog: (callback: (log: string) => void) => void;
    onEnd: (
      callback: (params: { success: boolean; error?: string }) => void,
    ) => void;
    onStep: (callback: (params: UpdateStepParams) => void) => void;
  };
  troubleshoot: {
    rebootHmi: () => Promise<{ success: boolean; error?: string }>;
    restartBackend: () => Promise<{ success: boolean; error?: string }>;
    exportLogs: () => Promise<{ success: boolean; error?: string }>;
  };
  nixos: {
    isNixOSAvailable: boolean;
    listGenerations: () => Promise<NixOSGeneration[]>;
    setGeneration: (generationId: string) => Promise<void>;
    deleteGeneration: (generationId: string) => Promise<void>;
    deleteAllOldGenerations: () => Promise<void>;
  };
}
