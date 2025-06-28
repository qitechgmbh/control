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
  contextBridge.exposeInMainWorld("update", {
    execute: (params: UpdateExecuteInvokeParams) =>
      ipcRenderer.invoke(UPDATE_EXECUTE, params),
    cancel: () => ipcRenderer.invoke(UPDATE_CANCEL),
    onLog: (callback: (log: string) => void) =>
      ipcRenderer.on(UPDATE_LOG, (_event, log: string) => {
        callback(log);
      }),
    onEnd: (callback: (params: { success: boolean; error?: string }) => void) =>
      ipcRenderer.on(UPDATE_END, (_event, params) => {
        callback(params);
      }),
  });
}
