import { z } from "zod";

export const rustEnum: z.core.CheckFn<object> = (ctx) => {
  const definedKeys = Object.keys(ctx.value).filter(
    (key) => ctx.value[key] !== undefined,
  );

  if (definedKeys.length === 1) {
    return;
  }

  ctx.issues.push({
    code: "custom",
    message: "Exactly one property must be defined",
    input: definedKeys,
  });
};
