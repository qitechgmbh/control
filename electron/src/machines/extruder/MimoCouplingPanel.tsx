import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { roundToDecimals } from "@/lib/decimal";
import React, { useState } from "react";
import {
  MAX_CONDITION_NUMBER,
  MAX_RGA_DIAGONAL,
  MIN_RGA_DIAGONAL,
  MIN_SNR_RATIO,
  MimoGains,
  MimoModel,
  MimoState,
  MimoTraceData,
  NEGLIGIBLE_RGA_DEVIATION,
  mimoPhaseLabels,
  mimoRunningPhases,
  mimoZoneLabels,
  modelIsSynthesizable,
} from "./mimoSchema";
import { ResponseSpeed, responseSpeeds } from "./temperatureAutoTuneSchema";

const SPEED_OPTIONS = Object.fromEntries(
  responseSpeeds.map(({ key, label }) => [key, { children: label }]),
) as Record<ResponseSpeed, { children: string }>;

const MODE_OPTIONS = {
  decentralized: { children: "Independent" },
  mimo: { children: "Coupled (MIMO)" },
} as const;

type Props = {
  mimoState?: MimoState;
  trace?: MimoTraceData | null;
  /** Machine must be in Heat mode with the screw stopped for a campaign to start. */
  canStart: boolean;
  onStart: (stepDuty: number, maxRiseCelsius: number) => void;
  onStop: () => void;
  onSynthesize: (method: string, lambdaFactor: number) => void;
  onSetMode: (mode: "decentralized" | "mimo") => void;
};

function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  if (hours > 0) return `${hours}h ${String(minutes).padStart(2, "0")}m`;
  return `${minutes}m ${String(total % 60).padStart(2, "0")}s`;
}

function zoneLabel(zoneOrder: string[], index: number): string {
  const name = zoneOrder[index] ?? String(index);
  return mimoZoneLabels[name] ?? name;
}

/**
 * Coupling identification and matrix-gain control for the four heating zones.
 *
 * The zones conduct heat into each other through the barrel. This panel measures that coupling as
 * a 4x4 transfer matrix, reports whether it is strong enough to be worth compensating, and
 * switches the heaters between four independent PID loops and a single matrix-gain controller.
 */
