import { create } from "zustand";
import { produce } from "immer";

export type MachineName = string; // e.g., "mock1", "winder2", "extruder2"

export type TimeFrame = number | "all";

interface GraphSettingsState {
  timeframe: TimeFrame;
  setTimeframe: (graphTimeFrame: TimeFrame) => void;
  getTimeframe: () => TimeFrame;
}

export const useGraphSettingsStore = create<GraphSettingsState>((set, get) => ({
  timeframe: 30 * 1000 * 60,
  setTimeframe: (graphTimeFrame) =>
    set(
      produce((state: GraphSettingsState) => {
        state.timeframe = graphTimeFrame;
      }),
    ),

  getTimeframe: () => {
    const state = get();
    return state.timeframe;
  },
}));
