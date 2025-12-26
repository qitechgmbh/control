import React, { useMemo } from "react";
import { Icon } from "./Icon";
import { cn } from "@/lib/utils";
import type { UpdateStep } from "@/stores/updateStore";
import { useUpdateStore } from "@/stores/updateStore";

interface UpdateProgressBarProps {
  steps: UpdateStep[];
  overallProgress: number;
  className?: string;
}

export function UpdateProgressBar({
  steps,
  overallProgress,
  className,
}: UpdateProgressBarProps) {
  const startTime = useUpdateStore((state) => state.startTime);

  const timeEstimate = useMemo(() => {
    if (!startTime || overallProgress === 0) {
      return null;
    }

    const elapsedTime = Date.now() - startTime;
    const elapsedSeconds = elapsedTime / 1000;

    // Wait at least 5 seconds and have at least 5% progress before showing estimate
    if (elapsedSeconds < 5 || overallProgress < 5) {
      return null;
    }

    // Simple estimate: if we've done X% in Y seconds, remaining is (Y / X) * (100 - X)
    const remainingSeconds =
      (elapsedSeconds / overallProgress) * (100 - overallProgress);

    // Apply reasonable bounds: minimum 30 seconds, maximum 30 minutes
    const boundedSeconds = Math.max(30, Math.min(1800, remainingSeconds));

    return Math.round(boundedSeconds);
  }, [startTime, overallProgress]);

  const formatTime = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds}s`;
    }
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;

    // Round to nearest minute for longer estimates (5+ minutes) for simplicity
    if (minutes >= 5 && remainingSeconds >= 30) {
      return `${minutes + 1}m`;
    }
    if (remainingSeconds === 0) {
      return `${minutes}m`;
    }
    return `${minutes}m ${remainingSeconds}s`;
  };

  return (
    <div className={cn("space-y-4", className)}>
      {/* Overall Progress Bar */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Overall Progress</span>
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold tabular-nums">
              {overallProgress}%
            </span>
            {timeEstimate !== null && (
              <span className="text-xs text-gray-500 tabular-nums dark:text-gray-400">
                ~{formatTime(timeEstimate)} remaining
              </span>
            )}
          </div>
        </div>
        <div className="relative h-2.5 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-800">
          <div
            className="h-full rounded-full bg-gray-900 transition-all duration-500 ease-out dark:bg-gray-100"
            style={{ width: `${overallProgress}%` }}
          />
        </div>
      </div>

      {/* Steps List */}
      <div className="space-y-2">
        {steps.map((step) => (
          <div
            key={step.id}
            className={cn(
              "flex items-center gap-3 rounded border px-3 py-2.5 transition-all duration-200",
              step.status === "in-progress" &&
                "border-gray-400 bg-gray-50 dark:border-gray-600 dark:bg-gray-900",
              step.status === "completed" &&
                "border-gray-300 bg-white dark:border-gray-700 dark:bg-gray-950",
              step.status === "error" &&
                "border-red-300 bg-red-50 dark:border-red-800 dark:bg-red-950/30",
              step.status === "pending" &&
                "border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-950",
            )}
          >
            {/* Status Icon */}
            <div className="flex-shrink-0">
              {step.status === "pending" && (
                <div className="size-5 rounded-full border-2 border-gray-300 dark:border-gray-700" />
              )}
              {step.status === "in-progress" && (
                <Icon
                  name="lu:Loader"
                  className="size-5 animate-spin text-gray-600 dark:text-gray-400"
                />
              )}
              {step.status === "completed" && (
                <div className="flex size-5 items-center justify-center rounded-full bg-gray-700 dark:bg-gray-300">
                  <Icon
                    name="lu:Check"
                    className="size-3 text-white dark:text-gray-900"
                  />
                </div>
              )}
              {step.status === "error" && (
                <div className="flex size-5 items-center justify-center rounded-full bg-red-600 dark:bg-red-500">
                  <Icon name="lu:X" className="size-3 text-white" />
                </div>
              )}
            </div>

            {/* Step Content */}
            <div className="min-w-0 flex-1">
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <span
                    className={cn(
                      "text-sm",
                      step.status === "completed" &&
                        "text-gray-600 dark:text-gray-400",
                      step.status === "error" &&
                        "font-medium text-red-700 dark:text-red-400",
                      step.status === "in-progress" &&
                        "font-medium text-gray-900 dark:text-gray-100",
                      step.status === "pending" &&
                        "text-gray-500 dark:text-gray-500",
                    )}
                  >
                    {step.label}
                  </span>
                  {/* Subsector Badge - minimal inline display */}
                  <span
                    className={cn("text-xs text-gray-400 dark:text-gray-600")}
                  >
                    (
                    {step.id === "rust-build"
                      ? "rust & electron"
                      : step.subsector}
                    )
                  </span>
                </div>
                {/* Progress percentage for in-progress steps */}
                {step.status === "in-progress" &&
                  step.progress !== undefined &&
                  step.progress > 0 && (
                    <span className="text-xs font-medium text-gray-600 tabular-nums dark:text-gray-400">
                      {Math.round(step.progress)}%
                    </span>
                  )}
              </div>

              {/* Progress bar for steps with detailed tracking */}
              {step.status === "in-progress" &&
                step.progress !== undefined &&
                step.progress > 0 && (
                  <div className="mt-2">
                    <div className="h-1.5 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
                      <div
                        className="h-full rounded-full bg-gray-700 transition-all duration-500 dark:bg-gray-300"
                        style={{ width: `${step.progress}%` }}
                      />
                    </div>
                  </div>
                )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
