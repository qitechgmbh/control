import { cn } from "@/utils/tailwind";
import React from "react";

type Props = {
  inside: number;
  outside: number;
  min: number;
  max: number;
  current: number;
  className?: string;
};

export function TraverseBar({
  inside,
  outside,
  min,
  max,
  current,
  className,
}: Props) {
  // Calculate percentages for positioning
  const range = outside - inside;
  const minStopPercent = ((min - inside) / range) * 100;
  const maxStopPercent = ((max - inside) / range) * 100;
  const currentPercent = 100 - ((current - inside) / range) * 100;

  return (
    <div className={cn("w-full space-y-1 pt-6", className)}>
      <div className="relative h-4 w-full">
        {/* Base bar with overflow hidden to clip inner bar */}
        <div className="absolute h-full w-full overflow-hidden rounded-full bg-gray-200">
          {/* Stop range (black section) - no rounded corners */}
          <div
            className="absolute h-full bg-black"
            style={{
              right: `${minStopPercent}%`,
              width: `${maxStopPercent - minStopPercent}%`,
            }}
          />
        </div>

        {/* Current position indicator - outside the overflow container so it's not clipped */}
        <div
          className="absolute -top-5 z-10 h-14 w-1.5 rounded-full bg-black"
          style={{
            left: `${currentPercent}%`,
            transform: "translateX(-50%)",
          }}
        />
      </div>

      {/* Labels */}
      <div className="flex justify-between text-xs text-gray-400">
        <span>OUT</span>
        <span>IN</span>
      </div>
    </div>
  );
}
