import { BrowserWindow } from "electron";
import { addThemeEventListeners } from "./theme/theme-listeners";
import { addWindowEventListeners } from "./window/window-listeners";
import { addEnvironmentEventListeners } from "./environment/environment-listeners";
import { addUpdateEventListeners } from "./update/update-listeners";
import { addTroubleshootEventListeners } from "./troubleshoot/troubleshoot-listeners";
import { addNixOSEventListeners } from "./nixos/nixos-listeners";
// Keyboard is now handled in-app via VirtualKeyboard component
// import { addKeyboardEventListeners } from "./keyboard/keyboard-listeners";

export default function registerListeners(mainWindow: BrowserWindow) {
  addWindowEventListeners(mainWindow);
  addThemeEventListeners();
  addEnvironmentEventListeners();
  addUpdateEventListeners();
  addTroubleshootEventListeners();
  addNixOSEventListeners();
  // Keyboard is now handled in-app via VirtualKeyboard component
  // addKeyboardEventListeners(mainWindow);
}