export function MimoCouplingPanel({
  mimoState,
  trace,
  canStart,
  onStart,
  onStop,
  onSynthesize,
  onSetMode,
}: Props) {
  const [stepDuty, setStepDuty] = useState(10);
  const [maxRise, setMaxRise] = useState(15);
  const [lambdaFactor, setLambdaFactor] = useState<ResponseSpeed>("0.5");

  const phase = mimoState?.phase ?? "idle";
  const isRunning = mimoRunningPhases.includes(phase);
  const model = mimoState?.model ?? null;
  const gains = mimoState?.gains ?? null;
  const mode = mimoState?.mode ?? "decentralized";
  const zoneOrder = mimoState?.zone_order ?? [];

  const startCampaign = () => {
    if (
      !window.confirm(
        "This runs all four heaters open-loop for several hours. None of the zones will " +
          "regulate for the duration, and the barrel will end the run hotter than it started.\n\n" +
          "Only start this on an unloaded machine you can leave alone. Continue?",
      )
    ) {
      return;
    }
    onStart(stepDuty / 100, maxRise);
  };

  const changeMode = (next: string) => {
    if (next !== "decentralized" && next !== "mimo") return;
    if (next === mode) return;
    const message =
      next === "mimo"
        ? "Switch the heating zones onto the coupled MIMO controller?"
        : "Switch the heating zones back to four independent PID loops?";
    if (!window.confirm(message)) return;
    onSetMode(next);
  };

  return (
    <ControlCard title="Zone Coupling & MIMO Control">
      <Label label="Control Structure">
        <SelectionGroup
          value={mode}
          options={MODE_OPTIONS}
          onChange={changeMode}
          disabled={isRunning || !gains}
        />
        <p className="text-muted-foreground mt-2 text-sm">
          {mode === "mimo"
            ? "One controller drives all four heaters, using the measured coupling to hold each zone against its neighbours."
            : gains
              ? "Four independent PID loops. Coupling is rejected as disturbance."
              : "Four independent PID loops. Identify the coupling and synthesize gains to enable the coupled controller."}
        </p>
      </Label>

      <Label label="Step Size">
        <EditValue
          value={stepDuty}
          defaultValue={10}
          unit="%"
          title="Step Size"
          description="How far to step one heater above its holding duty while the other three are frozen. Larger steps resolve the weak, distant couplings better, but every step leaves the barrel hotter than it found it and four of them accumulate."
          min={2}
          max={30}
          step={1}
          renderValue={(v) => roundToDecimals(v, 0)}
          onChange={setStepDuty}
        />
      </Label>

      <Label label="Max Rise per Step">
        <EditValue
          value={maxRise}
          defaultValue={15}
          unit="C"
          title="Maximum Rise per Step"
          description="Abort if any zone moves further than this from the current step's starting point. Measured per step rather than per campaign, because a staircase deliberately leaves each zone hotter. Also capped internally against every zone's over-temperature cutoff."
          min={5}
          max={40}
          step={5}
          renderValue={(v) => roundToDecimals(v, 0)}
          onChange={setMaxRise}
        />
      </Label>

      <Label label="Actions">
        {!canStart && !isRunning && (
          <p className="mb-2 text-sm text-amber-600">
            Requires Heat mode with the screw stopped. Material flow is a large,
            variable heat load that would corrupt every measurement.
          </p>
        )}
        {mode === "mimo" && !isRunning && (
          <p className="mb-2 text-sm text-amber-600">
            Switch back to independent control before re-identifying: the
            campaign has to drive the heaters itself.
          </p>
        )}
        <div className="flex gap-4">
          <button
            onClick={startCampaign}
            disabled={!canStart || isRunning || mode === "mimo"}
            className="inline-block w-fit rounded bg-blue-600 px-4 py-4 text-base text-white hover:bg-blue-700 disabled:opacity-50"
          >
            Measure Coupling
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
            {mimoPhaseLabels[phase] ?? phase}
            {mimoState?.column != null
              ? ` — stepping ${zoneLabel(zoneOrder, mimoState.column)}`
              : ""}
          </span>
          <div className="h-3 w-full rounded bg-slate-200">
            <div
              className="h-3 rounded bg-blue-500 transition-all"
              style={{ width: `${mimoState?.progress_percent ?? 0}%` }}
            />
          </div>
          <span className="text-muted-foreground text-sm">
            {roundToDecimals(mimoState?.progress_percent ?? 0, 0)}% ·{" "}
            {formatDuration(mimoState?.elapsed_seconds ?? 0)} elapsed ·{" "}
            {mimoState?.columns_done ?? 0} of {zoneOrder.length || 4} zones
            measured
          </span>
          {isRunning && (
            <span className="text-muted-foreground text-sm">
              A full campaign takes two to three hours. All four zones are
              open-loop for the duration — none of them is holding setpoint.
            </span>
          )}
        </div>
      </Label>

      {phase === "failed" && mimoState?.failure_reason && (
        <StatusBadge variant="error">
          Campaign aborted: {mimoState.failure_reason}. All four zones are back
          under normal PID control.
        </StatusBadge>
      )}

      {trace && trace.samples.length > 1 && (
        <Label label="Live Trace">
          <TraceChart trace={trace} />
        </Label>
      )}

      {model && (
        <ModelSection
          model={model}
          gains={gains}
          lambdaFactor={lambdaFactor}
          setLambdaFactor={setLambdaFactor}
          onSynthesize={onSynthesize}
          disabled={isRunning}
          synthesisError={mimoState?.synthesis_error ?? null}
        />
      )}
    </ControlCard>
  );
}

