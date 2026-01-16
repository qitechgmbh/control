import { machineIdentification } from "@/machines/types";
import { z } from "zod";

export type PresetSchema<T> = z.ZodType<T>;

export const presetSchema = <T>(dataSchema: PresetSchema<T>) =>
  z.object({
    id: z.number().int().nonnegative().optional(),
    name: z.string().nonempty(),
    lastModified: z.coerce.date(),
    machineIdentification,
    schemaVersion: z.number().int().positive(),
    data: dataSchema.optional(),
  });

export type Preset<T> = z.infer<ReturnType<typeof presetSchema<T>>>;
