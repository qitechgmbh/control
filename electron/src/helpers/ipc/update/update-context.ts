import { GithubSource } from "@/setup/GithubSourceDialog";
import {
  UPDATE_FETCH_SEND,
  UPDATE_FETCH_RECV,
  UPDATE_CANCEL,
  UPDATE_END,
  UPDATE_EXECUTE,
  UPDATE_LOG,
  UPDATE_STEP,
} from "./update-channels";
import { UpdateTargets } from "@/helpers/update_helpers";

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

  let currentFetchRecvListener: 
    | ((event: any, result: UpdateTargets | string) => void) 
    | null = null;
    
  let currentLogListener: ((event: any, log: string) => void) | null = null;
  let currentEndListener: ((event: any, params: any) => void) | null = null;
  let currentStepListener:
    | ((event: any, params: UpdateStepParams) => void)
    | null = null;

  contextBridge.exposeInMainWorld("update", {
    fetchTargets: (source: GithubSource) =>
      ipcRenderer.invoke(UPDATE_FETCH_SEND, source),
    execute: (params: UpdateExecuteInvokeParams) =>
      ipcRenderer.invoke(UPDATE_EXECUTE, params),
    cancel: () => ipcRenderer.invoke(UPDATE_CANCEL),

    onFetchRecv: (callback: (result: UpdateTargets | string) => void) => {
      if (currentFetchRecvListener) {
        ipcRenderer.removeListener(UPDATE_FETCH_RECV, currentFetchRecvListener);
      }

      currentFetchRecvListener = (_event, result: UpdateTargets | string) => {
        callback(result);
      };

      ipcRenderer.on(UPDATE_FETCH_RECV, currentFetchRecvListener);
    },

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
