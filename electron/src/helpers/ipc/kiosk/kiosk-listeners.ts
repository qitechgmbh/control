import { BrowserWindow, ipcMain } from "electron";
import { spawn } from "child_process";
import { KIOSK_SET_LOCKED_CHANNEL } from "./kiosk-channels";

// Disables GNOME's built-in touchscreen gestures (overview, workspace swipes).
// Installed on QiTech OS but only enabled while the kiosk is locked.
const GNOME_GESTURES_EXTENSION = "disable-gestures-2021@verycrazydog.gmail.com";

function setGnomeGesturesDisabled(disabled: boolean) {
  if (process.env.QITECH_OS !== "true") return;

  const child = spawn("gnome-extensions", [
    disabled ? "enable" : "disable",
    GNOME_GESTURES_EXTENSION,
  ]);
  child.on("error", (error) => {
    console.warn("Failed to toggle GNOME gestures:", error);
  });
  child.on("exit", (code) => {
    if (code !== 0) {
      console.warn(`gnome-extensions exited with code ${code}`);
    }
  });
}

export function addKioskEventListeners(mainWindow: BrowserWindow) {
  const { webContents } = mainWindow;
  let locked = false;
  let wasFullScreen = false;

  const setLocked = (value: boolean) => {
    if (value === locked) return;
    locked = value;

    if (locked) {
      wasFullScreen = mainWindow.isFullScreen();
      webContents.closeDevTools();
      mainWindow.setKiosk(true);
      mainWindow.setAlwaysOnTop(true, "screen-saver");
    } else {
      mainWindow.setAlwaysOnTop(false);
      mainWindow.setKiosk(false);
      mainWindow.setFullScreen(wasFullScreen);
    }
    mainWindow.setMinimizable(!locked);
    mainWindow.setClosable(!locked);
    setGnomeGesturesDisabled(locked);
  };

  // The lock does not survive a restart, so make sure gestures are back on
  setGnomeGesturesDisabled(false);

  // Swallow every key while locked: reload, devtools, zoom, fullscreen toggles
  webContents.on("before-input-event", (event) => {
    if (locked) event.preventDefault();
  });
  webContents.on("devtools-opened", () => {
    if (locked) webContents.closeDevTools();
  });
  webContents.on("will-navigate", (event) => {
    if (locked) event.preventDefault();
  });
  // A reload resets the renderer's lock state, so keep both sides in sync
  webContents.on("did-finish-load", () => setLocked(false));
  mainWindow.on("close", (event) => {
    if (locked) event.preventDefault();
  });

  ipcMain.handle(KIOSK_SET_LOCKED_CHANNEL, (_, value: boolean) => {
    setLocked(value);
  });
}
