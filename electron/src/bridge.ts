import type { NativeBridge } from "@ui/bridge/types";

export const electronBridge: NativeBridge = {
  theme: window.themeMode,
  window: window.electronWindow,
  environment: window.environment,
  update: window.update,
  troubleshoot: window.troubleshoot,
  nixos: window.nixos,
};
