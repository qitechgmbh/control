import { z } from "zod";

/**
 * Creates a Zod schema for objects where exactly one property must be defined.
 * This is useful for handling Rust enum JSON representations where only one variant exists at a time.
 *
 * @param schemas A record mapping property names to their Zod schemas
 * @returns A Zod schema that validates that exactly one property is defined
 */
export function rustEnumSchema<T extends Record<string, z.ZodType>>(
  schemas: T,
): z.ZodType<{ [K in keyof T]?: z.infer<T[K]> }> {
  // Create an object schema where all properties are optional
  const schemaEntries = Object.entries(schemas).map(([key, schema]) => [
    key,
    schema.optional(),
  ]);

  const objectSchema = z.object(
    Object.fromEntries(schemaEntries) as {
      [K in keyof T]: z.ZodOptional<T[K]>;
    },
  );

  // Add refinement to ensure exactly one property is defined
  return objectSchema.refine(
    (data) => {
      const definedKeys = Object.keys(data).filter(
        (key) => data[key] !== undefined,
      );
      return definedKeys.length === 1;
    },
    {
      message: "Exactly one property must be defined",
      path: ["_exclusive"], // Add a path to help with error identification
    },
  );
}
