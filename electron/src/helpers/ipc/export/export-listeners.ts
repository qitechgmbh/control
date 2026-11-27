import { ipcMain, dialog } from "electron";
import fs from "fs";
import { Readable } from "stream";
import { pipeline } from "stream/promises";
import type { ReadableStream as WebReadableStream } from "stream/web";
import {
  EXPORT_SAVE_FILE,
  EXPORT_EJECT_USB,
  EXPORT_CHOOSE_PATH,
  EXPORT_WRITE_FILE,
  EXPORT_DOWNLOAD,
  ChoosePathParams,
  ChoosePathResult,
  DownloadParams,
  SaveFileParams,
  SaveFileResult,
  WriteFileParams,
} from "./export-channels";
import { getRemovableVolumeRoot, ejectVolume } from "./removable-media";

function savedResult(filePath: string): SaveFileResult {
  const mountPath = getRemovableVolumeRoot(filePath);
  return {
    success: true,
    filePath,
    isRemovable: mountPath !== null,
    mountPath: mountPath ?? undefined,
  };
}

function failedResult(error: unknown): SaveFileResult {
  return {
    success: false,
    error: error instanceof Error ? error.message : String(error),
  };
}

/** Streams an HTTP response body into `filePath` (via a temp file, so a failed download leaves nothing behind). */
async function download({ filePath, url, body }: DownloadParams) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok || !response.body) {
    let message = `HTTP ${response.status}`;
    try {
      const json = await response.json();
      if (json?.error) message = json.error;
    } catch {
      // keep the status message
    }
    throw new Error(message);
  }

  const tmpPath = `${filePath}.part`;
  try {
    await pipeline(
      Readable.fromWeb(response.body as WebReadableStream<Uint8Array>),
      fs.createWriteStream(tmpPath),
    );
    await fs.promises.rename(tmpPath, filePath);
  } catch (error) {
    await fs.promises.rm(tmpPath, { force: true });
    throw error;
  }
}

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
        return savedResult(filePath);
      } catch (error) {
        console.error("Failed to save file:", error);
        return failedResult(error);
      }
    },
  );

  ipcMain.handle(
    EXPORT_CHOOSE_PATH,
    async (_event, params: ChoosePathParams): Promise<ChoosePathResult> => {
      const { canceled, filePath } = await dialog.showSaveDialog({
        title: "Export",
        defaultPath: params.suggestedName,
        filters: params.filters,
      });
      return canceled || !filePath
        ? { canceled: true }
        : { canceled: false, filePath };
    },
  );

  ipcMain.handle(
    EXPORT_WRITE_FILE,
    async (_event, params: WriteFileParams): Promise<SaveFileResult> => {
      try {
        await fs.promises.writeFile(
          params.filePath,
          Buffer.from(params.content, params.encoding),
        );
        return savedResult(params.filePath);
      } catch (error) {
        console.error("Failed to write file:", error);
        return failedResult(error);
      }
    },
  );

  ipcMain.handle(
    EXPORT_DOWNLOAD,
    async (_event, params: DownloadParams): Promise<SaveFileResult> => {
      try {
        await download(params);
        return savedResult(params.filePath);
      } catch (error) {
        console.error("Failed to download export:", error);
        return failedResult(error);
      }
    },
  );

  ipcMain.handle(EXPORT_EJECT_USB, async (_event, mountPath: string) => {
    return await ejectVolume(mountPath);
  });
}
