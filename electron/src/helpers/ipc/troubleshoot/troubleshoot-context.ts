import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_START_LOG_STREAM,
  TROUBLESHOOT_STOP_LOG_STREAM,
  TROUBLESHOOT_LOG_DATA,
  TROUBLESHOOT_RESTART_BACKEND_DEBUG,
} from "./troubleshoot-channels";

export function exposeTroubleshootContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("troubleshoot", {
    rebootHmi: () => ipcRenderer.invoke(TROUBLESHOOT_REBOOT_HMI),
    restartBackend: () => ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND),
    restartBackendDebug: () =>
      ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND_DEBUG),
    startLogStream: () => ipcRenderer.invoke(TROUBLESHOOT_START_LOG_STREAM),
    stopLogStream: () => ipcRenderer.invoke(TROUBLESHOOT_STOP_LOG_STREAM),
    onLogData: (callback: (log: string) => void) =>
      ipcRenderer.on(TROUBLESHOOT_LOG_DATA, (_event, log: string) => {
        callback(log);
      }),
  });
}
