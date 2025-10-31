import { Icon, IconName } from "@/components/Icon";
import React, { useEffect, useMemo } from "react";
import {
  getUnitIcon,
  renderValueToReactNode,
  renderUnitSymbol,
  renderUnitSymbolLong,
  renderUnitSyntax,
  Unit,
} from "./units";
import { TouchButton } from "@/components/touch/TouchButton";
import { Separator } from "@/components/ui/separator";
import { Popover, PopoverContent } from "@/components/ui/popover";
import { PopoverTrigger } from "@radix-ui/react-popover";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useFormValues } from "@/lib/useFormValues";
import { TouchInput } from "@/components/touch/TouchInput";
import { IconText } from "@/components/IconText";
import { cva } from "class-variance-authority";
import { z } from "zod";
import { TouchNumpad } from "@/components/touch/TouchNumpad";

type Props = {
  unit?: Unit;
  value?: number;
  title: string;
  description?: string;
  icon?: IconName;
  defaultValue?: number;
  min?: number;
  minSlider?: number; // Override the slider min value
  minLabel?: string;
  max?: number;
  maxSlider?: number; // Override the slider max value
  maxLabel?: string;
  step?: number;
  valueSchema?: z.ZodType<number>;
  inverted?: boolean;
  confirmation?: string;
  renderValue: (value: number) => string;
  onChange?: (value: number) => void;
};

const inputRowStyle = cva(
  "flex w-full flex-row items-start justify-center gap-4",
  {
    variants: {
      inverted: {
        true: "flex-row-reverse",
      },
    },
  },
);

const buttonStyle = cva("flex w-min flex-col items-center gap-4 ", {
  variants: {
    open: {
      true: "bg-slate-100",
    },
  },
});

/**
 * EditValue Component
 *
 * A comprehensive numeric input component with:
 * - Touch-friendly numpad interface
 * - Precision handling for floating point operations
 * - Real-time validation with visual feedback
 * - Decimal point manipulation (add/move)
 * - +/- increment buttons with step-based rounding
 * - Slider for range selection
 * - Unit display and formatting
 *
 * Key Features:
 * - Handles trailing decimals and zeros properly
 * - Prevents floating-point artifacts through step-based rounding
 * - Supports decimal point movement with cursor tracking
 * - Bidirectional sync between form state and input display
 * - Resets invalid inputs to last valid value on blur
 */
