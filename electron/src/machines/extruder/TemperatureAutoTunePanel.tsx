import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { roundToDecimals } from "@/lib/decimal";
import React, { useState } from "react";
import {
  fittedCurve,
  ImcGains,
  TemperatureAutoTuneResult,
  TemperatureAutoTuneState,
  TemperatureAutoTuneTraceData,
  ResponseSpeed,
  responseSpeeds,
  tunePhaseLabels,
  TuneZone,
} from "./temperatureAutoTuneSchema";

const ZONE_OPTIONS: Record<TuneZone, { children: string }> = {
  front: { children: "Front" },
  middle: { children: "Middle" },
  back: { children: "Back" },
  nozzle: { children: "Nozzle" },
};

const SPEED_OPTIONS = Object.fromEntries(
  responseSpeeds.map(({ key, label }) => [key, { children: label }]),
) as Record<ResponseSpeed, { children: string }>;

type Props = {
  tuneState?: TemperatureAutoTuneState;
  trace?: TemperatureAutoTuneTraceData | null;
  /** Machine must be in Heat mode with the screw stopped for a run to start. */
  canStart: boolean;
  onStart: (
    zone: TuneZone,
    stepDuty: number,
    maxRiseCelsius: number,
    lambdaFactor: number,
  ) => void;
  onStop: () => void;
  onApply: (form: "pi" | "pid") => void;
};

const RUNNING_PHASES = ["waiting_for_steady", "baseline_hold", "step"];

function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const minutes = Math.floor(total / 60);
  return `${minutes}m ${String(total % 60).padStart(2, "0")}s`;
}

/**
 * IMC (Internal Model Control) auto-tuner for the heating zones.
 *
 * Runs an open-loop step test on one zone at a time, fits a first-order-plus-dead-time model to
 * the response, and derives PI and PID gains from it. The other three zones keep regulating
 * throughout, so the identified model reflects the machine as it actually runs in production.
 */
export function TemperatureAutoTunePanel({
  tuneState,
  trace,
  canStart,
  onStart,
  onStop,
  onApply,
}: Props) {
  const [zone, setZone] = useState<TuneZone>("middle");
  const [stepDuty, setStepDuty] = useState(10);
  const [maxRise, setMaxRise] = useState(30);
  const [lambdaFactor, setLambdaFactor] = useState<ResponseSpeed>("0.3");

  const phase = tuneState?.phase ?? "idle";
  const isRunning = RUNNING_PHASES.includes(phase);
  const result = tuneState?.result ?? null;
  const activeZone = tuneState?.zone ?? null;

  return (
    <ControlCard title="Temperature PID Auto-Tune (IMC)">
      <Label label="Zone">
        <SelectionGroup
          value={zone}
          options={ZONE_OPTIONS}
          onChange={setZone}
          disabled={isRunning}
        />
      </Label>

      <Label label="Step Size">
        <EditValue
          value={stepDuty}
          defaultValue={10}
          unit="%"
          title="Step Size"
          description="How far to step the heater duty cycle above its current level. Aim for a temperature rise of 8-20 °C: too small and the fit is buried in noise, too large and the zone leaves its linear region."
          min={-50}
          max={50}
          step={1}
          renderValue={(v) => roundToDecimals(v, 0)}
          onChange={setStepDuty}
        />
      </Label>

      <Label label="Max Rise">
        <EditValue
          value={maxRise}
          defaultValue={30}
          unit="C"
          title="Maximum Temperature Rise"
          description="Abort the run if the zone moves further than this from its baseline. Also capped internally so the run stops well short of the over-temperature cutoff."
          min={5}
          max={60}
          step={5}
          renderValue={(v) => roundToDecimals(v, 0)}
          onChange={setMaxRise}
        />
      </Label>

      <Label label="Response Speed">
        <SelectionGroup
          value={lambdaFactor}
          options={SPEED_OPTIONS}
          onChange={setLambdaFactor}
          disabled={isRunning}
        />
      </Label>

      <Label label="Actions">
        {!canStart && !isRunning && (
          <p className="mb-2 text-sm text-amber-600">
            Requires Heat mode with the screw stopped. Material flow is a large,
            variable heat load that would corrupt the step response.
          </p>
        )}
        <div className="flex gap-4">
          <button
            onClick={() =>
              onStart(zone, stepDuty / 100, maxRise, Number(lambdaFactor))
            }
            disabled={!canStart || isRunning}
            className="inline-block w-fit rounded bg-blue-600 px-4 py-4 text-base text-white hover:bg-blue-700 disabled:opacity-50"
          >
            Start Auto-Tune
          </button>
          <button
            onClick={onStop}
            disabled={!isRunning}
            className="inline-block w-fit rounded bg-red-600 px-4 py-4 text-base text-white hover:bg-red-700 disabled:opacity-50"
          >
            Stop
          </button>
        </div>
      </Label>

      <Label label="Status">
        <div className="flex flex-col gap-2">
          <span className="text-base">
            {tunePhaseLabels[phase] ?? phase}
            {activeZone ? ` — ${activeZone}` : ""}
          </span>
          <div className="h-3 w-full rounded bg-slate-200">
            <div
              className="h-3 rounded bg-blue-500 transition-all"
              style={{ width: `${tuneState?.progress ?? 0}%` }}
            />
          </div>
          <span className="text-muted-foreground text-sm">
            {roundToDecimals(tuneState?.progress ?? 0, 0)}% ·{" "}
            {formatDuration(tuneState?.elapsed_seconds ?? 0)} elapsed
            {isRunning &&
              ` · holding ${roundToDecimals(
                (tuneState?.current_duty ?? 0) * 100,
                1,
              )}% duty`}
          </span>
          {isRunning && (
            <span className="text-muted-foreground text-sm">
              A full run takes roughly 15-30 minutes. The other zones keep
              regulating, and their duty will drop as heat bleeds across.
            </span>
          )}
        </div>
      </Label>

      {phase === "failed" && tuneState?.failure_reason && (
        <StatusBadge variant="error">
          Run aborted: {tuneState.failure_reason}. The zone is back under normal
          PID control.
        </StatusBadge>
      )}

      {trace && trace.samples.length > 1 && (
        <Label label="Recorded Curve">
          <TraceChart
            trace={trace}
            result={result}
            baselineTemperature={tuneState?.baseline_temperature ?? 0}
          />
        </Label>
      )}

      {result && <ResultSection result={result} onApply={onApply} />}
    </ControlCard>
  );
}

