import {
  GNOME_HIDE_VIRTUAL_KEYBAORD_CHANNEL,
  GNOME_SHOW_VIRTUAL_KEYBAORD_CHANNEL,
} from "./gnome-channels";

export function exposeGnomeContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("gnome", {
    showVirtualKeyboard: () =>
      ipcRenderer.invoke(GNOME_SHOW_VIRTUAL_KEYBAORD_CHANNEL),
    hideVirtualKeyboard: () =>
      ipcRenderer.invoke(GNOME_HIDE_VIRTUAL_KEYBAORD_CHANNEL),
  });
}