/** Per-zone colours, consistent between the trace and the matrix headers. */
const ZONE_COLORS = ["#2563eb", "#16a34a", "#d97706", "#dc2626"];

/**
 * All four temperatures and all four duties over the campaign.
 *
 * The point is to make coupling visible as it happens: when one zone is stepped, its neighbours
 * should visibly follow. A campaign where the other three traces stay flat has either measured a
 * genuinely decoupled barrel or gone wrong, and either way that is worth seeing hours before the
 * numbers arrive.
 */
function TraceChart({ trace }: { trace: MimoTraceData }) {
  const width = 720;
  const height = 240;
  const pad = { top: 12, right: 12, bottom: 26, left: 44 };

  const samples = trace.samples;
  const zoneCount = trace.zone_order.length || 4;

  const tMin = samples[0]?.t_seconds ?? 0;
  const tMax = samples[samples.length - 1]?.t_seconds ?? 1;
  const temps = samples.flatMap((s) => s.temperatures);
  const yMin = Math.min(...temps);
  const yMax = Math.max(...temps);
  const ySpan = Math.max(yMax - yMin, 1);

  const x = (t: number) =>
    pad.left +
    ((t - tMin) / Math.max(tMax - tMin, 1e-9)) * (width - pad.left - pad.right);
  const y = (v: number) =>
    height -
    pad.bottom -
    ((v - yMin) / ySpan) * (height - pad.top - pad.bottom);

  const path = (zone: number) =>
    samples
      .map(
        (s, i) =>
          `${i === 0 ? "M" : "L"}${x(s.t_seconds).toFixed(1)},${y(
            s.temperatures[zone] ?? 0,
          ).toFixed(1)}`,
      )
      .join(" ");

  return (
    <div className="w-full overflow-x-auto">
      <svg width={width} height={height} className="max-w-full">
        <rect
          x={pad.left}
          y={pad.top}
          width={width - pad.left - pad.right}
          height={height - pad.top - pad.bottom}
          fill="none"
          stroke="#cbd5e1"
        />
        {[yMin, (yMin + yMax) / 2, yMax].map((v) => (
          <text
            key={v}
            x={pad.left - 6}
            y={y(v) + 4}
            textAnchor="end"
            fontSize="11"
            fill="#64748b"
          >
            {roundToDecimals(v, 0)}
          </text>
        ))}
        <text
          x={width / 2}
          y={height - 6}
          textAnchor="middle"
          fontSize="11"
          fill="#64748b"
        >
          {formatDuration(tMax - tMin)} elapsed
        </text>
        {Array.from({ length: zoneCount }, (_, z) => (
          <path
            key={z}
            d={path(z)}
            fill="none"
            stroke={ZONE_COLORS[z % ZONE_COLORS.length]}
            strokeWidth="1.5"
          />
        ))}
      </svg>
      <div className="mt-1 flex flex-wrap gap-4">
        {trace.zone_order.map((name, z) => (
          <span key={name} className="flex items-center gap-1 text-sm">
            <span
              className="inline-block h-2 w-4 rounded"
              style={{ background: ZONE_COLORS[z % ZONE_COLORS.length] }}
            />
            {mimoZoneLabels[name] ?? name}
          </span>
        ))}
      </div>
    </div>
  );
}

