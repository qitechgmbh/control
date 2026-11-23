/**
 * Graphs for machine pages.
 *
 *   const syncHook = useGraphSync("extruder-graphs");
 *   <GraphHistoryProvider machine={machineIdentification}>
 *     <BigGraph syncHook={syncHook} graphId="rpm" newData={...} config={...} />
 *     <GraphControlPanel syncHook={syncHook} />
 *   </GraphHistoryProvider>
 *
 * Layout of this folder:
 * - `BigGraph`, `MiniGraph`, `useGraphSync`, `types`: what pages use
 * - `engine/`:   Chart.js chart without React (frame loop, viewport, datasets, plugins)
 * - `history/`:  loading measurement/setpoint history from the database
 * - `markers/`:  marker buttons, dialogs and data hook (drawn by `engine/plugins/markers`)
 * - `controls/`: the floating control panel
 * - `export/`:   export dialog backend: Excel report, CSV/Parquet streamed from the database
 */
export { BigGraph, type BigGraphProps } from "./BigGraph";
export { MiniGraph } from "./MiniGraph";
export { GraphControlPanel } from "./controls/GraphControlPanel";
export { GraphHistoryProvider } from "./history/GraphHistoryContext";
export { useGraphSync, type GraphSync } from "./useGraphSync";
export type {
  GraphConfig,
  GraphLine,
  SeriesData,
  TimeWindowOption,
} from "./types";
export { DEFAULT_TIME_WINDOW_OPTIONS, DEFAULT_COLORS } from "./constants";
