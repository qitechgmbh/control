import React, { useState, useCallback, useEffect } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Separator } from "@/components/ui/separator";

type RatioInputProps = {
  master?: number;
  slave?: number;
  onRatioChange?: (master: number, slave: number) => void;
  title?: string;
  min?: number;
  max?: number;
  step?: number;
};

const ratioSchema = z.object({
  master: z.number().min(0.1).max(1000),
  slave: z.number().min(0.1).max(1000),
});

type RatioFormData = z.infer<typeof ratioSchema>;

export function RatioInput({
  master = 1,
  slave = 1,
  onRatioChange,
  title = "Motor Ratio",
  min = 0.1,
  max = 1000,
  step = 0.1,
}: RatioInputProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [activeField, setActiveField] = useState<"master" | "slave" | null>(
    null,
  );
  const [inputValue, setInputValue] = useState<string>("");

  const getInitialValues = useCallback(() => {
    return {
      master: master ?? 1,
      slave: slave ?? 1,
    };
  }, [master, slave]);

  const form = useForm<RatioFormData>({
    resolver: zodResolver(ratioSchema),
    defaultValues: getInitialValues(),
    mode: "all",
  });

  // Initialize form with current values when dialog opens
  useEffect(() => {
    if (isOpen) {
      const initialValues = getInitialValues();
      form.reset(initialValues);
      setActiveField(null);
      setInputValue("");
    }
  }, [isOpen, getInitialValues, form]);

  const formatRatioDisplay = useCallback(() => {
    const m = master ?? 1;
    const s = slave ?? 1;
    return `${m.toFixed(1)} : ${s.toFixed(1)}`;
  }, [master, slave]);

  // Increment/decrement functions with bounds checking
  const adjustRatioValue = useCallback(
    (field: "master" | "slave", increment: boolean) => {
      const currentValue = form.getValues(field);
      let newValue = increment ? currentValue + step : currentValue - step;

      // Round to avoid floating point precision issues
      newValue = Math.round(newValue * 10) / 10;

      // Clamp to min/max bounds
      if (newValue < min) {
        newValue = min;
      } else if (newValue > max) {
        newValue = max;
      }

      form.setValue(field, newValue, {
        shouldValidate: true,
        shouldDirty: true,
        shouldTouch: true,
      });
    },
    [form, step, min, max],
  );

  const handleNumpadClick = (digit: string) => {
    if (activeField === null) return;

    let newValue = inputValue + digit;
    setInputValue(newValue);
  };

  const handleDecimal = () => {
    if (activeField === null) return;
    if (!inputValue.includes(".")) {
      setInputValue(inputValue === "" ? "0." : inputValue + ".");
    }
  };

  const handleBackspace = () => {
    setInputValue(inputValue.slice(0, -1));
  };

  const handleClear = () => {
    setInputValue("");
  };

  const handleNumpadConfirm = () => {
    if (activeField === null || inputValue === "") return;

    let numValue = parseFloat(inputValue);

    // Clamp to min/max bounds
    if (numValue < min) {
      numValue = min;
    } else if (numValue > max) {
      numValue = max;
    }

    form.setValue(activeField, numValue, {
      shouldValidate: true,
      shouldDirty: true,
      shouldTouch: true,
    });

    setInputValue("");
    setActiveField(null);
  };

  const handleFieldClick = (field: "master" | "slave") => {
    if (activeField === field) {
      // If same field, submit and close
      handleNumpadConfirm();
    } else {
      // Switch to new field
      if (activeField !== null && inputValue !== "") {
        handleNumpadConfirm();
      }
      setActiveField(field);
      setInputValue("");
    }
  };

  const handleSubmit = () => {
    if (activeField !== null && inputValue !== "") {
      handleNumpadConfirm();
      return;
    }

    form.handleSubmit((data) => {
      onRatioChange?.(data.master, data.slave);
      setIsOpen(false);
    })();
  };

  const handleAbort = () => {
    const initialValues = getInitialValues();
    form.reset(initialValues);
    setActiveField(null);
    setInputValue("");
    setIsOpen(false);
  };

  const handleReset = () => {
    form.setValue("master", 1, {
      shouldValidate: true,
      shouldDirty: true,
      shouldTouch: true,
    });
    form.setValue("slave", 1, {
      shouldValidate: true,
      shouldDirty: true,
      shouldTouch: true,
    });
    setActiveField(null);
    setInputValue("");
  };

  // Watch individual form fields for real-time updates
  const masterValue = useWatch({
    control: form.control,
    name: "master",
    defaultValue: getInitialValues().master,
  });

  const slaveValue = useWatch({
    control: form.control,
    name: "slave",
    defaultValue: getInitialValues().slave,
  });

  // Get current form values with fallback
  const currentValues = {
    master: masterValue ?? 1,
    slave: slaveValue ?? 1,
  };

  // RatioValueInput component for individual ratio fields
  const RatioValueInput = ({
    field,
    label,
    value,
  }: {
    field: "master" | "slave";
    label: string;
    value: number;
  }) => {
    const isActive = activeField === field;
    const displayValue = isActive && inputValue !== "" ? inputValue : value.toFixed(1);

    return (
      <div className="flex flex-col items-center gap-3">
        <label className="text-sm font-medium text-gray-700">{label}</label>
        <div className="flex flex-col items-center gap-2">
          <TouchButton
            variant="outline"
            size="sm"
            className="h-10 w-20 p-0"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              adjustRatioValue(field, true);
            }}
            disabled={value >= max}
          >
            <Icon name="lu:ChevronUp" className="size-4" />
          </TouchButton>
          <div
            onClick={() => handleFieldClick(field)}
            className={`flex h-16 w-24 cursor-pointer items-center justify-center rounded border-2 font-mono text-3xl transition-colors ${
              isActive
                ? "border-blue-500 bg-blue-50"
                : "border-gray-300 bg-white"
            }`}
          >
            {displayValue}
          </div>
          <TouchButton
            variant="outline"
            size="sm"
            className="h-10 w-20 p-0"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              adjustRatioValue(field, false);
            }}
            disabled={value <= min}
          >
            <Icon name="lu:ChevronDown" className="size-4" />
          </TouchButton>
        </div>
      </div>
    );
  };

  // Numpad component
  const Numpad = () => (
    <div className="flex flex-col gap-3">
      {/* Display showing input value */}
      <div className="flex h-12 items-center justify-center rounded border-2 border-gray-300 bg-gray-50 font-mono text-xl">
        {inputValue === "" ? "0" : inputValue}
      </div>

      {/* Numpad grid */}
      <div className="grid grid-cols-3 gap-2">
        {[
          ["1", "2", "3"],
          ["4", "5", "6"],
          ["7", "8", "9"],
          [".", "0", "DEL"],
        ].map((row, i) => (
          <div key={i} className="contents">
            {row.map((key) => (
              <TouchButton
                key={key}
                variant="outline"
                className="h-12 p-0"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  if (key === "DEL") {
                    handleBackspace();
                  } else if (key === ".") {
                    handleDecimal();
                  } else {
                    handleNumpadClick(key);
                  }
                }}
              >
                {key === "DEL" ? (
                  <Icon name="lu:Delete" className="size-4" />
                ) : (
                  key
                )}
              </TouchButton>
            ))}
          </div>
        ))}
      </div>

      {/* Control buttons */}
      <div className="flex gap-2">
        <TouchButton
          variant="outline"
          className="flex-1 h-10"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleClear();
          }}
        >
          C
        </TouchButton>
        <TouchButton
          className="flex-1 h-10 bg-green-600 text-white hover:bg-green-700"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleNumpadConfirm();
          }}
          disabled={activeField === null || inputValue === ""}
        >
          <Icon name="lu:Check" className="size-4" />
        </TouchButton>
      </div>
    </div>
  );

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <TouchButton
          variant="outline"
          className="h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50"
        >
          <Icon name="lu:Settings" className="mr-2 size-4" />
          {formatRatioDisplay()}
        </TouchButton>
      </DialogTrigger>

      <DialogContent className="max-w-4xl p-8">
        <div className="flex flex-col gap-8">
          {/* Header */}
          <div className="flex min-h-[2.5rem] items-center justify-between">
            <div className="flex items-center gap-3">
              <Icon name="lu:Settings" className="size-6" />
              <h3 className="text-xl font-semibold">{title}</h3>
            </div>
            <TouchButton
              variant="ghost"
              size="sm"
              onClick={handleReset}
              className="text-blue-600 hover:text-blue-700"
            >
              <Icon name="lu:RotateCcw" className="mr-2 size-4" />
              Reset to 1:1
            </TouchButton>
          </div>

          {/* Description */}
          <div className="-mt-4 text-base text-gray-600">
            Set the gear ratio. For every {currentValues.master.toFixed(1)}{" "}
            rotation(s) of the master puller, the motor will make{" "}
            {currentValues.slave.toFixed(1)} rotation(s).
          </div>

          <Separator />

          {/* Main content: Ratio inputs and Numpad */}
          <div className="flex gap-8">
            {/* Ratio inputs */}
            <div className="flex flex-1 items-center justify-center gap-8">
              <RatioValueInput
                field="master"
                label="Master"
                value={currentValues.master}
              />
              <div className="mt-8 text-4xl font-bold">:</div>
              <RatioValueInput
                field="slave"
                label="Slave"
                value={currentValues.slave}
              />
            </div>

            {/* Numpad */}
            <div className="w-40">
              <Numpad />
            </div>
          </div>

          <Separator />

          {/* Action buttons */}
          <div className="flex gap-4">
            <TouchButton
              variant="outline"
              className="h-14 flex-1"
              onClick={handleAbort}
            >
              <Icon name="lu:X" className="mr-2 size-4" />
              Cancel
            </TouchButton>
            <TouchButton
              className="h-14 flex-1 bg-blue-600 text-white hover:bg-blue-700"
              onClick={handleSubmit}
            >
              <Icon name="lu:Check" className="mr-2 size-4" />
              Apply
            </TouchButton>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
