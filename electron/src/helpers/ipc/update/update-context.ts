import {
  UPDATE_CANCEL,
  UPDATE_START,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
} from "./update-channels";

export function exposeUpdateContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");

  const context: UpdateContext = {
    execute: (info) => ipcRenderer.invoke(UPDATE_EXECUTE, info),
    cancel: () => ipcRenderer.invoke(UPDATE_CANCEL),
    onLog: (callback) =>
      ipcRenderer.on(UPDATE_LOG, (_, line: string) => callback(line)),
    onStart: (callback) => ipcRenderer.on(UPDATE_START, callback),
    onEnd: (callback) => ipcRenderer.on(UPDATE_END, callback),
    onStep: (callback) =>
      ipcRenderer.on(UPDATE_STEP, (_, status) => callback(status)),
  };

  contextBridge.exposeInMainWorld("update", context);
}
