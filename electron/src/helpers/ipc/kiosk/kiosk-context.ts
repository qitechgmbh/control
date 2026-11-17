import { KIOSK_SET_LOCKED_CHANNEL } from "./kiosk-channels";

export function exposeKioskContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("kiosk", {
    setLocked: (locked: boolean) =>
      ipcRenderer.invoke(KIOSK_SET_LOCKED_CHANNEL, locked),
  });
}
