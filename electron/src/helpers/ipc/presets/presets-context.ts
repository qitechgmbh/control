import { PRESETS_READ, PRESETS_WRITE } from "./presets-channels";

export function exposePresetsContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("presetStorage", {
    read: () => ipcRenderer.invoke(PRESETS_READ),
    write: (content: string) => ipcRenderer.invoke(PRESETS_WRITE, content),
  });
}
