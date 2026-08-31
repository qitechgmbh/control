import type { UpdateInfo } from "@/stores/updateStore";
import { useUpdateStore } from "@/stores/updateStore";

let currentLogListener: ((line: string) => void) | null = null;
let currentStepListener:
  | ((params: {
      stepId: string;
      status: "pending" | "in-progress" | "completed" | "error";
      progress?: number;
    }) => void)
  | null = null;

export async function updateExecute(
  source: UpdateInfo,
  onLog: (log: string) => void,
): Promise<{ success: boolean; error?: string }> {
  return new Promise((resolve) => {
    // Remove previous listener if exists
    if (currentLogListener) {
      window.update.onLog(() => {}); // clear previous
      currentLogListener = null;
    }

    if (currentStepListener) {
      window.update.onStep(() => {}); // clear previous
      currentStepListener = null;
    }

    currentLogListener = onLog;
    window.update.onLog(currentLogListener);

    // Set up step listener
    currentStepListener = (params) => {
      const { setStepStatus, setStepProgress } = useUpdateStore.getState();
      setStepStatus(params.stepId, params.status);
      if (params.progress !== undefined) {
        setStepProgress(params.stepId, params.progress);
      }
    };
    window.update.onStep(currentStepListener);

    window.update.execute(source);

    window.update.onEnd((params) => {
      window.update.onLog(() => {}); // remove listener
      window.update.onEnd(() => {});
      window.update.onStep(() => {}); // remove step listener
      currentLogListener = null;
      currentStepListener = null;
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
  const result = await window.update.cancel();
  window.update.onLog(() => {}); // remove listener
  window.update.onEnd(() => {}); // remove listener
  window.update.onStep(() => {}); // remove step listener
  return result;
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
