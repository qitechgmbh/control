import { z } from "zod";

/**
 * Steady-state gain matrix / Relative Gain Array result of a thermal coupling test.
 * Both matrices are indexed [output_zone][input_zone] in the order given by `zones`.
 */
export const thermalCouplingResultSchema = z.object({
  zones: z.tuple([z.string(), z.string(), z.string(), z.string()]),
  gain_matrix: z.array(z.array(z.number())),
  rga_matrix: z.array(z.array(z.number())),
});

export type ThermalCouplingResult = z.infer<typeof thermalCouplingResultSchema>;

/**
 * Thermal coupling test state schema
 */
export const thermalCouplingTestStateSchema = z.object({
  state: z.string(),
  zone_under_test: z.string().nullable(),
  elapsed_secs: z.number(),
  duration_secs: z.number(),
  zones_completed: z.number(),
  error: z.string().nullable(),
  result: thermalCouplingResultSchema.nullable(),
});

export type ThermalCouplingTestState = z.infer<
  typeof thermalCouplingTestStateSchema
>;
