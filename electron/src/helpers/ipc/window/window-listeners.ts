import { BrowserWindow, ipcMain } from "electron";
import {
  WIN_CLOSE_CHANNEL,
  WIN_FULLSCREEN_CHANNEL,
  WIN_MAXIMIZE_CHANNEL,
  WIN_MINIMIZE_CHANNEL,
} from "./window-channels";

export function addWindowEventListeners(mainWindow: BrowserWindow) {
  // Remove any existing handlers to prevent duplicates
  ipcMain.removeHandler(WIN_MINIMIZE_CHANNEL);
  ipcMain.removeHandler(WIN_MAXIMIZE_CHANNEL);
  ipcMain.removeHandler(WIN_FULLSCREEN_CHANNEL);
  ipcMain.removeHandler(WIN_CLOSE_CHANNEL);

  ipcMain.handle(WIN_MINIMIZE_CHANNEL, () => {
    mainWindow.minimize();
  });
  ipcMain.handle(WIN_MAXIMIZE_CHANNEL, () => {
    if (mainWindow.isMaximized()) {
      mainWindow.unmaximize();
    } else {
      mainWindow.maximize();
    }
  });
  ipcMain.handle(WIN_FULLSCREEN_CHANNEL, (event, value: boolean) => {
    mainWindow.setFullScreen(value);
  });
  ipcMain.handle(WIN_CLOSE_CHANNEL, () => {
    mainWindow.close();
  });
}
