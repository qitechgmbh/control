import {
  UPDATE_CANCEL,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
} from "./update-channels";

type UpdateExecuteInvokeParams = {
  source: {
    githubRepoOwner: string;
    githubRepoName: string;
    githubToken?: string;
    tag?: string;
    branch?: string;
    commit?: string;
  };
};

type UpdateStepParams = {
  stepId: string;
  status: "pending" | "in-progress" | "completed" | "error";
  progress?: number;
};

export function exposeUpdateContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");

  let currentLogListener: ((event: any, log: string) => void) | null = null;
  let currentEndListener: ((event: any, params: any) => void) | null = null;
  let currentStepListener:
    | ((event: any, params: UpdateStepParams) => void)
    | null = null;

  contextBridge.exposeInMainWorld("update", {
    execute: (params: UpdateExecuteInvokeParams) =>
      ipcRenderer.invoke(UPDATE_EXECUTE, params),
    cancel: () => ipcRenderer.invoke(UPDATE_CANCEL),

    onLog: (callback: (log: string) => void) => {
      if (currentLogListener) {
        ipcRenderer.removeListener(UPDATE_LOG, currentLogListener);
      }

      currentLogListener = (_event, log: string) => {
        callback(log);
      };

      ipcRenderer.on(UPDATE_LOG, currentLogListener);
    },

    onEnd: (
      callback: (params: { success: boolean; error?: string }) => void,
    ) => {
      if (currentEndListener) {
        ipcRenderer.removeListener(UPDATE_END, currentEndListener);
      }

      currentEndListener = (_event, params) => {
        callback(params);
      };

      ipcRenderer.on(UPDATE_END, currentEndListener);
    },

    onStep: (callback: (params: UpdateStepParams) => void) => {
      if (currentStepListener) {
        ipcRenderer.removeListener(UPDATE_STEP, currentStepListener);
      }

      currentStepListener = (_event, params) => {
        callback(params);
      };

      ipcRenderer.on(UPDATE_STEP, currentStepListener);
    },
  });
}