function ModelSection({
  model,
  gains,
  lambdaFactor,
  setLambdaFactor,
  onSynthesize,
  disabled,
  synthesisError,
}: {
  model: MimoModel;
  gains: MimoGains | null;
  lambdaFactor: ResponseSpeed;
  setLambdaFactor: (v: ResponseSpeed) => void;
  onSynthesize: (method: string, lambdaFactor: number) => void;
  disabled: boolean;
  synthesisError: string | null;
}) {
  const zoneOrder = model.zone_order;
  const synthesizable = modelIsSynthesizable(model);
  const worthIt = model.max_rga_deviation >= NEGLIGIBLE_RGA_DEVIATION;

  return (
    <>
      <Label label="Coupling Matrix">
        <p className="text-muted-foreground mb-2 text-sm">
          Steady-state gain in °C per unit of duty: row = zone that responds,
          column = heater that was driven. Zones are in physical order along the
          barrel, so a healthy measurement is strongest on the diagonal and
          fades outwards.
        </p>
        <MatrixGrid
          zoneOrder={zoneOrder}
          value={(i, j) => model.g[i][j].gp}
          format={(v) => roundToDecimals(v, 1)}
          shade={(i, j) => {
            const own = model.g[j][j].gp;
            return own > 0 ? Math.min(Math.abs(model.g[i][j].gp / own), 1) : 0;
          }}
          title={(i, j) =>
            `τ ${roundToDecimals(model.g[i][j].tau, 0)}s · dead time ${roundToDecimals(
              model.g[i][j].theta,
              0,
            )}s · SNR ${roundToDecimals(model.g[i][j].snr_ratio, 1)}`
          }
          lowConfidence={(i, j) =>
            i !== j && model.g[i][j].snr_ratio < MIN_SNR_RATIO
          }
        />
      </Label>

      <Label label="Interaction (RGA)">
        <p className="text-muted-foreground mb-2 text-sm">
          How much each pairing's gain changes once the other loops are closed.
          A diagonal of 1 means the zones do not interact.
        </p>
        <MatrixGrid
          zoneOrder={zoneOrder}
          value={(i, j) => model.rga[i][j]}
          format={(v) => roundToDecimals(v, 2)}
          shade={(i, j) => Math.min(Math.abs(model.rga[i][j]), 1)}
        />
      </Label>

      <Label label="Diagnostics">
        <div className="flex flex-wrap gap-2">
          <StatusBadge
            variant={
              model.condition_number > MAX_CONDITION_NUMBER
                ? "error"
                : "success"
            }
          >
            Condition number {roundToDecimals(model.condition_number, 1)}
          </StatusBadge>
          <StatusBadge variant={model.niederlinski < 0 ? "error" : "success"}>
            Niederlinski {roundToDecimals(model.niederlinski, 2)}
          </StatusBadge>
          <StatusBadge variant={worthIt ? "success" : "error"}>
            Peak interaction {roundToDecimals(model.max_rga_deviation * 100, 0)}
            %
          </StatusBadge>
          <StatusBadge variant="success">
            Strongest cross-coupling{" "}
            {roundToDecimals(model.max_coupling_ratio * 100, 0)}%
          </StatusBadge>
        </div>

        {!worthIt && (
          <p className="mt-2 text-sm text-amber-600">
            These zones are already close to independent. MIMO control will not
            change much here — the coupling it exists to remove is small.
          </p>
        )}
        {model.rga.some(
          (row, i) => row[i] < MIN_RGA_DIAGONAL || row[i] > MAX_RGA_DIAGONAL,
        ) && (
          <p className="mt-2 text-sm text-red-600">
            At least one zone is dominated by its neighbours rather than its own
            heater. Pairing them is the wrong structure regardless of gains.
          </p>
        )}
        <p className="text-muted-foreground mt-2 text-sm">
          Measured at{" "}
          {model.setpoints.map((s) => roundToDecimals(s, 0)).join(" / ")} °C.
          The barrel is not linear across its whole range, so these numbers hold
          near that operating point — re-measure per recipe, not once per
          machine.
        </p>
      </Label>

      <Label label="Response Speed">
        <SelectionGroup
          value={lambdaFactor}
          options={SPEED_OPTIONS}
          onChange={setLambdaFactor}
          disabled={disabled}
        />
      </Label>

      <Label label="Synthesize Gains">
        <button
          onClick={() => onSynthesize("decoupler", Number(lambdaFactor))}
          disabled={disabled || !synthesizable}
          className="inline-block w-fit rounded bg-blue-600 px-4 py-4 text-base text-white hover:bg-blue-700 disabled:opacity-50"
        >
          Synthesize
        </button>
        {!synthesizable && (
          <p className="mt-2 text-sm text-red-600">
            This model is not usable for decoupling — see the diagnostics above.
          </p>
        )}
        {synthesisError && (
          <StatusBadge variant="error">{synthesisError}</StatusBadge>
        )}
      </Label>

      {gains && <GainsSection gains={gains} zoneOrder={zoneOrder} />}
    </>
  );
}

