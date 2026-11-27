import {
  EXPORT_SAVE_FILE,
  EXPORT_EJECT_USB,
  EXPORT_CHOOSE_PATH,
  EXPORT_WRITE_FILE,
  EXPORT_DOWNLOAD,
  ChoosePathParams,
  DownloadParams,
  SaveFileParams,
  WriteFileParams,
} from "./export-channels";

export function exposeFileExportContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("fileExport", {
    saveFile: (params: SaveFileParams) =>
      ipcRenderer.invoke(EXPORT_SAVE_FILE, params),
    choosePath: (params: ChoosePathParams) =>
      ipcRenderer.invoke(EXPORT_CHOOSE_PATH, params),
    writeFile: (params: WriteFileParams) =>
      ipcRenderer.invoke(EXPORT_WRITE_FILE, params),
    download: (params: DownloadParams) =>
      ipcRenderer.invoke(EXPORT_DOWNLOAD, params),
    ejectUsb: (mountPath: string) =>
      ipcRenderer.invoke(EXPORT_EJECT_USB, mountPath),
  });
}