export function EditValue({
  unit,
  value,
  renderValue,
  title,
  description,
  defaultValue,
  step = 1,
  valueSchema: schema,
  inverted,
  min,
  max,
  minLabel,
  maxLabel,
  minSlider,
  maxSlider,
  confirmation,
  onChange,
}: Props) {
  const defaultOrZero = defaultValue ?? 0;

  // Form setup
  const formSchema = z.object({
    value: schema ?? z.number(),
  });
  type FormSchema = z.infer<typeof formSchema>;

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    values: { value: value ?? defaultOrZero },
    defaultValues: { value: defaultValue },
    mode: "all",
  });

  const formValues = useFormValues(form);
  const { value: formValue } = formValues;

  // Calculate step decimals for precise rounding
  const stepDecimals = useMemo(() => {
    const stepString = step.toString();
    const decimalIndex = stepString.indexOf(".");
    return decimalIndex === -1 ? 0 : stepString.length - decimalIndex - 1;
  }, [step]);

  // Helper function to round values according to step precision
  const roundToStepDecimals = React.useCallback(
    (value: number) => {
      const multiplier = Math.pow(10, stepDecimals);
      return Math.round(value * multiplier) / multiplier;
    },
    [stepDecimals],
  );

  // Input state management
  const [valueString, setValueString] = React.useState("");
  const [valueStringDirty, setValueStringDirty] = React.useState(false);
  const [valueStringError, setValueStringError] = React.useState(false);
  const inputRef = React.useRef<HTMLInputElement>(null);
  const preventFormSyncRef = React.useRef(false);

  // Modal and numpad state
  const [open, setOpen] = React.useState(false);
  const [numpadExtended, setNumpadExtended] = React.useState(false);

  // Reset external value changes (e.g., from undefined to defined)
  useEffect(() => {
    form.reset({ value: value ?? defaultValue });
  }, [value, defaultValue]);

  // Sync form value to input display (when form changes externally)
  useEffect(() => {
    const hasTrailingDecimal =
      valueString.endsWith(".") && /^\d+\.$/.test(valueString);

    // Only update input from form if user isn't editing and no special conditions
    if (
      !valueStringDirty &&
      !preventFormSyncRef.current &&
      !hasTrailingDecimal
    ) {
      const displayValue =
        formValue !== undefined && formValue !== null
          ? roundToStepDecimals(formValue).toString()
          : "";
      setValueString(displayValue);
    }
  }, [formValue, valueStringDirty, valueString, roundToStepDecimals]);

  // Sync input changes to form (when user types)
  useEffect(() => {
    if (!valueStringDirty) return;

    const normalizedInput = valueString.replace(/,/g, ".");

    // Handle empty or incomplete input
    if (normalizedInput === "" || normalizedInput === "-") {
      setValueStringError(false);
      return;
    }

    // Validate number format (allows trailing decimals and zeros)
    const floatRegex = /^-?(\d+\.?\d*|\.\d+)$/;
    if (!floatRegex.test(normalizedInput)) {
      setValueStringError(true);
      return;
    }

    const numericValue = parseFloat(normalizedInput);
    const hasValidationError =
      isNaN(numericValue) ||
      (schema && !schema.safeParse(numericValue).success) ||
      (max !== undefined && numericValue > max) ||
      (min !== undefined && numericValue < min);

    // Handle special cases: trailing decimal points and zeros
    const hasTrailingDecimal = /^\d+\.$/.test(normalizedInput);
    const hasTrailingZeros = /\.\d*0+$/.test(normalizedInput);

    if (hasTrailingDecimal || hasTrailingZeros) {
      setValueStringError(hasValidationError);
      if (!hasValidationError) {
        form.setValue("value", numericValue);
        // Keep dirty state for trailing decimals, clear for trailing zeros
        if (hasTrailingZeros && !hasTrailingDecimal) {
          setValueStringDirty(false);
          setValueStringError(false);
          preventFormSyncRef.current = false;
        }
      }
      return;
    }

    // Regular number update
    if (hasValidationError) {
      setValueStringError(true);
    } else {
      form.setValue("value", numericValue);
      setValueStringDirty(false);
      setValueStringError(false);
      preventFormSyncRef.current = false;
    }
  }, [
    valueString,
    valueStringDirty,
    schema,
    max,
    min,
    form,
    roundToStepDecimals,
  ]);

  // Reset input state to clean form value
  const resetInput = React.useCallback(() => {
    setValueStringDirty(false);
    setValueStringError(false);
    preventFormSyncRef.current = false;

    const cleanValue =
      formValue !== undefined && formValue !== null
        ? roundToStepDecimals(formValue).toString()
        : "";
    setValueString(cleanValue);
  }, [formValue, roundToStepDecimals]);

  const handleAbort = () => {
    form.reset({ value: value ?? defaultValue });
    setOpen(false);
    setNumpadExtended(false);
    resetInput();
  };

  const handleSubmit = () => {
    form.handleSubmit((data) => {
      if (confirmation) {
        if (window.confirm(confirmation)) {
          onChange?.(data.value);
          setOpen(false);
          setNumpadExtended(false);
          resetInput();
        }
        // else do nothing if user cancels
      } else {
        onChange?.(data.value);
        setOpen(false);
        setNumpadExtended(false);
        resetInput();
      }
    })();
  };

  // Numpad interaction handlers
  const numpadHandlers = React.useMemo(() => {
    const ensureFocus = () => {
      if (inputRef.current && document.activeElement !== inputRef.current) {
        inputRef.current.focus();
      }
    };

    const updateCursorPosition = (position: number) => {
      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.setSelectionRange(position, position);
        }
      }, 0);
    };

    return {
      appendDigit: (digit: string) => {
        if (!inputRef.current) return;

        ensureFocus();
        const input = inputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const newValue =
          valueString.slice(0, start) + digit + valueString.slice(end);

        setValueString(newValue);
        setValueStringDirty(true);
        updateCursorPosition(start + 1);
      },

      addDecimal: () => {
        if (!inputRef.current) return;

        ensureFocus();
        const input = inputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;

        if (!valueString.includes(".")) {
          // Add decimal at cursor position
          const newValue =
            valueString.slice(0, start) + "." + valueString.slice(end);
          setValueString(newValue);
          setValueStringDirty(true);
          updateCursorPosition(start + 1);
        } else {
          // Move existing decimal to cursor position
          const currentDecimalIndex = valueString.indexOf(".");
          const valueWithoutDecimal = valueString.replace(".", "");
          const adjustedStart = start > currentDecimalIndex ? start - 1 : start;
          const newValue =
            valueWithoutDecimal.slice(0, adjustedStart) +
            "." +
            valueWithoutDecimal.slice(adjustedStart);

          preventFormSyncRef.current = true;
          setValueStringDirty(true);
          setValueString(newValue);

          setTimeout(() => {
            preventFormSyncRef.current = false;
          }, 100);
          updateCursorPosition(adjustedStart + 1);
        }
      },

      deleteChar: () => {
        if (!inputRef.current) return;

        ensureFocus();
        const input = inputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;

        let newValue: string;
        let newPosition: number;

        if (start !== end) {
          // Delete selection
          newValue = valueString.slice(0, start) + valueString.slice(end);
          newPosition = start;
        } else if (start > 0) {
          // Backspace
          newValue = valueString.slice(0, start - 1) + valueString.slice(start);
          newPosition = start - 1;
        } else if (start === 0 && valueString.length > 0) {
          // Delete at beginning
          newValue = valueString.slice(1);
          newPosition = 0;
        } else {
          return;
        }

        setValueString(newValue);
        setValueStringDirty(true);
        updateCursorPosition(newPosition);
      },

      toggleSign: () => {
        if (!inputRef.current) return;

        ensureFocus();
        const input = inputRef.current;
        const currentPos = input.selectionStart || 0;

        let newValue: string;
        let newPosition: number;

        if (valueString === "" || valueString === "0") {
          newValue = "-";
          newPosition = 1;
        } else if (valueString.startsWith("-")) {
          newValue = valueString.slice(1);
          newPosition = Math.max(0, currentPos - 1);
        } else {
          newValue = "-" + valueString;
          newPosition = currentPos + 1;
        }

        setValueString(newValue);
        setValueStringDirty(true);
        updateCursorPosition(newPosition);
      },

      moveCursorLeft: () => {
        if (!inputRef.current) return;

        ensureFocus();
        const currentPos = inputRef.current.selectionStart || 0;
        if (currentPos > 0) {
          inputRef.current.setSelectionRange(currentPos - 1, currentPos - 1);
        }
      },

      moveCursorRight: () => {
        if (!inputRef.current) return;

        ensureFocus();
        const currentPos = inputRef.current.selectionStart || 0;
        if (currentPos < valueString.length) {
          inputRef.current.setSelectionRange(currentPos + 1, currentPos + 1);
        }
      },
    };
  }, [valueString]);

  const setValue = (value: number) => {
    form.setValue("value", value);
  };

  // Continuous increment/decrement functionality
  const intervalRef = React.useRef<NodeJS.Timeout | null>(null);
  const timeoutRef = React.useRef<NodeJS.Timeout | null>(null);

  const startContinuousChange = React.useCallback(
    (increment: boolean) => {
      const performChange = () => {
        const currentValue = form.getValues().value;
        const newValue = increment
          ? max !== undefined
            ? Math.min(max, roundToStepDecimals(currentValue + step))
            : roundToStepDecimals(currentValue + step)
          : min !== undefined
            ? Math.max(min, roundToStepDecimals(currentValue - step))
            : roundToStepDecimals(currentValue - step);
        setValue(newValue);
      };

      // Initial delay before starting continuous changes
      timeoutRef.current = setTimeout(() => {
        // Start continuous changes at regular intervals
        intervalRef.current = setInterval(performChange, 100); // Change every 100ms
      }, 500); // Wait 500ms before starting continuous changes
    },
    [step, min, max, roundToStepDecimals, form],
  );

  const stopContinuousChange = React.useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  // Cleanup on unmount
  React.useEffect(() => {
    return () => {
      stopContinuousChange();
    };
  }, [stopContinuousChange]);

  const valueIsDefined = value !== undefined && value !== null;

  return (
    <Popover
      open={open}
      onOpenChange={(isOpen) => {
        setOpen(isOpen);
        if (!isOpen) {
          setNumpadExtended(false);
          form.reset({ value: value ?? defaultValue });
          resetInput();
        }
      }}
    >
      <PopoverTrigger className="w-min" asChild>
        <TouchButton
          className={buttonStyle({ open, class: "py-4" })}
          variant="outline"
          disabled={!valueIsDefined}
        >
          <div className="flex flex-row items-center gap-2">
            <span className="font-mono text-4xl font-bold">
              {renderValueToReactNode(value, unit, renderValue)}
            </span>
            <span>{renderUnitSymbol(unit)}</span>
          </div>
          <Separator orientation="vertical" className="mx-4" />
          <Icon name="lu:Pencil" className="size-6" />
        </TouchButton>
      </PopoverTrigger>
      {valueIsDefined && (
        <PopoverContent className="mx-8 flex w-auto rounded-2xl p-0 shadow-2xl">
          <div className="flex flex-col gap-6 p-6">
            <div className="text-l flex flex-row items-center gap-2">
              <Icon
                name={unit ? getUnitIcon(unit) : "lu:Pencil"}
                className="size-6"
              />
              <span>{title}</span>
            </div>
            {description && <span>{description}</span>}
            <Separator />

            <div className={inputRowStyle({ inverted })}>
              <TouchButton
                icon="lu:Minus"
                variant="outline"
                onClick={() =>
                  setValue(
                    min !== undefined
                      ? Math.max(min, roundToStepDecimals(formValue - step))
                      : roundToStepDecimals(formValue - step),
                  )
                }
                onMouseDown={() => startContinuousChange(false)}
                onMouseUp={stopContinuousChange}
                onMouseLeave={stopContinuousChange}
                onTouchStart={() => startContinuousChange(false)}
                onTouchEnd={stopContinuousChange}
              />
              <div className="flex flex-col items-center gap-2">
                <div>
                  <TouchInput
                    ref={inputRef}
                    className={`py-9 font-mono text-2xl transition-colors duration-200 ease-in-out ${valueStringError ? "text-red-500" : "text-black"}`}
                    value={valueString}
                    onChange={(e) => {
                      const value = e.target.value;
                      setValueString(value);
                      setValueStringDirty(true);
                    }}
                    onFocus={() => {
                      setNumpadExtended(true);
                    }}
                    onBlur={() => {
                      setValueStringDirty(false);
                      if (valueStringError) {
                        // Reset to actual external value when input is invalid
                        const resetValue = value ?? defaultValue;
                        setValueString(resetValue.toString());
                        setValueStringError(false);
                        form.setValue("value", resetValue);
                      }
                    }}
                  />
                </div>
                {unit && (
                  <span className="text-gray-400 uppercase">
                    {renderUnitSymbolLong(unit)}
                  </span>
                )}
              </div>
              <TouchButton
                icon="lu:Plus"
                variant="outline"
                onClick={() =>
                  setValue(
                    max !== undefined
                      ? Math.min(max, roundToStepDecimals(formValue + step))
                      : roundToStepDecimals(formValue + step),
                  )
                }
                onMouseDown={() => startContinuousChange(true)}
                onMouseUp={stopContinuousChange}
                onMouseLeave={stopContinuousChange}
                onTouchStart={() => startContinuousChange(true)}
                onTouchEnd={stopContinuousChange}
              />
            </div>
            <div className="py-0">
              <TouchSlider
                className="w-[48rem]"
                value={formValue ? [formValue] : undefined}
                onValueChange={(x) => setValue(x[0])}
                min={minSlider ?? min}
                max={maxSlider ?? max}
                step={step}
                inverted={inverted}
                unit={unit}
                minLabel={minLabel}
                maxLabel={maxLabel}
                renderValue={renderValue}
              />
            </div>
            <Separator />
            <div className="flex flex-row gap-4">
              <TouchButton
                variant="outline"
                icon="lu:X"
                className="h-21 flex-1"
                onClick={handleAbort}
              >
                Abort
              </TouchButton>
              <div className="flex flex-1 flex-col gap-2">
                <TouchButton
                  variant="default"
                  icon="lu:Save"
                  className="h-21 w-full"
                  onClick={handleSubmit}
                >
                  Save
                </TouchButton>
                {!form.formState.isValid && (
                  <IconText variant="error" icon={"lu:TriangleAlert"}>
                    {form.formState.errors.value?.message}
                  </IconText>
                )}
              </div>

              {defaultValue !== undefined && (
                <TouchButton
                  variant="outline"
                  icon="lu:Undo2"
                  className="h-21 flex-1"
                  onClick={() => {
                    setValue(defaultValue);
                    handleSubmit();
                  }}
                >
                  <span className="font-mono">
                    {renderUnitSyntax(renderValue(defaultValue), unit)}
                  </span>
                  {unit ? " " + renderUnitSymbol(unit) : ""} Default
                </TouchButton>
              )}
            </div>
          </div>
          {/* Numpad Side */}
          <div
            className={`flex flex-row gap-6 overflow-hidden transition-all duration-300 ease-in-out ${
              numpadExtended ? "max-w-[28em] opacity-100" : "max-w-0 opacity-0"
            }`}
          >
            <div className="h-full w-[1px] flex-shrink-0 bg-neutral-200" />
            <div className="flex-shrink-0 p-6 pl-0">
              <TouchNumpad
                onDigit={numpadHandlers.appendDigit}
                onDecimal={numpadHandlers.addDecimal}
                onDelete={numpadHandlers.deleteChar}
                onToggleSign={numpadHandlers.toggleSign}
                onCursorLeft={numpadHandlers.moveCursorLeft}
                onCursorRight={numpadHandlers.moveCursorRight}
              />
            </div>
          </div>
        </PopoverContent>
      )}
    </Popover>
  );
}
