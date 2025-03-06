import { z } from "zod";

// super refine function to validate if a string is a valid u16 number
export function validateU16<T>(): (value: T, ctx: z.RefinementCtx) => boolean {
  const refine = (value: T, ctx: z.RefinementCtx) => {
    console.log("value", value);
    if (typeof value === "string") {
      const number = parseInt(value, 10);
      if (number < 0 || number > 65535) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: "Wert muss u16 integer sein",
        });
      }
    }
    return true;
  };
  return refine;
}
