import { z } from "zod";

/**
 * PID auto-tune result schema
 */
export const pidAutoTuneResultSchema = z.object({
  kp: z.number(),
  ki: z.number(),
  kd: z.number(),
});

/**
 * PID auto-tune state schema
 */
export const pidAutoTuneStateSchema = z.object({
  state: z.string(),
  progress: z.number(),
  result: pidAutoTuneResultSchema.nullable(),
});

export type PidAutoTuneState = z.infer<typeof pidAutoTuneStateSchema>;
