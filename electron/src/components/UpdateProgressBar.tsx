import React from "react";
import { Icon } from "./Icon";
import { cn } from "@/lib/utils";
import type { UpdateStep } from "@/stores/updateStore";

interface UpdateProgressBarProps {
  steps: UpdateStep[];
  overallProgress: number;
  className?: string;
}

const subsectorColors = {
  nixos: "text-blue-600 dark:text-blue-400",
  rust: "text-orange-600 dark:text-orange-400",
  electron: "text-purple-600 dark:text-purple-400",
  general: "text-gray-600 dark:text-gray-400",
};

const subsectorBgColors = {
  nixos: "bg-blue-100 dark:bg-blue-950",
  rust: "bg-orange-100 dark:bg-orange-950",
  electron: "bg-purple-100 dark:bg-purple-950",
  general: "bg-gray-100 dark:bg-gray-950",
};

const subsectorBorderColors = {
  nixos: "border-blue-200 dark:border-blue-900",
  rust: "border-orange-200 dark:border-orange-900",
  electron: "border-purple-200 dark:border-purple-900",
  general: "border-gray-200 dark:border-gray-900",
};

export function UpdateProgressBar({
  steps,
  overallProgress,
  className,
}: UpdateProgressBarProps) {
  return (
    <div className={cn("space-y-6", className)}>
      {/* Overall Progress Bar */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-base font-semibold">Overall Progress</span>
          <span className="text-lg font-bold tabular-nums">{overallProgress}%</span>
        </div>
        <div className="relative h-4 w-full overflow-hidden rounded-full bg-gray-200 shadow-inner dark:bg-gray-800">
          <div
            className="h-full rounded-full bg-gradient-to-r from-blue-500 via-orange-500 to-purple-500 shadow-sm transition-all duration-500 ease-out"
            style={{ width: `${overallProgress}%` }}
          />
        </div>
      </div>

      {/* Steps List */}
      <div className="space-y-3">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-gray-700 dark:text-gray-300">
          Update Steps
        </h3>
        <div className="space-y-2">
          {steps.map((step, index) => (
            <div
              key={step.id}
              className={cn(
                "flex items-start gap-3 rounded-lg border-2 p-3 transition-all duration-300",
                step.status === "in-progress" &&
                  "border-blue-400 bg-blue-50 shadow-md dark:border-blue-600 dark:bg-blue-950/50",
                step.status === "completed" &&
                  "border-green-400 bg-green-50 dark:border-green-600 dark:bg-green-950/50",
                step.status === "error" &&
                  "border-red-400 bg-red-50 dark:border-red-600 dark:bg-red-950/50",
                step.status === "pending" &&
                  "border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-900/50",
              )}
            >
              {/* Status Icon */}
              <div className="flex-shrink-0 pt-0.5">
                {step.status === "pending" && (
                  <div className="size-6 rounded-full border-2 border-gray-300 dark:border-gray-700" />
                )}
                {step.status === "in-progress" && (
                  <Icon
                    name="lu:Loader"
                    className="size-6 animate-spin text-blue-500"
                  />
                )}
                {step.status === "completed" && (
                  <div className="flex size-6 items-center justify-center rounded-full bg-green-500">
                    <Icon
                      name="lu:Check"
                      className="size-4 text-white"
                    />
                  </div>
                )}
                {step.status === "error" && (
                  <div className="flex size-6 items-center justify-center rounded-full bg-red-500">
                    <Icon name="lu:X" className="size-4 text-white" />
                  </div>
                )}
              </div>

              {/* Step Content */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span
                    className={cn(
                      "text-sm font-medium",
                      step.status === "completed" && "text-green-700 dark:text-green-300",
                      step.status === "error" && "text-red-700 dark:text-red-300",
                      step.status === "in-progress" && "font-semibold text-blue-700 dark:text-blue-300",
                      step.status === "pending" && "text-gray-600 dark:text-gray-400",
                    )}
                  >
                    {step.label}
                  </span>
                </div>
                
                {/* Subsector Badge */}
                <div className="mt-1.5">
                  <span
                    className={cn(
                      "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold",
                      subsectorBgColors[step.subsector],
                      subsectorColors[step.subsector],
                      subsectorBorderColors[step.subsector],
                    )}
                  >
                    {step.subsector.toUpperCase()}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
