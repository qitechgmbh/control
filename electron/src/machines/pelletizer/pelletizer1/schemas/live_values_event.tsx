import { z } from "zod";

export const liveValuesEventDataSchema = z.object({
    frequency:   z.number(),
    temperature: z.number(),
    voltage:     z.number(),
    current:     z.number(),
});