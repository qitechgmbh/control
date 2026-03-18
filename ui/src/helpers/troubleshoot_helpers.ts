import { getBridge } from "@ui/bridge";

/**
 * Reboot the HMI panel
 */
export async function rebootHmi(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await getBridge().troubleshoot.rebootHmi();
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
    return await getBridge().troubleshoot.restartBackend();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function exportLogs(): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    return await getBridge().troubleshoot.exportLogs();
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
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
 * ?????? How would you see this?
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
    typeof window !== "undefined" &&
    typeof getBridge().troubleshoot !== "undefined"
  );
}
