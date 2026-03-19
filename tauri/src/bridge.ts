import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  NativeBridge,
  ThemeMode,
  EnvironmentInfo,
  NixOSGeneration,
  UpdateExecuteParams,
  UpdateStepParams,
} from "@ui/bridge/types";

let currentLogUnlisten: UnlistenFn | null = null;
let currentEndUnlisten: UnlistenFn | null = null;
let currentStepUnlisten: UnlistenFn | null = null;

export const tauriBridge: NativeBridge = {
  theme: {
    current: () => invoke<ThemeMode>("theme_current"),
    toggle: () => invoke<boolean>("theme_toggle"),
    dark: () => invoke("theme_dark"),
    light: () => invoke("theme_light"),
    system: () => invoke<boolean>("theme_system"),
  },
  window: {
    minimize: () => invoke("window_minimize"),
    maximize: () => invoke("window_maximize"),
    fullscreen: (value: boolean) => invoke("window_fullscreen", { value }),
    close: () => invoke("window_close"),
  },
  environment: {
    getInfo: () => invoke<EnvironmentInfo>("environment_get_info"),
  },
  update: {
    execute: (params: UpdateExecuteParams) =>
      invoke("update_execute", { params }),
    cancel: () => invoke<{ success: boolean; error?: string }>("update_cancel"),
    onLog: (callback: (log: string) => void) => {
      if (currentLogUnlisten) currentLogUnlisten();
      listen<string>("update-log", (event) => callback(event.payload)).then(
        (unlisten) => {
          currentLogUnlisten = unlisten;
        },
      );
    },
    onEnd: (
      callback: (params: { success: boolean; error?: string }) => void,
    ) => {
      if (currentEndUnlisten) currentEndUnlisten();
      listen<{ success: boolean; error?: string }>("update-end", (event) =>
        callback(event.payload),
      ).then((unlisten) => {
        currentEndUnlisten = unlisten;
      });
    },
    onStep: (callback: (params: UpdateStepParams) => void) => {
      if (currentStepUnlisten) currentStepUnlisten();
      listen<UpdateStepParams>("update-step", (event) =>
        callback(event.payload),
      ).then((unlisten) => {
        currentStepUnlisten = unlisten;
      });
    },
  },
  troubleshoot: {
    rebootHmi: () =>
      invoke<{ success: boolean; error?: string }>("troubleshoot_reboot_hmi"),
    restartBackend: () =>
      invoke<{ success: boolean; error?: string }>(
        "troubleshoot_restart_backend",
      ),
    exportLogs: () =>
      invoke<{ success: boolean; error?: string }>("troubleshoot_export_logs"),
  },
  nixos: {
    isNixOSAvailable: false,
    listGenerations: () => invoke<NixOSGeneration[]>("nixos_list_generations"),
    setGeneration: (generationId: string) =>
      invoke("nixos_set_generation", { generationId }),
    deleteGeneration: (generationId: string) =>
      invoke("nixos_delete_generation", { generationId }),
    deleteAllOldGenerations: () => invoke("nixos_delete_all_old_generations"),
  },
};

// Initialize nixos availability at startup
invoke<boolean>("nixos_is_available").then((available) => {
  tauriBridge.nixos.isNixOSAvailable = available;
});
