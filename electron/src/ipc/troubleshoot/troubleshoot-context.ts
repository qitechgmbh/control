import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_EXPORT_LOGS,
} from "./troubleshoot-channels";

export function exposeTroubleshootContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("troubleshoot", {
    rebootHmi: () => ipcRenderer.invoke(TROUBLESHOOT_REBOOT_HMI),
    restartBackend: () => ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND),
    exportLogs: () => ipcRenderer.invoke(TROUBLESHOOT_EXPORT_LOGS),
  });
}
