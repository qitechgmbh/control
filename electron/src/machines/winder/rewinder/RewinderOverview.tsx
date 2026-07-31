import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { TimeSeries } from "@/lib/timeseries";
import React from "react";
import { Spool } from "../Spool";
import { TensionArm } from "../TensionArm";
import { type Mode, type StateEvent } from "./rewinderNamespace";

type Props = {
  state: StateEvent["data"] | undefined;
  takeupSpoolRpm: TimeSeries;
  sourceSpoolRpm: TimeSeries;
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  takeupTensionArmAngle: TimeSeries;
  sourceTensionArmAngle: TimeSeries;
};

function PullerIcon() {
  return (
    <div className="flex w-full justify-center">
      <div className="h-32 w-12">
        <svg
          viewBox="0 0 96 200"
          width="100%"
          height="100%"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle
            cx="48"
            cy="56"
            r="44"
            className="fill-gray-200 stroke-black"
            strokeWidth="8"
          />
          <circle cx="48" cy="56" r="16" className="fill-black" />
          <circle
            cx="48"
            cy="152"
            r="44"
            className="fill-gray-200 stroke-black"
            strokeWidth="8"
          />
          <circle cx="48" cy="152" r="16" className="fill-black" />
        </svg>
      </div>
    </div>
  );
}

function TraverseGuide() {
  return (
    <div className="flex w-full justify-center">
      <div className="aspect-square h-32">
        <svg
          viewBox="0 0 128 128"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle
            cx="64"
            cy="32"
            r="28"
            className="fill-gray-200 stroke-black"
            strokeWidth="5"
          />
          <circle cx="64" cy="32" r="11" className="fill-black" />
        </svg>
      </div>
    </div>
  );
}

function armWheelBottom(angleDeg: number): number {
  const r = (angleDeg * Math.PI) / 180;
  return 64 + 32.6 * Math.cos(r) + 31;
}

const COL_CENTERS = [50, 150, 250, 350, 450, 550];

function FilamentPath({
  mode,
  canRewind,
  ttaAngle,
  staAngle,
}: {
  mode: Mode | undefined;
  canRewind: boolean;
  ttaAngle: number;
  staAngle: number;
}) {
  if (!mode || mode === "Standby") return null;

  const [xTS, xTR, xTTA, xPUL, xSTA, xSS] = COL_CENTERS;
  const staBot = armWheelBottom(staAngle);
  const ttaBot = armWheelBottom(ttaAngle);
  const pulY = 66;
  const trY = 4;

  const d = [
    `M ${xSS},128`,
    `C ${xSS - 40},128 ${xSTA + 40},${staBot} ${xSTA},${staBot}`,
    `C ${xSTA - 50},${staBot} ${xPUL + 50},${pulY} ${xPUL},${pulY}`,
    `C ${xPUL - 50},${pulY} ${xTTA + 50},${ttaBot} ${xTTA},${ttaBot}`,
    `C ${xTTA - 30},${ttaBot} ${xTR + 30},${trY} ${xTR},${trY}`,
    `C ${xTR - 30},${trY} ${xTS + 30},0 ${xTS},0`,
  ].join(" ");

  let stroke = "#111827";
  let strokeDasharray: string | undefined;
  let className = "";

  if (mode === "Prepare") {
    stroke = canRewind ? "#16a34a" : "#d97706";
  } else if (mode === "Hold") {
    strokeDasharray = "14 8";
  } else if (mode === "Pull") {
    strokeDasharray = "14 8";
    className = "animate-filament";
  }

  return (
    <path
      d={d}
      fill="none"
      stroke={stroke}
      strokeWidth="3"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeDasharray={strokeDasharray}
      className={className}
      vectorEffect="non-scaling-stroke"
      style={{ transition: "stroke 0.4s ease" }}
    />
  );
}

const LABEL = "text-center text-xs text-gray-400";

export function RewinderOverview({
  state,
  takeupSpoolRpm,
  sourceSpoolRpm,
  traversePosition,
  pullerSpeed,
  takeupTensionArmAngle,
  sourceTensionArmAngle,
}: Props) {
  const mode = state?.mode_state.mode;
  const canRewind = state?.mode_state.can_rewind ?? false;
  const ttaAngle = takeupTensionArmAngle.current?.value ?? 0;
  const staAngle = sourceTensionArmAngle.current?.value ?? 0;

  return (
    <div className="flex flex-col gap-4">
      <div className="relative">
        <div className="flex items-start">
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Takeup Spool</p>
            <Spool rpm={takeupSpoolRpm.current?.value} />
          </div>
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Traverse</p>
            <TraverseGuide />
          </div>
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Takeup TA</p>
            <TensionArm degrees={ttaAngle} />
          </div>
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Puller</p>
            <PullerIcon />
          </div>
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Source TA</p>
            <TensionArm degrees={staAngle} />
          </div>
          <div className="flex flex-1 flex-col items-center gap-1">
            <p className={LABEL}>Source Spool</p>
            <Spool rpm={sourceSpoolRpm.current?.value} clockwise />
          </div>
        </div>

        <svg
          viewBox="0 0 600 128"
          preserveAspectRatio="none"
          className="pointer-events-none absolute bottom-0 left-0 h-32 w-full"
        >
          <FilamentPath
            mode={mode}
            canRewind={canRewind}
            ttaAngle={ttaAngle}
            staAngle={staAngle}
          />
        </svg>
      </div>

      <div className="grid grid-cols-6 gap-4">
        <TimeSeriesValueNumeric
          label="Takeup"
          unit="rpm"
          timeseries={takeupSpoolRpm}
          renderValue={(v) => roundToDecimals(v, 0)}
        />
        <TimeSeriesValueNumeric
          label="Position"
          unit="mm"
          timeseries={traversePosition}
          renderValue={(v) => roundToDecimals(v, 1)}
        />
        <TimeSeriesValueNumeric
          label="TU Angle"
          unit="deg"
          timeseries={takeupTensionArmAngle}
          renderValue={(v) => roundDegreesToDecimals(v, 0)}
        />
        <TimeSeriesValueNumeric
          label="Speed"
          unit="m/min"
          timeseries={pullerSpeed}
          renderValue={(v) => roundToDecimals(v, 2)}
        />
        <TimeSeriesValueNumeric
          label="Src Angle"
          unit="deg"
          timeseries={sourceTensionArmAngle}
          renderValue={(v) => roundDegreesToDecimals(v, 0)}
        />
        <TimeSeriesValueNumeric
          label="Source"
          unit="rpm"
          timeseries={sourceSpoolRpm}
          renderValue={(v) => roundToDecimals(v, 0)}
        />
      </div>
    </div>
  );
}
