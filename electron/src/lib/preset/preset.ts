import { machineIdentificaiton } from "@/machines/types";
import { z } from "zod";

export type PresetSchema = z.ZodTypeAny;

export const presetSchema = <S extends PresetSchema>(dataSchema: S) =>
  z.object({
    id: z.number().int().nonnegative().optional(),
    name: z.string().nonempty(),
    lastModified: z.coerce.date(),
    machineIdentificaiton: machineIdentificaiton,
    schemaVersion: z.number().int().positive(),
    data: dataSchema,
  });

export type Preset<S extends PresetSchema> = z.infer<
  ReturnType<typeof presetSchema<S>>
>;
