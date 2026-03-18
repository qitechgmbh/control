import { BrowserWindow } from "electron";
import { addThemeEventListeners } from "./theme/theme-listeners";
import { addWindowEventListeners } from "./window/window-listeners";
import { addEnvironmentEventListeners } from "./environment/environment-listeners";
import { addUpdateEventListeners } from "./update/update-listeners";
import { addTroubleshootEventListeners } from "./troubleshoot/troubleshoot-listeners";
import { addNixOSEventListeners } from "./nixos/nixos-listeners";

export default function registerListeners(mainWindow: BrowserWindow) {
  addWindowEventListeners(mainWindow);
  addThemeEventListeners();
  addEnvironmentEventListeners();
  addUpdateEventListeners();
  addTroubleshootEventListeners();
  addNixOSEventListeners();
}
