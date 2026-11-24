import { ipcMain } from "electron";
import path from "node:path";
import { getHomeDir } from "../update/token-store";
import { PRESETS_READ, PRESETS_WRITE } from "./presets-channels";
import { PresetsFileStore } from "./presets-file-store";

export function addPresetsEventListeners() {
  const store = new PresetsFileStore(
    path.join(getHomeDir(), ".config", "qitech-control"),
  );

  // Handlers are synchronous so concurrent writes can never interleave.
  ipcMain.handle(PRESETS_READ, () => store.read());
  ipcMain.handle(PRESETS_WRITE, (_event, content: string) =>
    store.write(content),
  );
}
