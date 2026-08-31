import { ENVIRONMENT_INFO } from "./environment-channels";

export function exposeEnvironmentContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("environment", {
    getInfo: () => ipcRenderer.invoke(ENVIRONMENT_INFO),
  });
}
