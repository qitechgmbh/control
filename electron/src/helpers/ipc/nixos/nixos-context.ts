import {
  NIXOS_LIST_GENERATIONS,
  NIXOS_SET_GENERATION,
  NIXOS_DELETE_GENERATION,
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

export function exposeNixOSContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("nixos", {
    listGenerations: () => ipcRenderer.invoke(NIXOS_LIST_GENERATIONS),
    setGeneration: (generationId: string) =>
      ipcRenderer.invoke(NIXOS_SET_GENERATION, generationId),
    deleteGeneration: (generationId: string) =>
      ipcRenderer.invoke(NIXOS_DELETE_GENERATION, generationId),
  });
}
