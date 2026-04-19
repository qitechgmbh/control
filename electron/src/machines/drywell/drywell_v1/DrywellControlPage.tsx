import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { drywellV1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { drywellV1SerialRoute } from "@/routes/routes";
import { useDrywellNamespace } from "./drywellNamespace";
import { useMachineMutate } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { z } from "zod";

const TEMP_COLORS = [
  "#22d3ee",
  "#38bdf8",
  "#4ade80",
  "#facc15",
  "#f97316",
  "#f43f5e",
];
const TEMP_LABELS = ["T1", "T2", "T3", "T4", "T5", "T6"];
const HISTORY_LEN = 60;

type TempPoint = [number, number, number, number, number, number];

function isRunningStatus(status: number): boolean {
  return status !== 1 && status !== 5 && status !== 6;
}

function isCoolingStatus(status: number): boolean {
  return status === 5 || status === 6;
}

function statusLabel(status: number): string {
  const labels: Record<number, string> = {
    0: "Starting",
    1: "Standby",
    2: "Warming up",
    3: "Motor starting",
    4: "Running",
    5: "Cooling",
    6: "Fan stopping",
  };
  return labels[status] ?? `Status ${status}`;
}

function CircularGauge({
  value,
  label,
}: {
  value: number | undefined;
  label: string;
}) {
  const pct = value ?? 0;
  const r = 36;
  const cx = 50;
  const cy = 50;
  const circumference = 2 * Math.PI * r;
  const dash = (pct / 100) * circumference;
  const gap = circumference - dash;

  return (
    <div className="flex flex-col items-center gap-1">
      <div className="relative h-24 w-24">
        <svg viewBox="0 0 100 100" className="h-full w-full -rotate-90">
          <circle
            cx={cx}
            cy={cy}
            r={r}
            fill="none"
            stroke="#e5e7eb"
            strokeWidth="10"
          />
          <circle
            cx={cx}
            cy={cy}
            r={r}
            fill="none"
            stroke="#22c55e"
            strokeWidth="10"
            strokeDasharray={`${dash} ${gap}`}
            strokeLinecap="round"
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-lg leading-none font-bold">
            {pct.toFixed(0)}
          </span>
          <span className="text-xs text-gray-400">%</span>
        </div>
      </div>
      <span className="text-sm font-semibold text-gray-600">{label}</span>
    </div>
  );
}

function PowerBar({
  label,
  value,
}: {
  label: string;
  value: number | undefined;
}) {
  const pct = value ?? 0;
  return (
    <div className="flex flex-col gap-1">
      <div className="flex justify-between text-sm text-gray-500">
        <span className="font-semibold">{label}</span>
        <span>Power %</span>
      </div>
      <div className="h-4 overflow-hidden rounded-full bg-gray-100">
        <div
          className="h-full rounded-full bg-gray-800 transition-all duration-500"
          style={{ width: `${Math.min(pct, 100)}%` }}
        />
      </div>
      <div className="text-right text-xl font-bold">{pct.toFixed(0)} %</div>
    </div>
  );
}

function TempChart({ history }: { history: TempPoint[] }) {
  const width = 280;
  const height = 140;
  const pad = { top: 8, right: 8, bottom: 8, left: 8 };

  const allVals = history.flatMap((p) => p as number[]);
  const minV = allVals.length ? Math.min(...allVals) - 5 : 20;
  const maxV = allVals.length ? Math.max(...allVals) + 5 : 200;

  const xScale = (i: number) =>
    pad.left +
    (i / Math.max(history.length - 1, 1)) * (width - pad.left - pad.right);
  const yScale = (v: number) =>
    pad.top +
    (1 - (v - minV) / (maxV - minV)) * (height - pad.top - pad.bottom);

  const paths = [0, 1, 2, 3, 4, 5].map((si) => {
    if (history.length < 2) return null;
    const d = history
      .map(
        (pt, i) =>
          `${i === 0 ? "M" : "L"}${xScale(i).toFixed(1)},${yScale(pt[si]).toFixed(1)}`,
      )
      .join(" ");
    return (
      <path
        key={si}
        d={d}
        fill="none"
        stroke={TEMP_COLORS[si]}
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
    );
  });

  return (
    <div className="flex flex-col gap-2">
      <svg width={width} height={height} className="w-full">
        {paths}
      </svg>
      <div className="flex flex-wrap gap-x-3 gap-y-1">
        {TEMP_LABELS.map((lbl, i) => (
          <div key={lbl} className="flex items-center gap-1 text-xs">
            <div
              className="h-1.5 w-3 rounded-full"
              style={{ background: TEMP_COLORS[i] }}
            />
            <span className="text-gray-500">{lbl}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function DrywellControlPage() {
  const { serial: serialString } = drywellV1SerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: drywellV1.machine_identification,
      serial: Number(serialString),
    }),
    [serialString],
  );

  const { liveValues } = useDrywellNamespace(machineIdentification);
  const v = liveValues?.data;

  const historyRef = useRef<TempPoint[]>([]);
  const [history, setHistory] = useState<TempPoint[]>([]);

  useEffect(() => {
    if (v === undefined) return;
    const pt: TempPoint = [
      v.temp_regen_in,
      v.temp_regen_out,
      v.temp_fan_inlet,
      v.temp_process,
      v.temp_safety,
      v.temp_return_air,
    ];
    const next = [...historyRef.current, pt].slice(-HISTORY_LEN);
    historyRef.current = next;
    setHistory(next);
  }, [v]);

  const isCooling = v !== undefined && isCoolingStatus(v.status);

  const {
    value: displayRunning,
    setOptimistic: setRunningOptimistic,
    setReal: setRunningReal,
  } = useStateOptimistic<boolean>();
  const {
    value: displayTargetTemp,
    setOptimistic: setTargetTempOptimistic,
    setReal: setTargetTempReal,
  } = useStateOptimistic<number>();

  useEffect(() => {
    if (v?.status !== undefined) setRunningReal(isRunningStatus(v.status));
  }, [v?.status, setRunningReal]);

  useEffect(() => {
    if (v?.target_temperature !== undefined)
      setTargetTempReal(v.target_temperature);
  }, [v?.target_temperature, setTargetTempReal]);

  const { request: sendMutation, isLoading: mutating } = useMachineMutate(
    z.any(),
  );

  const handleStartStop = () => {
    const next = !displayRunning;
    setRunningOptimistic(next);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetStartStop: next },
    });
  };

  const handleTargetTemp = (temp: number) => {
    const rounded = Math.round(Math.max(50, Math.min(180, temp)));
    setTargetTempOptimistic(rounded);
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: { SetTargetTemperature: rounded },
    });
  };

  const buttonLabel = isCooling
    ? "Cooling..."
    : displayRunning
      ? "Stop"
      : "Start";
  const buttonIcon = isCooling
    ? "lu:Wind"
    : displayRunning
      ? "lu:Square"
      : "lu:Play";
  const buttonVariant = displayRunning && !isCooling ? "default" : "outline";

  const temps: { id: string; value: number | undefined }[] = [
    { id: "T1", value: v?.temp_regen_in },
    { id: "T2", value: v?.temp_regen_out },
    { id: "T3", value: v?.temp_fan_inlet },
    { id: "T4", value: v?.temp_process },
    { id: "T5", value: v?.temp_safety },
    { id: "T6", value: v?.temp_return_air },
  ];

  return (
    <Page>
      <ControlGrid columns={3}>
        {/* Temperatures */}
        <ControlCard title="Temperature Sensors">
          <div className="grid grid-cols-2 gap-3">
            {temps.map(({ id, value }) => (
              <div key={id} className="flex items-baseline gap-1">
                <span className="w-6 text-sm font-semibold text-gray-400">
                  {id}:
                </span>
                <span className="text-2xl font-bold">
                  {value !== undefined ? `${value.toFixed(1)}°C` : "—"}
                </span>
              </div>
            ))}
          </div>
          <div className="flex items-baseline gap-1 border-t border-gray-100 pt-1">
            <span className="w-16 text-sm font-semibold text-gray-400">
              Dew Point:
            </span>
            <span className="text-xl font-bold">
              {v?.temp_dew_point !== undefined
                ? `${v.temp_dew_point.toFixed(1)}°C`
                : "—"}
            </span>
          </div>
        </ControlCard>

        {/* Heaters */}
        <ControlCard title="Heating Power">
          <PowerBar label="R1 Process" value={v?.power_process} />
          <PowerBar label="R2 Regen" value={v?.power_regen} />
        </ControlCard>

        {/* Overview chart */}
        <ControlCard title="Dryer Overview">
          <TempChart history={history} />
        </ControlCard>

        {/* Control */}
        <ControlCard title="Control">
          <Label label="Start / Stop">
            <TouchButton
              icon={buttonIcon}
              variant={buttonVariant}
              onClick={handleStartStop}
              isLoading={mutating}
              disabled={v === undefined || isCooling}
            >
              {buttonLabel}
            </TouchButton>
          </Label>
          <Label label="Target Temperature">
            <EditValue
              title="Target Temperature"
              value={displayTargetTemp}
              min={50}
              max={180}
              step={1}
              disabled={v === undefined}
              renderValue={(val) => val.toFixed(0)}
              unit="C"
              onChange={handleTargetTemp}
            />
          </Label>
          <Label label="Status">
            <span className="font-mono text-base text-gray-700">
              {v !== undefined ? statusLabel(v.status) : "—"}
            </span>
          </Label>
          {v?.alarm ? (
            <div className="rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-sm font-semibold text-red-700">
              Alarm {v.alarm}
            </div>
          ) : null}
          {v?.warning ? (
            <div className="rounded-xl border border-yellow-200 bg-yellow-50 px-3 py-2 text-sm font-semibold text-yellow-700">
              Warning {v.warning}
            </div>
          ) : null}
        </ControlCard>

        {/* Fans */}
        <ControlCard title="Fan Speed">
          <div className="flex h-full items-center justify-around">
            <CircularGauge value={v?.pwm_fan1} label="S1" />
            <CircularGauge value={v?.pwm_fan2} label="S2" />
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
