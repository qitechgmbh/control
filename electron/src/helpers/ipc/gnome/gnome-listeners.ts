import { BrowserWindow, ipcMain } from "electron";
import {
  GNOME_HIDE_VIRTUAL_KEYBAORD_CHANNEL,
  GNOME_SHOW_VIRTUAL_KEYBAORD_CHANNEL,
} from "./gnome-channels";
import { exec } from "child_process";

export function addGnomeEventListeners(mainWindow: BrowserWindow) {
  ipcMain.handle(GNOME_SHOW_VIRTUAL_KEYBAORD_CHANNEL, () => {
    exec(
      "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled true",
    );
  });
  ipcMain.handle(GNOME_HIDE_VIRTUAL_KEYBAORD_CHANNEL, () => {
    exec(
      "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled false",
    );
  });
}
