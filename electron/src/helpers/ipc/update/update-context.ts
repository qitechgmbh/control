import {
  UPDATE_CANCEL,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
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

export function exposeUpdateContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");

  let currentLogListener: ((event: any, log: string) => void) | null = null;
  let currentEndListener: ((event: any, params: any) => void) | null = null;

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
  });
}
