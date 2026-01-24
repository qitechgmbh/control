import { z } from "zod";

export const stateEventDataSchema = z.object({
    is_default_state: z.boolean(),

    inverter_state:     z.object({
    running_state:      z.number(),
    frequency_target:   z.number(),
    acceleration_level: z.number(),
    deceleration_level: z.number(),

    error_code:    z.number().nullable(),
    system_status: z.number(),
  }),
});