/**
 * The recorded response with the fitted model overlaid. A visible gap between the two is the
 * fastest way to spot a run that should not be trusted — faster than reading any single number.
 */
function TraceChart({
  trace,
  result,
  baselineTemperature,
}: {
  trace: TemperatureAutoTuneTraceData;
  result: TemperatureAutoTuneResult | null;
  baselineTemperature: number;
}) {
  const width = 640;
  const height = 200;
  const pad = { top: 8, right: 8, bottom: 20, left: 40 };

  const samples = trace.samples;
  const xs = samples.map((s) => s.t_seconds);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);

  // The step begins where the commanded duty first departs from its baseline.
  const baselineDuty = samples[0]?.duty ?? 0;
  const stepIndex = samples.findIndex(
    (s) => Math.abs(s.duty - baselineDuty) > 1e-6,
  );
  const stepStartSeconds = stepIndex >= 0 ? samples[stepIndex].t_seconds : maxX;

  const fitted = result
    ? fittedCurve(result, baselineTemperature, samples, stepStartSeconds)
    : [];

  const allTemps = [
    ...samples.map((s) => s.temperature),
    ...fitted.map((p) => p.temperature),
  ];
  const minY = Math.min(...allTemps);
  const maxY = Math.max(...allTemps);
  const spanY = maxY - minY || 1;
  const spanX = maxX - minX || 1;

  const sx = (t: number) =>
    pad.left + ((t - minX) / spanX) * (width - pad.left - pad.right);
  const sy = (v: number) =>
    pad.top + (1 - (v - minY) / spanY) * (height - pad.top - pad.bottom);

  const path = (points: { t_seconds: number; temperature: number }[]) =>
    points
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"}${sx(p.t_seconds).toFixed(1)},${sy(p.temperature).toFixed(1)}`,
      )
      .join(" ");

  return (
    <div className="w-full overflow-x-auto">
      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="h-auto w-full min-w-[480px]"
        role="img"
        aria-label="Recorded step response with fitted model"
      >
        <line
          x1={pad.left}
          y1={height - pad.bottom}
          x2={width - pad.right}
          y2={height - pad.bottom}
          className="stroke-slate-300"
          strokeWidth={1}
        />
        {stepIndex >= 0 && (
          <line
            x1={sx(stepStartSeconds)}
            y1={pad.top}
            x2={sx(stepStartSeconds)}
            y2={height - pad.bottom}
            className="stroke-slate-400"
            strokeWidth={1}
            strokeDasharray="3 3"
          />
        )}
        {fitted.length > 1 && (
          <path
            d={path(fitted)}
            fill="none"
            className="stroke-amber-500"
            strokeWidth={2}
            strokeDasharray="5 3"
          />
        )}
        <path
          d={path(
            samples.map((s) => ({
              t_seconds: s.t_seconds,
              temperature: s.temperature,
            })),
          )}
          fill="none"
          className="stroke-blue-600"
          strokeWidth={1.5}
        />
        <text x={4} y={sy(maxY) + 4} className="fill-slate-500 text-[10px]">
          {roundToDecimals(maxY, 1)}
        </text>
        <text x={4} y={sy(minY) + 4} className="fill-slate-500 text-[10px]">
          {roundToDecimals(minY, 1)}
        </text>
      </svg>
      <div className="text-muted-foreground flex gap-4 text-xs">
        <span className="text-blue-600">— measured</span>
        <span className="text-amber-600">-- fitted model</span>
        <span>{formatDuration(spanX)} shown</span>
      </div>
    </div>
  );
}

