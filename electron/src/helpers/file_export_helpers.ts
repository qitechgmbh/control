/**
 * File Export Helper Functions
 *
 * Wrapper functions for the generic "save to a user-chosen path" IPC
 * mechanism, following the same pattern as troubleshoot_helpers.ts.
 *
 * @see src/helpers/troubleshoot_helpers.ts for the same pattern
 */

import type {
  SaveFileParams,
  SaveFileResult,
} from "@/helpers/ipc/export/export-channels";
export type { SaveFileParams, SaveFileResult };

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
