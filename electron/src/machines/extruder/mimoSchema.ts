import { z } from "zod";

/**
 * Schemas for MIMO thermal control, shared between extruder2 and extruder3.
 *
 * Mirrors the Rust types in `machine_implementations/src/extruder1/api.rs`.
 *
 * Every matrix here is indexed in *physical* order along the barrel — nozzle, front, middle,
 * back — not the order the zones appear elsewhere in the UI. The backend sends `zone_order`
 * alongside the data so labelling never has to assume; use it rather than hard-coding.
 */

/** Physical order along the barrel, nozzle end first. Matches `ZONE_ORDER` in `mimo.rs`. */
export const mimoZoneOrder = ["nozzle", "front", "middle", "back"] as const;
export type MimoZone = (typeof mimoZoneOrder)[number];

export const mimoZoneLabels: Record<string, string> = {
  nozzle: "Nozzle",
  front: "Front",
  middle: "Middle",
  back: "Back",
};

export const mimoPhases = [
  "idle",
  "waiting_for_steady",
  "baseline_hold",
  "step",
  "completed",
  "failed",
] as const;
export type MimoPhase = (typeof mimoPhases)[number];

/** Human-readable phase labels, in run order. */
export const mimoPhaseLabels: Record<string, string> = {
  idle: "Idle",
  waiting_for_steady: "Waiting for steady state",
  baseline_hold: "Holding baseline power",
  step: "Measuring coupling",
  completed: "Completed",
  failed: "Failed",
};

/** Phases in which the campaign owns the heaters. */
export const mimoRunningPhases: string[] = [
  "waiting_for_steady",
  "baseline_hold",
  "step",
];

/** One entry of the coupling matrix: the response of one zone to a step on one heater. */
export const mimoEntrySchema = z.object({
  /** Steady-state gain, °C per unit of duty. */
  gp: z.number(),
  /** Time constant, seconds. */
  tau: z.number(),
  /** Dead time, seconds. */
  theta: z.number(),
  rms_residual: z.number(),
  /** Response size over peak-to-peak noise. Low is expected for a distant zone. */
  snr_ratio: z.number(),
});
export type MimoEntry = z.infer<typeof mimoEntrySchema>;

export const mimoModelSchema = z.object({
  /** `g[output][input]`, both in `zone_order`. */
  g: z.array(z.array(mimoEntrySchema)),
  zone_order: z.array(z.string()),
  setpoints: z.array(z.number()),
  /** Relative Gain Array of the DC gain matrix. */
  rga: z.array(z.array(z.number())),
  condition_number: z.number(),
  niederlinski: z.number(),
  /** Largest departure of the RGA diagonal from 1. Near zero means the zones barely interact. */
  max_rga_deviation: z.number(),
  /** Strongest off-diagonal gain as a fraction of the driven zone's own gain. */
  max_coupling_ratio: z.number(),
  identified_at_secs: z.number(),
});
export type MimoModel = z.infer<typeof mimoModelSchema>;

export const mimoGainsSchema = z.object({
  kp: z.array(z.array(z.number())),
  ki: z.array(z.array(z.number())),
  kd: z.array(z.array(z.number())),
  derivative_filter_tc: z.number(),
  method: z.string(),
});
export type MimoGains = z.infer<typeof mimoGainsSchema>;

export const mimoStateSchema = z.object({
  /** `"decentralized"` or `"mimo"`. */
  mode: z.string(),
  phase: z.string(),
  is_running: z.boolean(),
  progress_percent: z.number(),
  elapsed_seconds: z.number(),
  /** Index into `zone_order` of the zone currently being stepped. */
  column: z.number().nullable(),
  columns_done: z.number(),
  zone_order: z.array(z.string()),
  failure_reason: z.string().nullable(),
  model: mimoModelSchema.nullable(),
  gains: mimoGainsSchema.nullable(),
  synthesis_error: z.string().nullable(),
});
export type MimoState = z.infer<typeof mimoStateSchema>;

export const mimoTraceSampleSchema = z.object({
  t_seconds: z.number(),
  temperatures: z.array(z.number()),
  duties: z.array(z.number()),
});
export type MimoTraceSample = z.infer<typeof mimoTraceSampleSchema>;

export const mimoTraceSchema = z.object({
  phase: z.string(),
  column: z.number().nullable(),
  zone_order: z.array(z.string()),
  samples: z.array(mimoTraceSampleSchema),
});
export type MimoTraceData = z.infer<typeof mimoTraceSchema>;

/**
 * Thresholds the panel colours diagnostics against.
 *
 * These mirror the refusal limits the backend applies in `synth_decoupler.rs`, so what the panel
 * shows as a warning is the same thing that will decline to synthesize. Keeping them in step
 * matters: an operator who sees green everywhere and then gets a refusal has been misled.
 */
export const MAX_CONDITION_NUMBER = 20;
export const MIN_RGA_DIAGONAL = 0.3;
export const MAX_RGA_DIAGONAL = 3.0;
/** Below this the zones are effectively already decoupled and MIMO control has little to add. */
export const NEGLIGIBLE_RGA_DEVIATION = 0.1;
/** Confidence floor for a fitted entry, matching `MIN_SNR_RATIO` in `imc_tuner.rs`. */
export const MIN_SNR_RATIO = 5.0;

/** Whether the model looks good enough for the decoupler to accept it. */
export function modelIsSynthesizable(model: MimoModel): boolean {
  if (model.condition_number > MAX_CONDITION_NUMBER) return false;
  if (model.niederlinski < 0) return false;
  return model.rga.every((row, i) => {
    const d = row[i];
    return d >= MIN_RGA_DIAGONAL && d <= MAX_RGA_DIAGONAL;
  });
}
