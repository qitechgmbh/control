import { Unit } from "@/control/units";
import { GraphConfig, SeriesData, GraphLine } from "../types";

/**
 * Type definitions for export data structures
 */
export type GraphExportData = {
  config: GraphConfig;
  data: SeriesData;
  unit?: Unit;
  renderValue?: (value: number) => string;
};

export type PidSettings = {
  kp: number;
  ki: number;
  kd: number;
  zone?: string; // For temperature zones (front, middle, back, nozzle)
};

export type PidData = {
  temperature?: Record<string, PidSettings>; // keyed by zone
  pressure?: PidSettings;
};

export type CombinedSheetData = {
  sheetName: string;
  timestamps: number[];
  values: number[];
  unit: string;
  seriesTitle: string;
  graphTitle: string;
  targetLines: GraphLine[];
  color?: string;
};
