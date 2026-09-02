import { create } from "zustand";
import { produce } from "immer";

export type LogEntry = {
  id: string;
  timestamp: Date;
  source: string;
  level: "info" | "warn" | "error" | "debug";
  message: string;
  raw: string;
};

export type LogsState = {
  entries: LogEntry[];
  isStreaming: boolean;
  sources: Set<string>;
};

export type LogsActions = {
  addLogEntry: (entry: Omit<LogEntry, "id" | "timestamp">) => void;
  clearLogs: () => void;
  setStreaming: (streaming: boolean) => void;
  getLogsBySource: (source: string) => LogEntry[];
};

export type LogsStore = LogsState & LogsActions;

const initialState: LogsState = {
  entries: [],
  isStreaming: false,
  sources: new Set(),
};

export const useLogsStore = create<LogsStore>((set, get) => ({
  ...initialState,
  addLogEntry: (entry) =>
    set(
      produce((state: LogsState) => {
        const newEntry: LogEntry = {
          ...entry,
          id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          timestamp: new Date(),
        };

        state.entries.push(newEntry);
        state.sources.add(entry.source);

        // Keep only last 10000 entries to prevent memory issues
        if (state.entries.length > 10000) {
          state.entries.splice(0, state.entries.length - 10000);
        }
      }),
    ),

  clearLogs: () =>
    set(
      produce((state: LogsState) => {
        state.entries = [];
        state.sources.clear();
      }),
    ),

  setStreaming: (streaming) =>
    set(
      produce((state: LogsState) => {
        state.isStreaming = streaming;
      }),
    ),

  getLogsBySource: (source) => {
    console.log(source);
    const state = get();
    return state.entries.filter((entry) => entry.source === source);
  },
}));
