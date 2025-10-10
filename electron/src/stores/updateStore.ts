import { create } from "zustand";
import { produce } from "immer";

export type UpdateInfo = {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

export type UpdateState = {
  isUpdating: boolean;
  terminalLines: string[];
  currentUpdateInfo: UpdateInfo | null;
};

export type UpdateActions = {
  setUpdateInfo: (info: UpdateInfo) => void;
  startUpdate: () => void;
  stopUpdate: () => void;
  addTerminalLine: (line: string) => void;
  clearTerminalLines: () => void;
  resetUpdateState: () => void;
};

export type UpdateStore = UpdateState & UpdateActions;

const initialState: UpdateState = {
  isUpdating: false,
  terminalLines: [],
  currentUpdateInfo: null,
};

export const useUpdateStore = create<UpdateStore>((set) => ({
  ...initialState,

  setUpdateInfo: (info) =>
    set(
      produce((state: UpdateState) => {
        // Only allow setting update info if not currently updating
        if (!state.isUpdating) {
          // Check if the update target has changed
          const hasTargetChanged =
            !state.currentUpdateInfo ||
            state.currentUpdateInfo.githubRepoOwner !== info.githubRepoOwner ||
            state.currentUpdateInfo.githubRepoName !== info.githubRepoName ||
            state.currentUpdateInfo.tag !== info.tag ||
            state.currentUpdateInfo.branch !== info.branch ||
            state.currentUpdateInfo.commit !== info.commit;

          // Clear terminal lines if target changed
          if (hasTargetChanged) {
            state.terminalLines = [];
          }

          state.currentUpdateInfo = info;
        }
      }),
    ),

  startUpdate: () =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = true;
        state.terminalLines = [];
      }),
    ),

  stopUpdate: () =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = false;
      }),
    ),

  addTerminalLine: (line) =>
    set(
      produce((state: UpdateState) => {
        console.log(state.terminalLines);

        const lastLine = state.terminalLines[state.terminalLines.length - 1];
        if (line !== lastLine) {
          state.terminalLines.push(line);

          // Keep only last 10000 lines to prevent memory issues
          if (state.terminalLines.length > 10000) {
            state.terminalLines.splice(0, state.terminalLines.length - 10000);
          }
        }
      }),
    ),

  clearTerminalLines: () =>
    set(
      produce((state: UpdateState) => {
        state.terminalLines = [];
      }),
    ),

  resetUpdateState: () =>
    set(
      produce((state: UpdateState) => {
        state.isUpdating = false;
        state.terminalLines = [];
        state.currentUpdateInfo = null;
      }),
    ),
}));
