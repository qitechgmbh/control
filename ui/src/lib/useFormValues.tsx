import { useEffect, useState } from "react";
import { FieldValues, UseFormReturn } from "react-hook-form";

/**
 * This hook just returns the values of a form.
 * In contrast to using `watch` from `react-hook-form`, this hook
 * can reduce unnecessary re-renders when a render is triggered by a parent component.
 */
export function useFormValues<TFieldValues extends FieldValues>(
  form: UseFormReturn<TFieldValues>,
) {
  const [values, setValues] = useState<TFieldValues>(form.getValues());
  useEffect(() => {
    const subscription = form.watch(async (data) =>
      setValues(data as TFieldValues),
    );
    return () => subscription.unsubscribe();
  }, []);
  return values;
}
