import { getBridge } from "@ui/bridge";
import type { UpdateInfo } from "@ui/stores/updateStore";
import { useUpdateStore } from "@ui/stores/updateStore";

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
      getBridge().update.onLog(() => {}); // clear previous
      currentLogListener = null;
    }

    if (currentStepListener) {
      getBridge().update.onStep(() => {}); // clear previous
      currentStepListener = null;
    }

    currentLogListener = onLog;
    getBridge().update.onLog(currentLogListener);

    // Set up step listener
    currentStepListener = (params) => {
      const { setStepStatus, setStepProgress } = useUpdateStore.getState();
      setStepStatus(params.stepId, params.status);
      if (params.progress !== undefined) {
        setStepProgress(params.stepId, params.progress);
      }
    };
    getBridge().update.onStep(currentStepListener);

    getBridge().update.execute(source);

    getBridge().update.onEnd((params) => {
      getBridge().update.onLog(() => {}); // remove listener
      getBridge().update.onEnd(() => {});
      getBridge().update.onStep(() => {}); // remove step listener
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
  const result = await getBridge().update.cancel();
  getBridge().update.onLog(() => {}); // remove listener
  getBridge().update.onEnd(() => {}); // remove listener
  getBridge().update.onStep(() => {}); // remove step listener
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