function ResultSection({
  result,
  onApply,
}: {
  result: TemperatureAutoTuneResult;
  onApply: (form: "pi" | "pid") => void;
}) {
  return (
    <>
      <Label label="Identified Model">
        <div className="grid grid-cols-2 gap-x-6 gap-y-1 text-sm md:grid-cols-3">
          <Metric
            name="Process gain"
            value={`${roundToDecimals(result.process_gain, 1)} °C / duty`}
          />
          <Metric
            name="Time constant τ"
            value={`${roundToDecimals(result.time_constant, 1)} s`}
            note={`63% method: ${roundToDecimals(result.tau_63, 1)} s`}
          />
          <Metric
            name="Dead time θ"
            value={`${roundToDecimals(result.dead_time, 1)} s`}
            note={`±1 °C crossing: ${roundToDecimals(result.dead_time_threshold, 1)} s`}
          />
          <Metric
            name="Temperature rise"
            value={`${roundToDecimals(result.delta_pv, 2)} °C`}
            note={`for ${roundToDecimals(result.delta_u * 100, 1)}% duty`}
          />
          <Metric
            name="Signal / noise"
            value={`${roundToDecimals(result.snr_ratio, 1)}×`}
            note={`noise ${roundToDecimals(result.noise_peak_to_peak, 2)} °C p-p`}
          />
          <Metric
            name="Fit error"
            value={`${roundToDecimals(result.fit_error_pct, 2)}%`}
            note={`rms ${roundToDecimals(result.rms_residual, 3)} °C`}
          />
        </div>
      </Label>

      {!result.is_confident && (
        <StatusBadge variant="error">
          The step barely cleared the noise (
          {roundToDecimals(result.snr_ratio, 1)}×, want 5× or better). These
          gains are not trustworthy — re-run with a step of about{" "}
          {roundToDecimals(result.suggested_step_duty * 100, 0)}%.
        </StatusBadge>
      )}

      {!result.is_good_fit && (
        <StatusBadge variant="error">
          The response does not match a first-order model well (fit error{" "}
          {roundToDecimals(result.fit_error_pct, 1)}%). Check the curve above —
          a neighbouring zone may have moved during the run.
        </StatusBadge>
      )}

      <Label label="Tuning Candidates">
        <div className="overflow-x-auto">
          <table className="w-full min-w-[520px] text-sm">
            <thead className="text-muted-foreground text-left text-xs">
              <tr>
                <th className="py-1 pr-4">Form</th>
                <th className="py-1 pr-4">Kc</th>
                <th className="py-1 pr-4">Ti</th>
                <th className="py-1 pr-4">Td</th>
                <th className="py-1 pr-4">kp</th>
                <th className="py-1 pr-4">ki</th>
                <th className="py-1 pr-4">kd</th>
                <th className="py-1" />
              </tr>
            </thead>
            <tbody>
              <GainRow
                label="IMC-PI"
                gains={result.pi}
                onApply={() => onApply("pi")}
              />
              <GainRow
                label="IMC-PID"
                gains={result.pid}
                onApply={() => onApply("pid")}
              />
            </tbody>
          </table>
        </div>
        <p className="text-muted-foreground mt-2 text-xs">
          PI is the usual choice: the IMC rules give the two forms the same
          response speed, so derivative action mostly adds noise sensitivity.
          PID is worth trying on a dead-time-dominant zone such as the nozzle.
          Applying PID also sets a matching derivative filter.
        </p>
      </Label>
    </>
  );
}

function GainRow({
  label,
  gains,
  onApply,
}: {
  label: string;
  gains: ImcGains;
  onApply: () => void;
}) {
  return (
    <tr className="border-t border-slate-200">
      <td className="py-2 pr-4 font-medium">{label}</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.kc, 5)}</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.ti, 1)} s</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.td, 1)} s</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.kp, 5)}</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.ki, 7)}</td>
      <td className="py-2 pr-4">{roundToDecimals(gains.kd, 5)}</td>
      <td className="py-2">
        <button
          onClick={onApply}
          className="rounded bg-blue-600 px-3 py-2 text-sm text-white hover:bg-blue-700"
        >
          Apply
        </button>
      </td>
    </tr>
  );
}

function Metric({
  name,
  value,
  note,
}: {
  name: string;
  value: string;
  note?: string;
}) {
  return (
    <div>
      <div className="text-muted-foreground text-xs">{name}</div>
      <div className="font-medium">{value}</div>
      {note && <div className="text-muted-foreground text-xs">{note}</div>}
    </div>
  );
}
