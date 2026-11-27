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
  EXPORT_EXTENSION,
  EXPORT_FILE_SUFFIX,
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

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function failedResult(error: unknown): SaveFileResult {
  return { success: false, error: errorMessage(error) };
}

/** The only endpoint `EXPORT_DOWNLOAD` fetches (see `history_export.rs`). */
const HISTORY_EXPORT_URL =
  "http://localhost:3001/api/v1/machine/history/export";

/**
 * Base paths (chosen path without extension) the user picked in an export save dialog.
 * `EXPORT_WRITE_FILE` and `EXPORT_DOWNLOAD` write only next to one of them, so the
 * renderer cannot write anywhere the user did not choose.
 */
class ExportGrants {
  private static readonly MAX = 16;
  private readonly bases: string[] = [];

  grant(chosenPath: string) {
    const base = chosenPath.replace(EXPORT_EXTENSION, "");
    this.bases.push(base);
    if (this.bases.length > ExportGrants.MAX) this.bases.shift();
  }

  check(filePath: string) {
    const allowed = this.bases.some(
      (base) =>
        filePath.startsWith(base) &&
        EXPORT_FILE_SUFFIX.test(filePath.slice(base.length)),
    );
    if (!allowed) {
      throw new Error(
        `Not an export file chosen in the save dialog: ${filePath}`,
      );
    }
  }
}

const grants = new ExportGrants();

/** Streams the history export into `filePath` (via a temp file, so a failed download leaves nothing behind). */
async function download({ filePath, body }: DownloadParams) {
  const response = await fetch(HISTORY_EXPORT_URL, {
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
      try {
        const { canceled, filePath } = await dialog.showSaveDialog({
          title: "Export",
          defaultPath: params.suggestedName,
          filters: params.filters,
        });
        if (canceled || !filePath) return { canceled: true };
        grants.grant(filePath);
        return { canceled: false, filePath };
      } catch (error) {
        console.error("Export save dialog failed:", error);
        return { canceled: true, error: errorMessage(error) };
      }
    },
  );

  ipcMain.handle(
    EXPORT_WRITE_FILE,
    async (_event, params: WriteFileParams): Promise<SaveFileResult> => {
      try {
        grants.check(params.filePath);
        await fs.promises.writeFile(params.filePath, params.data);
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
        grants.check(params.filePath);
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
