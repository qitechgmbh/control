/**
 * Troubleshoot Helper Functions
 *
 * This module provides wrapper functions for troubleshoot IPC operations,
 * following the same pattern as update_helpers.ts. These helpers abstract
 * the direct IPC calls and provide better error handling and type safety.
 *
 * @see src/helpers/update_helpers.ts for similar pattern
 */

/**
 * Reboot the HMI panel
 */
export async function rebootHmi(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.troubleshoot.rebootHmi();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function restartBackend(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.troubleshoot.restartBackend();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function startLogStream(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.troubleshoot.startLogStream();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function stopLogStream(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await window.troubleshoot.stopLogStream();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Setup log data streaming with callback
 * @param onLogData Callback function to handle incoming log data
 */
export function setupLogDataListener(onLogData: (log: string) => void): void {
  window.troubleshoot.onLogData(onLogData);
}

/**
 * Cleanup log data listener
 */
export function cleanupLogDataListener(): void {
  window.troubleshoot.onLogData(() => {});
}

/**
 * Combined function to restart backend and show appropriate toast messages
 */
export async function restartBackendWithToast(): Promise<boolean> {
  const result = await restartBackend();

  if (result.success) {
    // Note: We can't import toast here due to circular dependencies
    // This function returns success/failure and the calling component handles toasts
    return true;
  } else {
    return false;
  }
}

/**
 * Combined function to reboot HMI and show appropriate toast messages
 */
export async function rebootHmiWithToast(): Promise<boolean> {
  const result = await rebootHmi();

  if (result.success) {
    return true;
  } else {
    return false;
  }
}

/**
 * Check if troubleshoot context is available
 */
export function isTroubleshootAvailable(): boolean {
  return (
    typeof window !== "undefined" && typeof window.troubleshoot !== "undefined"
  );
}
