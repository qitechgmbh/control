/**
 * NixOS Helper Functions
 *
 * This module provides wrapper functions for NixOS generation management IPC operations,
 * following the same pattern as update_helpers.ts and troubleshoot_helpers.ts.
 *
 * @see src/helpers/update_helpers.ts for similar pattern
 */

/**
 * List all available NixOS generations
 */
export async function listNixOSGenerations(): Promise<{
  success: boolean;
  generations: NixOSGeneration[];
  error?: string;
}> {
  try {
    return await window.nixos.listGenerations();
  } catch (error) {
    return {
      success: false,
      generations: [],
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Set/switch to a specific NixOS generation
 */
export async function setNixOSGeneration(generationId: string): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.nixos.setGeneration(generationId);
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Delete a specific NixOS generation
 */
export async function deleteNixOSGeneration(generationId: string): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.nixos.deleteGeneration(generationId);
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Check if NixOS context is available
 */
export function isNixOSAvailable(): boolean {
  return typeof window !== "undefined" && typeof window.nixos !== "undefined";
}
