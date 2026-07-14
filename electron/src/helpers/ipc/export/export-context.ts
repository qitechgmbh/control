import { EXPORT_SAVE_FILE, EXPORT_EJECT_USB } from "./export-channels";

export function exposeFileExportContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("fileExport", {
    saveFile: (params: {
      suggestedName: string;
      filters?: { name: string; extensions: string[] }[];
      content: string;
      encoding: "utf8" | "base64";
    }) => ipcRenderer.invoke(EXPORT_SAVE_FILE, params),
    ejectUsb: (mountPath: string) =>
      ipcRenderer.invoke(EXPORT_EJECT_USB, mountPath),
  });
}
