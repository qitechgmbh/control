import { Icon } from "@/components/Icon";
import { cva } from "class-variance-authority";
import React from "react";

type HopperFigureProps = {
  label: string;
  enabled: boolean;
  error: boolean;
  rpm: number;
};

const hopperBoxClass = cva(
  "flex w-32 flex-col items-center gap-1 rounded-lg border-2 p-3 transition-colors",
  {
    variants: {
      state: {
        error: "border-red-500 bg-red-50",
        enabled: "border-green-500 bg-green-50",
        idle: "border-gray-300 bg-gray-50",
      },
    },
  },
);

const spinClass = cva("transition-transform", {
  variants: {
    spinning: {
      true: "animate-spin",
      false: "",
    },
  },
});

function HopperFigure({ label, enabled, error, rpm }: HopperFigureProps) {
  const state = error ? "error" : enabled ? "enabled" : "idle";

  return (
    <div className={hopperBoxClass({ state })}>
      <span className="text-sm font-semibold">{label}</span>
      <Icon
        name="lu:Package"
        className={spinClass({ spinning: enabled && Math.abs(rpm) > 0.5 })}
      />
      <span className="text-xs text-gray-600">{rpm.toFixed(1)} rpm</span>
    </div>
  );
}

type MixerFigureProps = {
  mixingMotorOn: boolean;
  hopperA: { enabled: boolean; error: boolean; rpm: number };
  hopperB: { enabled: boolean; error: boolean; rpm: number };
};

export function MixerFigure({
  mixingMotorOn,
  hopperA,
  hopperB,
}: MixerFigureProps) {
  return (
    <div className="flex flex-col items-center gap-2 py-4">
      <div className="flex flex-row gap-6">
        <HopperFigure label="Hopper A" {...hopperA} />
        <HopperFigure label="Hopper B" {...hopperB} />
      </div>

      <Icon name="lu:ArrowDown" className="text-gray-400" />

      <div
        className={
          "flex w-40 flex-col items-center gap-1 rounded-full border-2 p-4 transition-colors " +
          (mixingMotorOn
            ? "border-green-500 bg-green-50"
            : "border-gray-300 bg-gray-50")
        }
      >
        <Icon
          name="lu:RotateCw"
          className={spinClass({ spinning: mixingMotorOn })}
        />
        <span className="text-sm font-semibold">Mixing Drum</span>
      </div>
    </div>
  );
}
