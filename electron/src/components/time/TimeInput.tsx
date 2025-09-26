import React, { useState, useCallback, useEffect } from "react";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon } from "@/components/Icon";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Separator } from "@/components/ui/separator";

type TimeInputProps = {
  timestamp?: number | null;
  onTimeChange?: (timestamp: number | null) => void;
  onClear?: () => void;
};

const timeSchema = z.object({
  hours: z.number().min(0).max(23),
  minutes: z.number().min(0).max(59),
  seconds: z.number().min(0).max(59),
});

type TimeFormData = z.infer<typeof timeSchema>;

export function TimeInput({
  timestamp,
  onTimeChange,
  onClear,
}: TimeInputProps) {
  const [isOpen, setIsOpen] = useState(false);

  // Get current time for initial values
  const getCurrentTime = useCallback(() => {
    const now = new Date();
    return {
      hours: now.getHours(),
      minutes: now.getMinutes(),
      seconds: now.getSeconds(),
    };
  }, []);

  // Get initial values based on timestamp or current time
  const getInitialValues = useCallback(() => {
    if (timestamp) {
      const date = new Date(timestamp);
      return {
        hours: date.getHours(),
        minutes: date.getMinutes(),
        seconds: date.getSeconds(),
      };
    }
    return getCurrentTime();
  }, [timestamp, getCurrentTime]);

  const form = useForm<TimeFormData>({
    resolver: zodResolver(timeSchema),
    defaultValues: getInitialValues(),
    mode: "all",
  });

  // Convert timestamp to time components
  const timestampToTime = useCallback((ts: number) => {
    const date = new Date(ts);
    return {
      hours: date.getHours(),
      minutes: date.getMinutes(),
      seconds: date.getSeconds(),
    };
  }, []);

  // Check if the selected time would be in the future (today)
  const isTimeInFuture = useCallback((time: TimeFormData) => {
    const now = new Date();
    const date = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate(),
      time.hours,
      time.minutes,
      time.seconds,
    );
    return date.getTime() > now.getTime();
  }, []);

  // Convert time components to timestamp (today's date only)
  const timeToTimestamp = useCallback((time: TimeFormData) => {
    const now = new Date();
    const date = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate(),
      time.hours,
      time.minutes,
      time.seconds,
    );

    return date.getTime();
  }, []);

  // Initialize form with timestamp or current time when dialog opens
  useEffect(() => {
    if (isOpen) {
      const initialValues = getInitialValues();
      form.reset(initialValues);
    }
  }, [isOpen, getInitialValues, form]);

  const formatTimeDisplay = useCallback(() => {
    if (!timestamp) return "Show from Time";
    const time = timestampToTime(timestamp);
    return `${time.hours.toString().padStart(2, "0")}:${time.minutes.toString().padStart(2, "0")}:${time.seconds.toString().padStart(2, "0")}`;
  }, [timestamp, timestampToTime]);

  // Increment/decrement functions with bounds checking and cascading
  const adjustTimeValue = useCallback(
    (field: "hours" | "minutes" | "seconds", increment: boolean) => {
      // Define the order of fields for cascading
      const fields: ("hours" | "minutes" | "seconds")[] = [
        "seconds",
        "minutes",
        "hours",
      ];
      const startIndex = fields.indexOf(field);
      let carry = increment ? 1 : -1;

      for (let i = startIndex; i < fields.length; i++) {
        const currentField = fields[i];
        const currentValue = form.getValues(currentField);
        const max = currentField === "hours" ? 23 : 59;
        let newValue = currentValue + carry;
        carry = 0;

        if (increment) {
          if (newValue > max) {
            newValue = 0;
            carry = 1;
          }
        } else {
          if (newValue < 0) {
            newValue = max;
            carry = -1;
          }
        }

        form.setValue(currentField, newValue, {
          shouldValidate: true,
          shouldDirty: true,
          shouldTouch: true,
        });

        // If no carry, stop cascading
        if (carry === 0) {
          break;
        }
      }
    },
    [form],
  );

  const handleSubmit = () => {
    form.handleSubmit((data) => {
      // Prevent submission if time is in the future
      if (isTimeInFuture(data)) {
        return;
      }

      const newTimestamp = timeToTimestamp(data);
      onTimeChange?.(newTimestamp);
      setIsOpen(false);
    })();
  };

  const handleAbort = () => {
    if (timestamp) {
      const time = timestampToTime(timestamp);
      form.reset(time);
    } else {
      const currentTime = getCurrentTime();
      form.reset(currentTime);
    }
    setIsOpen(false);
  };

  const handleClear = () => {
    onClear?.();
    setIsOpen(false);
  };

  // Watch individual form fields for real-time updates
  const hours = useWatch({
    control: form.control,
    name: "hours",
    defaultValue: getInitialValues().hours,
  });

  const minutes = useWatch({
    control: form.control,
    name: "minutes",
    defaultValue: getInitialValues().minutes,
  });

  const seconds = useWatch({
    control: form.control,
    name: "seconds",
    defaultValue: getInitialValues().seconds,
  });

  // Get current form values with fallback
  const currentValues = {
    hours: hours ?? 0,
    minutes: minutes ?? 0,
    seconds: seconds ?? 0,
  };

  // TimeValueInput component for individual time fields
  const TimeValueInput = ({
    field,
    label,
    value,
  }: {
    field: "hours" | "minutes" | "seconds";
    label: string;
    value: number;
  }) => (
    <div className="flex flex-col items-center gap-3">
      <label className="text-sm font-medium text-gray-700">{label}</label>
      <div className="flex flex-col items-center gap-2">
        <TouchButton
          variant="outline"
          size="sm"
          className="h-10 w-16 p-0"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            adjustTimeValue(field, true);
          }}
        >
          <Icon name="lu:ChevronUp" className="size-4" />
        </TouchButton>
        <div className="flex h-16 w-20 items-center justify-center rounded border-2 border-gray-300 bg-white font-mono text-3xl">
          {value.toString().padStart(2, "0")}
        </div>
        <TouchButton
          variant="outline"
          size="sm"
          className="h-10 w-16 p-0"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            adjustTimeValue(field, false);
          }}
        >
          <Icon name="lu:ChevronDown" className="size-4" />
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
          <Icon name="lu:Clock" className="mr-2 size-4" />
          {formatTimeDisplay()}
        </TouchButton>
      </DialogTrigger>

      <DialogContent className="max-w-2xl p-8">
        <div className="flex flex-col gap-8">
          {/* Header */}
          <div className="flex min-h-[2.5rem] items-center justify-between">
            <div className="flex items-center gap-3">
              <Icon name="lu:Clock" className="size-6" />
              <h3 className="text-xl font-semibold">Show from Time</h3>
            </div>
            {timestamp && (
              <TouchButton
                variant="ghost"
                size="sm"
                onClick={handleClear}
                className="text-red-600 hover:text-red-700"
              >
                <Icon name="lu:X" className="mr-2 size-4" />
                Clear
              </TouchButton>
            )}
          </div>

          {/* Description */}
          <div className="-mt-4 text-base text-gray-600">
            Set a specific time to display data from that point forward in the
            graph.
          </div>

          {/* Error for future time */}
          {isTimeInFuture(currentValues) && (
            <div className="rounded-lg border border-red-200 bg-red-50 p-4">
              <div className="flex items-start gap-3">
                <Icon name="lu:X" className="mt-0.5 size-5 text-red-600" />
                <div>
                  <div className="font-medium text-red-800">
                    Cannot set future time
                  </div>
                  <div className="mt-1 text-sm text-red-700">
                    The selected time is in the future. Please choose a time
                    that has already occurred today.
                  </div>
                </div>
              </div>
            </div>
          )}

          <Separator />

          {/* Time inputs */}
          <div className="flex items-center justify-center gap-8">
            <TimeValueInput
              field="hours"
              label="Hours"
              value={currentValues.hours}
            />
            <div className="mt-8 text-4xl font-bold">:</div>
            <TimeValueInput
              field="minutes"
              label="Minutes"
              value={currentValues.minutes}
            />
            <div className="mt-8 text-4xl font-bold">:</div>
            <TimeValueInput
              field="seconds"
              label="Seconds"
              value={currentValues.seconds}
            />
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
              className={`h-14 flex-1 ${
                isTimeInFuture(currentValues)
                  ? "cursor-not-allowed bg-gray-300 text-gray-500"
                  : "bg-blue-600 text-white hover:bg-blue-700"
              }`}
              onClick={handleSubmit}
              disabled={isTimeInFuture(currentValues)}
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
