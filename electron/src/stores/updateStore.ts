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

export type UpdateStep = {
  id: string;
  label: string;
  status: "pending" | "in-progress" | "completed" | "error";
  subsector: "nixos" | "rust" | "electron" | "general";
  progress?: number; // 0-100 for steps with detailed progress tracking
};

export type UpdateState = {
  isUpdating: boolean;
  terminalLines: string[];
  currentUpdateInfo: UpdateInfo | null;
  steps: UpdateStep[];
  currentStepIndex: number;
  overallProgress: number; // 0-100
};

export type UpdateActions = {
  setUpdateInfo: (info: UpdateInfo) => void;
  startUpdate: () => void;
  stopUpdate: () => void;
  addTerminalLine: (line: string) => void;
  clearTerminalLines: () => void;
  resetUpdateState: () => void;
  setStepStatus: (stepId: string, status: UpdateStep["status"]) => void;
  setStepProgress: (stepId: string, progress: number) => void;
  initializeSteps: () => void;
  updateProgress: (progress: number) => void;
};

export type UpdateStore = UpdateState & UpdateActions;

const defaultSteps: UpdateStep[] = [
  {
    id: "clone-repo",
    label: "Clone repository",
    status: "pending",
    subsector: "general",
  },
  {
    id: "rust-build",
    label: "Build system packages",
    status: "pending",
    subsector: "rust",
  },
  {
    id: "system-install",
    label: "Install system updates",
    status: "pending",
    subsector: "nixos",
  },
];

const initialState: UpdateState = {
  isUpdating: false,
  terminalLines: [],
  currentUpdateInfo: null,
  steps: defaultSteps,
  currentStepIndex: 0,
  overallProgress: 0,
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
        state.steps = defaultSteps.map((step) => ({ ...step }));
        state.currentStepIndex = 0;
        state.overallProgress = 0;
      }),
    ),

  initializeSteps: () =>
    set(
      produce((state: UpdateState) => {
        state.steps = defaultSteps.map((step) => ({ ...step }));
        state.currentStepIndex = 0;
        state.overallProgress = 0;
      }),
    ),

  setStepStatus: (stepId: string, status: UpdateStep["status"]) =>
    set(
      produce((state: UpdateState) => {
        const stepIndex = state.steps.findIndex((s) => s.id === stepId);
        if (stepIndex !== -1) {
          state.steps[stepIndex].status = status;

          // Update current step index if this step is in progress
          if (status === "in-progress") {
            state.currentStepIndex = stepIndex;
          }

          // Calculate overall progress
          const completedSteps = state.steps.filter(
            (s) => s.status === "completed",
          ).length;
          state.overallProgress = Math.round(
            (completedSteps / state.steps.length) * 100,
          );
        }
      }),
    ),

  setStepProgress: (stepId: string, progress: number) =>
    set(
      produce((state: UpdateState) => {
        const stepIndex = state.steps.findIndex((s) => s.id === stepId);
        if (stepIndex !== -1) {
          state.steps[stepIndex].progress = Math.min(
            100,
            Math.max(0, progress),
          );
        }
      }),
    ),

  updateProgress: (progress: number) =>
    set(
      produce((state: UpdateState) => {
        state.overallProgress = Math.min(100, Math.max(0, progress));
      }),
    ),
}));
