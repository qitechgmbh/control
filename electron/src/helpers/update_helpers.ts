import type { UpdateInfo } from "@/stores/updateStore";
import { useUpdateStore } from "@/stores/updateStore";

export async function updateExecute(
  source: UpdateInfo,
  onLog: (log: string) => void,
): Promise<{ success: boolean; error?: string }> {
  return new Promise((resolve) => {
    window.update.onLog(onLog);
    window.update.execute(source);
    window.update.onEnd((params) => {
      window.update.onLog(() => {});
      window.update.onEnd(() => {});
      resolve(params);
    });
  });
}

// Enhanced helper that automatically manages store state
export async function updateExecuteWithStore(
  source: UpdateInfo,
): Promise<{ success: boolean; error?: string }> {
  const { setUpdateInfo, startUpdate, stopUpdate, addTerminalLine } =
    useUpdateStore.getState();

  setUpdateInfo(source);
  startUpdate();

  try {
    const result = await updateExecute(source, addTerminalLine);
    stopUpdate();
    return result;
  } catch (error) {
    stopUpdate();
    throw error;
  }
}

export async function updateCancel(): Promise<{
  success: boolean;
  error?: string;
}> {
  return window.update.cancel();
}

// Enhanced helper that automatically manages store state
export async function updateCancelWithStore(): Promise<{
  success: boolean;
  error?: string;
}> {
  const result = await updateCancel();
  if (result.success) {
    const { stopUpdate } = useUpdateStore.getState();
    stopUpdate();
  }
  return result;
}
