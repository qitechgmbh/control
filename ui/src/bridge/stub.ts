import type { NativeBridge } from "./types";

const notAvailable =
  <T = void>(name: string, fallback?: T) =>
  (): Promise<T> => {
    console.warn(`[bridge] ${name} is not available in this environment`);
    return Promise.resolve(fallback as T);
  };

export const stubBridge: NativeBridge = {
  theme: {
    current: notAvailable("theme.current"),
    toggle: notAvailable("theme.toggle"),
    dark: notAvailable("theme.dark"),
    light: notAvailable("theme.light"),
    system: notAvailable("theme.system"),
  },
  window: {
    minimize: notAvailable("window.minimize"),
    maximize: notAvailable("window.maximize"),
    fullscreen: notAvailable("window.fullscreen"),
    close: notAvailable("window.close"),
  },
  environment: {
    getInfo: () =>
      Promise.resolve({
        qitechOs: false,
      }),
  },
  update: {
    execute: notAvailable("update.execute"),
    cancel: () => Promise.resolve({ success: false, error: "Not available" }),
    onLog: () => {},
    onEnd: () => {},
    onStep: () => {},
  },
  troubleshoot: {
    rebootHmi: () =>
      Promise.resolve({ success: false, error: "Not available" }),
    restartBackend: () =>
      Promise.resolve({ success: false, error: "Not available" }),
    exportLogs: () =>
      Promise.resolve({ success: false, error: "Not available" }),
  },
  nixos: {
    isNixOSAvailable: false,
    listGenerations: () => Promise.resolve([]),
    setGeneration: notAvailable("nixos.setGeneration"),
    deleteGeneration: notAvailable("nixos.deleteGeneration"),
    deleteAllOldGenerations: notAvailable("nixos.deleteAllOldGenerations"),
  },
};
