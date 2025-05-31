import { Icon, IconName } from "@/components/Icon";
import React, { useEffect } from "react";
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
  defaultValue: number;
  min?: number;
  minLabel?: string;
  max?: number;
  maxLabel?: string;
  step?: number;
  valueSchema?: z.ZodType<number>;
  inverted?: boolean;
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
  onChange,
}: Props) {
  const formSchema = z.object({
    value: schema ?? z.number(),
  });
  type FormSchema = z.infer<typeof formSchema>;
  const form = useForm<FormSchema>({
    resolver: zodResolver<FormSchema>(formSchema),
    values: { value: value ?? defaultValue },
    defaultValues: {
      value: defaultValue,
    },
    mode: "all",
  });
  const formValues = useFormValues(form);
  const { value: formValue } = formValues;

  // Input state management
  const [valueString, setValueString] = React.useState("");
  const [valueStringDirty, setValueStringDirty] = React.useState(false);
  const [valueStringError, setValueStringError] = React.useState(false);
  const inputRef = React.useRef<HTMLInputElement>(null);

  // Modal and numpad state
  const [open, setOpen] = React.useState(false);
  const [numpadExtended, setNumpadExtended] = React.useState(false);

  // if external value changes
  // for example from undefined to defined
  // we reset the form
  useEffect(() => {
    form.reset({
      value: value ?? defaultValue,
    });
  }, [value, defaultValue]);

  // form to input sync
  useEffect(() => {
    // Only update from form if user is not actively editing
    if (!valueStringDirty) {
      const newValueString =
        formValue !== undefined && formValue !== null
          ? formValue.toString()
          : "";
      setValueString(newValueString);
    }
  }, [formValue, valueStringDirty]);

  // input to form sync
  useEffect(() => {
    if (!valueStringDirty) return;
    // replace , with .
    const newValueString = valueString.replace(/,/g, ".");

    // Handle empty string case - don't reset dirty state immediately
    if (newValueString === "" || newValueString === "-") {
      setValueStringError(false);
      // Keep dirty state true for empty values to prevent form from overwriting
      return;
    }

    // Regex that properly handles integers and decimals
    // Allows: 123, -123, 123., 123.45, .45, -.45
    const floatRegex = /^-?(\d+\.?\d*|\.\d+)$/;

    // Check if it's a valid number format
    if (!floatRegex.test(newValueString)) {
      setValueStringError(true);
      return;
    }

    // Parse the number early to check bounds even for trailing decimals
    const numericValue = parseFloat(newValueString);

    // Check bounds even for incomplete numbers (like "1010.")
    const hasError =
      isNaN(numericValue) ||
      (schema && !schema.safeParse(numericValue).success) ||
      (max !== undefined && numericValue > max) ||
      (min !== undefined && numericValue < min);

    // Handle trailing decimal point case (e.g., "5." should be allowed but not converted yet)
    // BUT still show error if it violates bounds
    if (newValueString.endsWith(".") && /^\d+\.$/.test(newValueString)) {
      setValueStringError(hasError);
      // Keep dirty state true to prevent form from overwriting the trailing decimal
      return;
    }

    // Handle trailing zeros after decimal (e.g., "5.0" should not auto-convert to prevent "5.05" issues)
    if (/\.\d*0+$/.test(newValueString)) {
      setValueStringError(hasError);
      // Keep dirty state true to prevent form from auto-converting "5.0" to "5"
      return;
    }

    if (hasError) {
      setValueStringError(true);
    } else {
      form.setValue("value", numericValue);
      setValueStringDirty(false);
      setValueStringError(false);
    }
  }, [valueString, valueStringDirty, schema, max, min, form]);

  // Reset function for closing modal
  const resetInput = React.useCallback(() => {
    setValueStringDirty(false);
    setValueStringError(false);
  }, []);

  const handleAbort = () => {
    // Reset form to the current external value (not defaultValue)
    form.reset({
      value: value ?? defaultValue,
    });
    setOpen(false);
    setNumpadExtended(false);
    resetInput();
  };

  const handleSubmit = () => {
    form.handleSubmit((data) => {
      onChange?.(data.value);
      setOpen(false);
      setNumpadExtended(false);
    })();
  };

  // Create stable numpad handlers with cursor position support
  const numpadHandlers = React.useMemo(() => {
    // Helper function to ensure input has focus
    const ensureFocus = () => {
      if (inputRef.current && document.activeElement !== inputRef.current) {
        inputRef.current.focus();
      }
    };

    return {
      appendDigit: (digit: string) => {
        if (inputRef.current) {
          ensureFocus();
          const input = inputRef.current;
          const start = input.selectionStart || 0;
          const end = input.selectionEnd || 0;
          const currentValue = valueString;
          const newValue =
            currentValue.slice(0, start) + digit + currentValue.slice(end);
          setValueString(newValue);
          setValueStringDirty(true);

          // Set cursor position after the inserted digit
          setTimeout(() => {
            if (input) {
              input.setSelectionRange(start + 1, start + 1);
            }
          }, 0);
        }
      },
      addDecimal: () => {
        if (inputRef.current && !valueString.includes(".")) {
          ensureFocus();
          const input = inputRef.current;
          const start = input.selectionStart || 0;
          const end = input.selectionEnd || 0;
          const currentValue = valueString;
          const newValue =
            currentValue.slice(0, start) + "." + currentValue.slice(end);
          setValueString(newValue);
          setValueStringDirty(true);

          // Set cursor position after the decimal point
          setTimeout(() => {
            if (input) {
              input.setSelectionRange(start + 1, start + 1);
            }
          }, 0);
        }
      },
      deleteChar: () => {
        if (inputRef.current) {
          ensureFocus();
          const input = inputRef.current;
          const start = input.selectionStart || 0;
          const end = input.selectionEnd || 0;
          const currentValue = valueString;

          if (start !== end) {
            // Delete selection
            const newValue =
              currentValue.slice(0, start) + currentValue.slice(end);
            setValueString(newValue);
            setValueStringDirty(true);
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(start, start);
              }
            }, 0);
          } else if (start > 0) {
            // Delete character before cursor (normal backspace behavior)
            const newValue =
              currentValue.slice(0, start - 1) + currentValue.slice(start);
            setValueString(newValue);
            setValueStringDirty(true);
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(start - 1, start - 1);
              }
            }, 0);
          } else if (start === 0 && currentValue.length > 0) {
            // Cursor is at beginning, delete character to the right (like Delete key)
            const newValue = currentValue.slice(1);
            setValueString(newValue);
            setValueStringDirty(true);
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(0, 0);
              }
            }, 0);
          }
        }
      },
      toggleSign: () => {
        if (inputRef.current) {
          ensureFocus();
          const input = inputRef.current;
          const currentValue = valueString;

          if (currentValue === "" || currentValue === "0") {
            // For empty or zero, start with negative
            setValueString("-");
            setValueStringDirty(true);
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(1, 1);
              }
            }, 0);
          } else if (currentValue.startsWith("-")) {
            // Remove negative sign
            const newValue = currentValue.slice(1);
            setValueString(newValue);
            setValueStringDirty(true);
            // Keep cursor in same relative position
            const currentPos = input.selectionStart || 0;
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(
                  Math.max(0, currentPos - 1),
                  Math.max(0, currentPos - 1),
                );
              }
            }, 0);
          } else {
            // Add negative sign
            const newValue = "-" + currentValue;
            setValueString(newValue);
            setValueStringDirty(true);
            // Keep cursor in same relative position
            const currentPos = input.selectionStart || 0;
            setTimeout(() => {
              if (input) {
                input.setSelectionRange(currentPos + 1, currentPos + 1);
              }
            }, 0);
          }
        }
      },
      moveCursorLeft: () => {
        if (inputRef.current) {
          ensureFocus();
          const input = inputRef.current;
          const currentPos = input.selectionStart || 0;
          if (currentPos > 0) {
            input.setSelectionRange(currentPos - 1, currentPos - 1);
          }
        }
      },
      moveCursorRight: () => {
        if (inputRef.current) {
          ensureFocus();
          const input = inputRef.current;
          const currentPos = input.selectionStart || 0;
          const maxPos = valueString.length;
          if (currentPos < maxPos) {
            input.setSelectionRange(currentPos + 1, currentPos + 1);
          }
        }
      },
    };
  }, [valueString]);

  const setValue = (value: number) => {
    form.setValue("value", value);
  };

  const valueIsDefined = value !== undefined && value !== null;

  return (
    <Popover
      open={open}
      onOpenChange={(isOpen) => {
        setOpen(isOpen);
        if (!isOpen) {
          setNumpadExtended(false);
          // Reset form and input to saved value when closing without saving
          form.reset({
            value: value ?? defaultValue,
          });
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
        <PopoverContent className="mx-8 flex w-min rounded-2xl p-0 shadow-2xl">
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
                      ? Math.max(min, formValue - step)
                      : formValue - step,
                  )
                }
              />
              <div className="flex flex-col items-center gap-2">
                <div>
                  <TouchInput
                    ref={inputRef}
                    className={`font-mono text-2xl transition-colors duration-200 ease-in-out ${valueStringError ? "text-red-500" : "text-black"}`}
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
                      // Always reset dirty state on blur to allow form sync
                      setValueStringDirty(false);
                      if (valueStringError) {
                        setValueString(formValue.toString());
                        setValueStringError(false);
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
                      ? Math.min(max, formValue + step)
                      : formValue + step,
                  )
                }
              />
            </div>
            <div className="py-0">
              <TouchSlider
                className="w-[48rem]"
                value={formValue ? [formValue] : undefined}
                onValueChange={(x) => setValue(x[0])}
                min={min}
                max={max}
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
                className="flex-1"
                onClick={handleAbort}
              >
                Abort
              </TouchButton>
              <div className="flex flex-1 flex-col gap-2">
                <TouchButton
                  variant="default"
                  icon="lu:Save"
                  className="w-full"
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
                  className="flex-1"
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
              numpadExtended ? "max-w-[420px] opacity-100" : "max-w-0 opacity-0"
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
