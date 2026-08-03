import {
  EXPORT_SAVE_FILE,
  EXPORT_EJECT_USB,
  SaveFileParams,
} from "./export-channels";

export function exposeFileExportContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("fileExport", {
    saveFile: (params: SaveFileParams) =>
      ipcRenderer.invoke(EXPORT_SAVE_FILE, params),
    ejectUsb: (mountPath: string) =>
      ipcRenderer.invoke(EXPORT_EJECT_USB, mountPath),
  });
}
