import { create } from "zustand";
import { produce } from "immer";

export type UpdateState = {
  isUpdating: boolean;
  logs: string[];
  updateSource: {
    githubRepoOwner: string;
    githubRepoName: string;
    githubToken?: string;
    tag?: string;
    branch?: string;
    commit?: string;
  } | null;
  updateResult: {
    success: boolean;
    error?: string;
  } | null;
};

export type UpdateActions = {
  startUpdate: (source: UpdateState["updateSource"]) => void;
  addLog: (log: string) => void;
  clearLogs: () => void;
  finishUpdate: (result: { success: boolean; error?: string }) => void;
  cancelUpdate: () => void;
  resetUpdate: () => void;
};

export type UpdateStore = UpdateState & UpdateActions;

const initialState: UpdateState = {
  isUpdating: false,
  logs: [],
  updateSource: null,
  updateResult: null,
};

export const useUpdateStore = create<UpdateStore>((set) => ({
  ...initialState,

  startUpdate: (source) =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = true;
        state.updateSource = source;
        state.logs = [];
        state.updateResult = null;
      }),
    ),

  addLog: (log) =>
    set(
      produce((state: UpdateState) => {
        state.logs.push(log);
        
        // Keep only last 1000 log entries to prevent memory issues
        if (state.logs.length > 1000) {
          state.logs.splice(0, state.logs.length - 1000);
        }
      }),
    ),

  clearLogs: () =>
    set(
      produce((state: UpdateState) => {
        state.logs = [];
      }),
    ),

  finishUpdate: (result) =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = false;
        state.updateResult = result;
      }),
    ),

  cancelUpdate: () =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = false;
        state.updateResult = { success: false, error: "Update was cancelled by user" };
      }),
    ),

  resetUpdate: () =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = false;
        state.logs = [];
        state.updateSource = null;
        state.updateResult = null;
      }),
    ),
}));
