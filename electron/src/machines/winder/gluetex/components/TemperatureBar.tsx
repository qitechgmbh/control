import { cn } from "@/utils/tailwind";
import React from "react";

type Props = {
  min: number;
  max: number;
  minLimit: number;
  maxLimit: number;
  current: number;
  className?: string;
};

export function TemperatureBar({
  min,
  max,
  minLimit,
  maxLimit,
  current,
  className,
}: Props) {
  // Calculate percentages for positioning
  const range = max - min;
  const minLimitPercent = ((minLimit - min) / range) * 100;
  const maxLimitPercent = ((maxLimit - min) / range) * 100;
  const currentPercent = ((current - min) / range) * 100;

  // Determine if current value is within limits
  const isInRange = current >= minLimit && current <= maxLimit;

  return (
    <div className={cn("w-full space-y-1 pt-6", className)}>
      <div className="relative h-4 w-full">
        {/* Base bar with overflow hidden to clip inner bar */}
        <div className="absolute h-full w-full overflow-hidden rounded-full bg-gray-200">
          {/* Valid range (green section) */}
          <div
            className="absolute h-full bg-green-600"
            style={{
              left: `${minLimitPercent}%`,
              width: `${maxLimitPercent - minLimitPercent}%`,
            }}
          />
        </div>

        {/* Current temperature indicator - outside the overflow container so it's not clipped */}
        <div
          className={cn(
            "absolute -top-5 z-10 h-14 w-1.5 rounded-full",
            isInRange ? "bg-green-600" : "bg-red-600",
          )}
          style={{
            left: `${currentPercent}%`,
            transform: "translateX(-50%)",
          }}
        />
      </div>

      {/* Labels */}
      <div className="flex justify-between text-xs text-gray-400">
        <span>MIN</span>
        <span>MAX</span>
      </div>
    </div>
  );
}
