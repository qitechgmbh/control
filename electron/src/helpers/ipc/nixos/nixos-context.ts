import {
  NIXOS_IS_AVAILABLE,
  NIXOS_LIST_GENERATIONS,
  NIXOS_SET_GENERATION,
  NIXOS_DELETE_GENERATION,
  NIXOS_DELETE_ALL_OLD_GENERATIONS,
} from "./nixos-channels";

export type NixOSGeneration = {
  id: string;
  name: string;
  version: string;
  current: boolean;
  date: string;
  path: string;
  kernelVersion?: string;
  description?: string;
};

export async function exposeNixOSContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  const isNixOSAvailable = await ipcRenderer.invoke(NIXOS_IS_AVAILABLE);

  const context: NixOSContext = {
    isNixOSAvailable,
    listGenerations: () => ipcRenderer.invoke(NIXOS_LIST_GENERATIONS),
    setGeneration: (generationId: string) =>
      ipcRenderer.invoke(NIXOS_SET_GENERATION, generationId),
    deleteGeneration: (generationId: string) =>
      ipcRenderer.invoke(NIXOS_DELETE_GENERATION, generationId),
    deleteAllOldGenerations: () =>
      ipcRenderer.invoke(NIXOS_DELETE_ALL_OLD_GENERATIONS),
  };

  contextBridge.exposeInMainWorld("nixos", context);
}
