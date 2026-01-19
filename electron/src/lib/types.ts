import { z } from "zod";

export function rustEnumSchema<T extends Record<string, z.ZodType>>(
  schemas: T,
) {
  const shape: any = {};

  for (const [key, schema] of Object.entries(schemas)) {
    if (!schema) {
      //cath undefined 'undefined' early
      throw new Error(
        `rustEnumSchema: Schema for key "${key}" is undefined. Check for circular imports.`,
      );
    }
    shape[key] = schema.optional();
  }

  const objectSchema = z.object(shape);

  return objectSchema.refine(
    (data) => {
      const definedKeys = Object.keys(data).filter(
        (key) => data[key] !== undefined,
      );
      return definedKeys.length === 1;
    },
    {
      message: "Exactly one property must be defined",
      path: ["_exclusive"],
    },
  );
}
