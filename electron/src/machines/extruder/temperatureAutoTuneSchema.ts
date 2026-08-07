import { z } from "zod";

/**
 * Schemas for the IMC temperature auto-tuner, shared between extruder2 and extruder3.
 *
 * Mirrors the Rust types in `machine_implementations/src/extruder1/api.rs`.
 */

export const heatingZoneNames = ["front", "middle", "back", "nozzle"] as const;
export type TuneZone = (typeof heatingZoneNames)[number];

/**
 * One tuning candidate, in both IMC (Kc/Ti/Td) and parallel-PID (kp/ki/kd) parameterisations.
 */
export const imcGainsSchema = z.object({
  kc: z.number(),
  ti: z.number(),
  td: z.number(),
  kp: z.number(),
  ki: z.number(),
  kd: z.number(),
});
export type ImcGains = z.infer<typeof imcGainsSchema>;

/**
 * Identified model plus fit diagnostics from a completed run.
 */
export const temperatureAutoTuneResultSchema = z.object({
  /** Steady-state gain, °C per unit of duty cycle. */
  process_gain: z.number(),
  /** Fitted time constant, seconds. */
  time_constant: z.number(),
  /** Fitted dead time, seconds. */
  dead_time: z.number(),
  /** Classical 63.2% construction — cross-check only. */
  tau_63: z.number(),
  /** Dead time from the first threshold crossing — cross-check only, reads above the fitted value. */
  dead_time_threshold: z.number(),
  rms_residual: z.number(),
  fit_error_pct: z.number(),
  is_good_fit: z.boolean(),
  delta_pv: z.number(),
  delta_u: z.number(),
  lambda: z.number(),
  noise_peak_to_peak: z.number(),
  snr_ratio: z.number(),
  is_confident: z.boolean(),
  suggested_step_duty: z.number(),
  pi: imcGainsSchema,
  pid: imcGainsSchema,
});
export type TemperatureAutoTuneResult = z.infer<
  typeof temperatureAutoTuneResultSchema
>;

export const tunePhases = [
  "idle",
  "waiting_for_steady",
  "baseline_hold",
  "step",
  "completed",
  "failed",
] as const;
export type TunePhase = (typeof tunePhases)[number];

/** Human-readable phase labels, in run order. */
export const tunePhaseLabels: Record<string, string> = {
  idle: "Idle",
  waiting_for_steady: "Waiting for steady state",
  baseline_hold: "Holding baseline power",
  step: "Step response",
  completed: "Completed",
  failed: "Failed",
};

export const temperatureAutoTuneStateSchema = z.object({
  zone: z.string().nullable(),
  phase: z.string(),
  progress: z.number(),
  elapsed_seconds: z.number(),
  baseline_duty: z.number(),
  baseline_temperature: z.number(),
  current_duty: z.number(),
  result: temperatureAutoTuneResultSchema.nullable(),
  failure_reason: z.string().nullable(),
});
export type TemperatureAutoTuneState = z.infer<
  typeof temperatureAutoTuneStateSchema
>;

export const temperatureAutoTuneSampleSchema = z.object({
  t_seconds: z.number(),
  temperature: z.number(),
  duty: z.number(),
});
export type TemperatureAutoTuneSample = z.infer<
  typeof temperatureAutoTuneSampleSchema
>;

/**
 * The recorded step-test curve. Sent whole rather than incrementally, so this stays stateless and
 * survives a page reload mid-run.
 */
export const temperatureAutoTuneTraceDataSchema = z.object({
  zone: z.string().nullable(),
  phase: z.string(),
  samples: z.array(temperatureAutoTuneSampleSchema),
});
export type TemperatureAutoTuneTraceData = z.infer<
  typeof temperatureAutoTuneTraceDataSchema
>;

/** Operator-facing response-speed presets, mapped to the IMC lambda factor. */
export const responseSpeeds = [
  { label: "Aggressive", factor: 0.5 },
  { label: "Moderate", factor: 1.0 },
  { label: "Conservative", factor: 2.0 },
] as const;

/**
 * Reconstruct the fitted FOPDT curve so it can be overlaid on the recorded trace. A visible
 * mismatch between the two is the fastest way to spot a bad run.
 */
export function fittedCurve(
  result: TemperatureAutoTuneResult,
  baselineTemperature: number,
  samples: TemperatureAutoTuneSample[],
  stepStartSeconds: number,
): { t_seconds: number; temperature: number }[] {
  const { time_constant: tau, dead_time: theta, delta_pv: amplitude } = result;
  if (tau <= 0) return [];
  return samples
    .filter((s) => s.t_seconds >= stepStartSeconds)
    .map((s) => {
      const t = s.t_seconds - stepStartSeconds;
      const value =
        t < theta ? 0 : amplitude * (1 - Math.exp(-(t - theta) / tau));
      return {
        t_seconds: s.t_seconds,
        temperature: baselineTemperature + value,
      };
    });
}
