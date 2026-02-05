import EventEmitter from "node:events";
import {
  TROUBLESHOOT_GET_LOG_LINES,
  TROUBLESHOOT_ON_LOG_LINE,
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
} from "./troubleshoot-channels";

export function exposeTroubleshootContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");

  const emitter = new EventEmitter<TroubleshootContextEvents>;

  ipcRenderer.on(TROUBLESHOOT_ON_LOG_LINE, (_event, numLines: number) => {
    emitter.emit("log-line", numLines);
  });

  const troubleshoot: TroubleshootContext = {
    rebootHmi() {
      return ipcRenderer.invoke(TROUBLESHOOT_REBOOT_HMI);
    },

    restartBackend() {
        return ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND);
    },

    getLogLines(start, count) {
      return ipcRenderer.invoke(TROUBLESHOOT_GET_LOG_LINES, start, count);
    },

    subscribeLogLine(callback) {
        emitter.on("log-line", callback);
        return () => emitter.off("log-line", callback);
    },
  };

  contextBridge.exposeInMainWorld("troubleshoot", troubleshoot);
}
