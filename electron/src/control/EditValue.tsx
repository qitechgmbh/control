import { Icon, IconName } from "@/components/Icon";
import React, { useEffect } from "react";
import {
  getUnitIcon,
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
import { useFormValues } from "@/hooks/useFormValues";
import { TouchInput } from "@/components/touch/TouchInput";
import { IconText } from "@/components/IconText";
import { cva } from "class-variance-authority";
import { z } from "zod";

type Props = {
  unit?: Unit;
  value: number;
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
}: Props) {
  const formSchema = z.object({
    value: schema ?? z.number(),
  });
  type FormSchema = z.infer<typeof formSchema>;
  const form = useForm<FormSchema>({
    resolver: zodResolver<FormSchema>(formSchema),
    values: { value },
    defaultValues: {
      value: defaultValue,
    },
    mode: "all",
  });
  const formValues = useFormValues(form);
  const { value: formValue } = formValues;

  const handleAbort = () => {
    form.reset();
    setOpen(false);
  };

  const handleSubmit = () => {
    form.handleSubmit((data) => {
      console.log(data);
    })();
    setOpen(false);
  };

  const [open, setOpen] = React.useState(false);

  const setValue = (value: number) => {
    form.setValue("value", value);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger className="w-min">
        <TouchButton className={buttonStyle({ open })} variant="outline">
          <div className="flex flex-row items-center gap-2">
            <span className="font-mono text-4xl font-bold">
              {renderValue(value)}
            </span>
            <span>{renderUnitSymbol(unit)}</span>
          </div>
          <Separator orientation="vertical" className="mx-4" />
          <Icon name="lu:Pencil" className="size-6" />
        </TouchButton>
      </PopoverTrigger>
      <PopoverContent className="mx-8 flex w-min flex-col gap-6 rounded-2xl p-6 shadow-2xl">
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
            <EditValueText
              formValue={formValue}
              setFormValue={(value) => setValue(value)}
              valueSchema={schema}
              min={min}
              max={max}
            />
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
              onClick={handleAbort}
            >
              <span className="font-mono">
                {renderUnitSyntax(renderValue(defaultValue), unit)}
              </span>
              {unit ? " " + renderUnitSymbol(unit) : ""} Default
            </TouchButton>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}

type EditValueTextProps = {
  formValue: number;
  setFormValue: (value: number) => void;
  valueSchema?: z.ZodType<number>;
  min?: number;
  max?: number;
};

const inputStyle = cva("font-mono text-2xl", {
  variants: {
    error: {
      true: "text-red-500",
    },
  },
});

function EditValueText({
  formValue,
  setFormValue,
  valueSchema,
  min,
  max,
}: EditValueTextProps) {
  // string value for input
  const [valueString, setValueString] = React.useState("");
  const [valueStringDirty, setValueStringDirty] = React.useState(false); // prevent update loop
  const [valueStringError, setValueStringError] = React.useState(false);

  // form to input
  useEffect(() => {
    setValueString(formValue?.toString());
  }, [formValue]);

  // input to form
  useEffect(() => {
    if (!valueStringDirty) return;
    // replace , with .
    const newValueString = valueString.replace(/,/g, ".");
    const floatRegex = /^-?\d+(\.\d*)?$/;

    if (
      !floatRegex.test(newValueString) ||
      (valueSchema &&
        !valueSchema.safeParse(parseFloat(newValueString)).success) ||
      (max !== undefined && parseFloat(newValueString) > max) ||
      (min !== undefined && parseFloat(newValueString) < min)
    ) {
      setValueStringError(true);
    } else {
      const newValue = parseFloat(newValueString);
      setFormValue(newValue);
      setValueStringDirty(false);
      setValueStringError(false);
    }
  }, [valueString]);

  return (
    <div>
      <TouchInput
        className={inputStyle({ error: valueStringError })}
        value={valueString}
        onChange={(e) => {
          const value = e.target.value;
          setValueString(value);
          setValueStringDirty(true);
        }}
        onBlur={() => {
          if (valueStringError) {
            setValueString(formValue.toString());
            setValueStringError(false);
            setValueStringDirty(false);
          }
        }}
      />
    </div>
  );
}
