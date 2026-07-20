import { ipcMain, dialog } from "electron";
import fs from "fs";
import {
  EXPORT_SAVE_FILE,
  EXPORT_EJECT_USB,
  SaveFileParams,
  SaveFileResult,
} from "./export-channels";
import { getRemovableVolumeRoot, ejectVolume } from "./removable-media";

export function addExportEventListeners() {
  ipcMain.handle(
    EXPORT_SAVE_FILE,
    async (_event, params: SaveFileParams): Promise<SaveFileResult> => {
      try {
        const { canceled, filePath } = await dialog.showSaveDialog({
          title: "Save File",
          defaultPath: params.suggestedName,
          filters: params.filters,
        });

        if (canceled || !filePath) {
          return { success: false, error: "Export cancelled by user" };
        }

        fs.writeFileSync(
          filePath,
          Buffer.from(params.content, params.encoding),
        );

        const mountPath = getRemovableVolumeRoot(filePath);
        return {
          success: true,
          filePath,
          isRemovable: mountPath !== null,
          mountPath: mountPath ?? undefined,
        };
      } catch (error) {
        console.error("Failed to save file:", error);
        return {
          success: false,
          error: error instanceof Error ? error.message : String(error),
        };
      }
    },
  );

  ipcMain.handle(EXPORT_EJECT_USB, async (_event, mountPath: string) => {
    return await ejectVolume(mountPath);
  });
}
