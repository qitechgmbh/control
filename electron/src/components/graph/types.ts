import { IconName } from "@/components/icons/Icon";
import { TimeSeries } from "@/lib/timeseries";

/** A time range in Unix ms. */
export type ViewRange = { min: number; max: number };

/** Length of the visible window in ms, or everything the database holds. */
export type TimeWindow = number | "all";

export type TimeWindowOption = {
  value: TimeWindow;
  label: string;
};

/** What every graph of a page shows. Owned by `useGraphSync`, so all graphs move together. */
export type GraphView = {
  timeWindow: TimeWindow;
  isLiveMode: boolean;
  /** Range picked by pan/zoom (historical mode). */
  xRange?: ViewRange;
  /** When historical mode was entered: the right edge of the view if no range is picked. */
  historicalFreezeTimestamp: number | null;
  /** Show everything from this timestamp on, instead of a fixed window. */
  showFromTimestamp: number | null;
  /** Graphs of one group share their hover crosshair. */
  cursorGroup: string;
};

// --- lines drawn in addition to the series ---

type BaseGraphLine = {
  color: string;
  width?: number;
  dash?: number[];
  show?: boolean;
};

/** A fixed horizontal line, e.g. a tolerance. Always kept inside the y range. */
type ThresholdLine = BaseGraphLine & {
  type: "threshold";
  value: number;
  label?: string;
};

/** A setpoint drawn as a step line: its history from the database, then the live value. */
type TargetLine = BaseGraphLine & {
  type: "target";
  value: number;
  label?: string;
  targetSeries?: TimeSeries;
  /** Config property path whose history is loaded from the database, e.g. `screw.target_rpm`. */
  history?: string;
};

export type GraphLine = ThresholdLine | TargetLine;

// --- graph configuration ---

export type GraphConfig = {
  title: string;
  icon?: IconName;
  lines?: GraphLine[];
  timeWindows?: Array<{ value: TimeWindow; label: string }>;
  defaultTimeWindow?: TimeWindow;
  exportFilename?: string;
  showLegend?: boolean;
  colors?: {
    primary?: string;
    grid?: string;
    axis?: string;
    background?: string;
  };
};

/** One line of a graph: live data from a buffer, and optionally its database history. */
export type SeriesData = {
  newData: TimeSeries | null;
  /** Measurement path whose history is loaded from the database, e.g. `motor.rpm`. */
  history?: string;
  /** Factor applied to database values when the UI shows a converted unit (e.g. 100 for %). */
  historyScale?: number;
  title?: string;
  color?: string;
  lines?: GraphLine[];
};

export type DataSeries = SeriesData | SeriesData[];