function GainsSection({
  gains,
  zoneOrder,
}: {
  gains: MimoGains;
  zoneOrder: string[];
}) {
  return (
    <Label label="Synthesized Gains">
      <p className="text-muted-foreground mb-2 text-sm">
        Integral gains are decoupled — row = heater, column = the zone whose
        error drives it. The off-diagonal terms are what hold a zone steady when
        its neighbour changes. Proportional and derivative gains stay diagonal
        on purpose: the coupling paths carry minutes of dead time, and applying
        a steady-state correction to the fast path would fire it long before the
        heat it cancels has arrived.
      </p>
      <MatrixGrid
        zoneOrder={zoneOrder}
        value={(i, j) => gains.ki[i][j]}
        format={(v) => v.toExponential(2)}
        shade={(i, j) => {
          const own = Math.abs(gains.ki[j][j]);
          return own > 0 ? Math.min(Math.abs(gains.ki[i][j] / own), 1) : 0;
        }}
      />
      <p className="text-muted-foreground mt-2 text-sm">
        Proportional:{" "}
        {zoneOrder
          .map(
            (name, i) =>
              `${mimoZoneLabels[name] ?? name} ${roundToDecimals(gains.kp[i][i], 5)}`,
          )
          .join(" · ")}
      </p>
      <p className="text-muted-foreground text-sm">
        Method: {gains.method}
        {gains.derivative_filter_tc > 0 &&
          ` · derivative filter ${roundToDecimals(gains.derivative_filter_tc, 1)}s`}
      </p>
    </Label>
  );
}

/**
 * A 4x4 matrix with zone headers and a magnitude shade.
 *
 * The shading is the point: banded structure is visible instantly, so an operator can tell a
 * plausible measurement from a broken one without reading sixteen numbers.
 */
function MatrixGrid({
  zoneOrder,
  value,
  format,
  shade,
  title,
  lowConfidence,
}: {
  zoneOrder: string[];
  value: (i: number, j: number) => number;
  format: (v: number) => string | number;
  shade: (i: number, j: number) => number;
  title?: (i: number, j: number) => string;
  lowConfidence?: (i: number, j: number) => boolean;
}) {
  const n = zoneOrder.length;
  return (
    <div className="overflow-x-auto">
      <table className="border-collapse text-sm">
        <thead>
          <tr>
            <th className="px-2 py-1" />
            {zoneOrder.map((name) => (
              <th
                key={name}
                className="text-muted-foreground px-2 py-1 text-center font-medium"
              >
                {mimoZoneLabels[name] ?? name}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: n }, (_, i) => (
            <tr key={i}>
              <th className="text-muted-foreground px-2 py-1 text-right font-medium">
                {mimoZoneLabels[zoneOrder[i]] ?? zoneOrder[i]}
              </th>
              {Array.from({ length: n }, (_, j) => {
                const v = value(i, j);
                const alpha = Math.max(0, Math.min(shade(i, j), 1));
                return (
                  <td
                    key={j}
                    title={title?.(i, j)}
                    className="border border-slate-200 px-3 py-2 text-right tabular-nums"
                    style={{
                      background:
                        v < 0
                          ? `rgba(220, 38, 38, ${alpha * 0.35})`
                          : `rgba(37, 99, 235, ${alpha * 0.35})`,
                      fontWeight: i === j ? 600 : 400,
                      opacity: lowConfidence?.(i, j) ? 0.45 : 1,
                    }}
                  >
                    {format(v)}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
