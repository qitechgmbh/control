import { contextBridge, ipcRenderer } from "electron";
import { KEYBOARD_SHOW, KEYBOARD_HIDE } from "./keyboard-channels";

export function exposeKeyboardContext() {
  contextBridge.exposeInMainWorld("keyboard", {
    show: () => ipcRenderer.invoke(KEYBOARD_SHOW),
    hide: () => ipcRenderer.invoke(KEYBOARD_HIDE),
  });
}

