import { contextBridge, ipcRenderer } from "electron";

export function exposeEnvironmentContext() {
  contextBridge.exposeInMainWorld("environment", {
    getInfo: () => ipcRenderer.invoke("environment-get-info"),
  });
}
