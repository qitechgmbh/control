import { toast } from "sonner";
import { UpdateStep, useUpdateStore } from "./updateStore";

export type Update = {
  isUpdating: boolean;
  terminalLines: string[];
  currentUpdateInfo?: UpdateInfo;
  steps: UpdateStep[];
  overallProgress: number;
  start: (info: UpdateInfo) => void;
  cancel: () => void;
};

function linkStoreWithUpdateIPC() {
  const store = useUpdateStore.getState();

  window.update.onStart(() => store.startUpdate());
  window.update.onEnd(() => store.stopUpdate());
  window.update.onLog((line) => store.addTerminalLine(line));

  window.update.onStep((step) => {
    store.setStepStatus(step.stepId, step.status);
    if (step.progress !== undefined) {
      store.setStepProgress(step.stepId, step.progress);
    }
  });
}
linkStoreWithUpdateIPC();

export function useUpdate(): Update {
  const {
    currentUpdateInfo,
    setUpdateInfo,
    isUpdating,
    terminalLines,
    steps,
    overallProgress,
  } = useUpdateStore();

  async function start(info: UpdateInfo) {
    setUpdateInfo(info);

    try {
      await window.update.execute(info);
      toast.success("Update applied successfully");
    } catch (error: any) {
      toast.error("Update failed: " + error.toString());
    }
  }

  async function cancel() {
    try {
      await window.update.cancel();
      toast.info("Update cancelled successfully");
    } catch (error: any) {
      toast.error("Failed to cancel update: " + error.toString());
    }
  }

  return {
    isUpdating,
    terminalLines,
    steps,
    currentUpdateInfo,
    overallProgress,
    start,
    cancel,
  };
}
