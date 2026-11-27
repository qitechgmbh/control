/**
 * File Export Helper Functions
 *
 * Wrapper functions for the generic "save to a user-chosen path" IPC
 * mechanism, following the same pattern as troubleshoot_helpers.ts.
 *
 * @see src/helpers/troubleshoot_helpers.ts for the same pattern
 */

import type {
  ChoosePathParams,
  ChoosePathResult,
  DownloadParams,
  SaveFileParams,
  SaveFileResult,
  WriteFileParams,
} from "@/helpers/ipc/export/export-channels";
export type { SaveFileParams, SaveFileResult };

function failed(error: unknown): SaveFileResult {
  return {
    success: false,
    error: error instanceof Error ? error.message : String(error),
  };
}

export async function saveFile(
  params: SaveFileParams,
): Promise<SaveFileResult> {
  try {
    return await window.fileExport.saveFile(params);
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/** Asks the user for a file path (save dialog) without writing anything yet. */
export async function choosePath(
  params: ChoosePathParams,
): Promise<ChoosePathResult> {
  return await window.fileExport.choosePath(params);
}

/** Writes renderer-side content to a path chosen with `choosePath`. */
export async function writeFile(
  params: WriteFileParams,
): Promise<SaveFileResult> {
  try {
    return await window.fileExport.writeFile(params);
  } catch (error) {
    return failed(error);
  }
}

/** Streams a backend response into a file; the data never enters the renderer. */
export async function downloadToFile(
  params: DownloadParams,
): Promise<SaveFileResult> {
  try {
    return await window.fileExport.download(params);
  } catch (error) {
    return failed(error);
  }
}

export async function ejectUsb(mountPath: string): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.fileExport.ejectUsb(mountPath);
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
