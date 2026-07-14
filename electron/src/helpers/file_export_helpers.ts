/**
 * File Export Helper Functions
 *
 * Wrapper functions for the generic "save to a user-chosen path" IPC
 * mechanism, following the same pattern as troubleshoot_helpers.ts.
 *
 * @see src/helpers/troubleshoot_helpers.ts for the same pattern
 */

export type SaveFileParams = {
  suggestedName: string;
  filters?: { name: string; extensions: string[] }[];
  content: string;
  encoding: "utf8" | "base64";
};

export type SaveFileResult = {
  success: boolean;
  error?: string;
  filePath?: string;
  isRemovable?: boolean;
  mountPath?: string;
};

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
