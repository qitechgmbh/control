import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_RESTART_BACKEND_INTO_PREOP,
  TROUBLESHOOT_EXPORT_LOGS,
} from "./troubleshoot-channels";

export function exposeTroubleshootContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("troubleshoot", {
    rebootHmi: () => ipcRenderer.invoke(TROUBLESHOOT_REBOOT_HMI),
    restartBackend: () => ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND),
    restartBackendIntoPreop: () =>
      ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND_INTO_PREOP),
    exportLogs: () => ipcRenderer.invoke(TROUBLESHOOT_EXPORT_LOGS),
  });
}
