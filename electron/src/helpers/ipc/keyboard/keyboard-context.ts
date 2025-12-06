import { contextBridge, ipcRenderer } from "electron";
import { KEYBOARD_SHOW, KEYBOARD_HIDE, VIRTUAL_KEYBOARD_VISIBILITY_CHANGED } from "./keyboard-channels";

export function exposeKeyboardContext() {
  contextBridge.exposeInMainWorld("keyboard", {
    show: () => ipcRenderer.invoke(KEYBOARD_SHOW),
    hide: () => ipcRenderer.invoke(KEYBOARD_HIDE),
    setVirtualKeyboardVisibility: (visible: boolean) => {
      ipcRenderer.send(VIRTUAL_KEYBOARD_VISIBILITY_CHANGED, visible);
    },
  });
}


