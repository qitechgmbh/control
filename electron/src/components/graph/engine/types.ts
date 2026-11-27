import type { TimeSeries } from "@/lib/timeseries";
import type { ViewRange } from "../types";
import type { HLine } from "./plugins/thresholds";
import type { ChartMarker } from "./plugins/markers";

/** Everything `GraphEngine` draws, pushed by `BigGraph` on every render. */
export type EngineInputs = {
  series: EngineSeries[];
  targets: EngineTarget[];
  hLines: HLine[];
  markers: ChartMarker[];
  view: EngineView;
  renderValue?: (value: number) => string;
  colors: { grid: string; axis: string };
};

export type EngineSeries = {
  newData: TimeSeries | null;
  history?: string;
  historyScale?: number;
  color: string;
  visible: boolean;
};

/** A setpoint step line of one series. */
export type EngineTarget = {
  color: string;
  value: number;
  history?: string;
  targetSeries?: TimeSeries;
  dash?: number[];
  width?: number;
  visible: boolean;
};

/** The shared page view, as the engine needs it. */
export type EngineView = {
  live: boolean;
  /** Window length in ms ("all" = database retention, fitted to the data). */
  windowMs: number;
  all: boolean;
  /** Range picked by pan/zoom (historical). */
  range?: ViewRange;
  showFrom?: number | null;
  freeze?: number | null;
  /** Changes whenever the shared view changed (`GraphView.revision`). */
  revision: number;
};

export type EngineCallbacks = {
  /** Cursor value per series, or null when the cursor left all synced charts. */
  onCursor: (values: (number | null)[] | null) => void;
  /** User panned/zoomed this chart. */
  onUserRange: (range: ViewRange) => void;
  onStatus: (status: { loading: boolean; error: string | null }) => void;
};
