// Core components
export { BigGraph } from "./BigGraph";
export { GraphControls, FloatingControlPanel } from "./GraphControls";

// Synchronized components
export {
  SyncedBigGraph,
  SyncedGraphControls,
  SyncedFloatingControlPanel,
  AutoSyncedBigGraph,
} from "./SyncedComponents";

// Hooks
export { useGraphSync } from "./useGraphSync";

// Types - make sure these are exported with 'type'
export type {
  PropGraphSync,
  GraphLine,
  GraphConfig,
  BigGraphProps,
  TimeWindowOption,
  ControlProps,
} from "./types";

// Constants
export { DEFAULT_TIME_WINDOW_OPTIONS, DEFAULT_COLORS } from "./constants";

// Excel helpers (re-export)
export { exportGraphsToExcel } from "./excelExport";
export type { GraphExportData } from "./excelExport";
