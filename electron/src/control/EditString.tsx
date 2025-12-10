import { Icon, IconName } from "@/components/Icon";
import React, { useEffect, useState } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { Separator } from "@/components/ui/separator";
import { Popover, PopoverContent } from "@/components/ui/popover";
import { PopoverTrigger } from "@radix-ui/react-popover";
import { TouchInput } from "@/components/touch/TouchInput";
import { IconText } from "@/components/IconText";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";

type Props = {
  value: string;
  title: string;
  description?: string;
  icon?: IconName;
  defaultValue?: string;
  confirmation?: string;
  onChange: (value: string) => void;
};

/**
 * EditString Component
 *
 * A comprehensive string input component with:
 * - Touch-friendly input interface inside a Popover
 * - Real-time validation based on a basic string schema
 * - Save/Abort/Reset functionality
 */
export function EditString({
  value,
  title,
  description,
  icon,
  defaultValue,
  confirmation,
  onChange,
}: Props) {
  // --- Form Setup ---
  // A simple string schema for validation
  const formSchema = z.object({
    value: z.string().min(1, { message: "Value cannot be empty" }),
  });
  type FormSchema = z.infer<typeof formSchema>;

  const defaultString = defaultValue ?? "";

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    // Use the external value for the initial state
    values: { value: value ?? defaultString },
    defaultValues: { value: defaultString },
    mode: "all",
  });

  const { formState, handleSubmit, register, watch, reset } = form;
  const formValue = watch("value");

  // --- State Management ---
  const [open, setOpen] = useState(false);

  // Sync external value changes
  useEffect(() => {
    reset({ value: value ?? defaultString });
  }, [value, defaultString, reset]);

  // --- Handlers ---
  const handleAbort = () => {
    // Reset form to the current external value when aborting
    reset({ value: value ?? defaultString });
    setOpen(false);
  };

  const onSubmit = (data: FormSchema) => {
    const submit = () => {
      onChange(data.value);
      setOpen(false);
    };

    if (confirmation) {
      if (window.confirm(confirmation)) {
        submit();
      }
      // else do nothing if user cancels
    } else {
      submit();
    }
  };

  const handleResetToDefault = () => {
    reset({ value: defaultString });
    // Note: To mimic the EditValue's behavior where it saves on default click,
    // we immediately save the default value.
    form.handleSubmit(onSubmit)();
  };

  const valueIsDefined = value !== undefined && value !== null;

  return (
    <Popover
      open={open}
      onOpenChange={(isOpen) => {
        setOpen(isOpen);
        if (!isOpen) {
          // Reset on close if user hasn't explicitly saved or aborted
          reset({ value: value ?? defaultString });
        }
      }}
    >
      <PopoverTrigger className="w-min" asChild>
        <TouchButton
          // Display the current value in the trigger button
          className={`flex w-min flex-col items-center gap-4 py-4 ${
            open ? "bg-slate-100" : ""
          }`}
          variant="outline"
          disabled={!valueIsDefined}
        >
          <div className="flex flex-row items-center gap-2">
            <span className="text-xl font-bold">{value}</span>
          </div>
          <Separator orientation="vertical" className="mx-4" />
          <Icon name="lu:Pencil" className="size-6" />
        </TouchButton>
      </PopoverTrigger>
      {valueIsDefined && (
        <PopoverContent className="mx-8 flex w-auto rounded-2xl p-0 shadow-2xl">
          <div className="flex min-w-[300px] flex-col gap-6 p-6">
            {/* Header */}
            <div className="flex flex-row items-center gap-2 text-xl">
              <Icon name={icon ?? "lu:Pencil"} className="size-6" />
              <span>{title}</span>
            </div>
            {description && <span>{description}</span>}
            <Separator />

            {/* Input Row */}
            <div className="flex w-full flex-row items-start justify-center gap-4">
              <TouchInput
                {...register("value")} // Register the input with react-hook-form
                className="py-6 text-xl"
                // No value prop here; react-hook-form handles it via register
              />
            </div>

            <Separator />

            {/* Action Buttons */}
            <div className="flex flex-row gap-4">
              <TouchButton
                variant="outline"
                icon="lu:X"
                className="h-16 flex-1"
                onClick={handleAbort}
              >
                Abort
              </TouchButton>

              <div className="flex flex-1 flex-col gap-2">
                <TouchButton
                  variant="default"
                  icon="lu:Save"
                  className="h-16 w-full"
                  onClick={handleSubmit(onSubmit)}
                  disabled={!formState.isValid}
                >
                  Save
                </TouchButton>
                {/* Error Message */}
                {!formState.isValid && formState.errors.value?.message && (
                  <IconText variant="error" icon={"lu:TriangleAlert"}>
                    {formState.errors.value.message}
                  </IconText>
                )}
              </div>

              {/* Default Button */}
              {defaultValue !== undefined && (
                <TouchButton
                  variant="outline"
                  icon="lu:Undo2"
                  className="h-16 flex-1"
                  onClick={handleResetToDefault}
                >
                  <span className="font-mono">{defaultValue}</span> Default
                </TouchButton>
              )}
            </div>
          </div>
        </PopoverContent>
      )}
    </Popover>
  );
}
