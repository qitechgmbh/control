import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { StatusBadge } from "@/control/StatusBadge";
import { AlertTriangle, Check } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import "./mixing-machine-preview.css";
import { ConnectedExtruderCard, ExtruderLinkState } from "./MixerExtruderLink";

export type MachinePhase =
  | "idle"
  | "starting"
  | "running"
  | "purging"
  | "fault";

const MATERIAL_A = {
  name: "Material A",
  color: "#2563eb",
};

const MATERIAL_B = {
  name: "Material B",
  color: "#d97706",
};

const recipes = [
  { name: "70 / 30 blend", ratioA: 70, feedRate: 12 },
  { name: "50 / 50 blend", ratioA: 50, feedRate: 10 },
  { name: "30 / 70 blend", ratioA: 30, feedRate: 9 },
];

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function DosingChannel({
  x,
  y,
  mirrored = false,
  running,
}: {
  x: number;
  y: number;
  mirrored?: boolean;
  running: boolean;
}) {
  const augerClipId = mirrored ? "mm-auger-clip-b" : "mm-auger-clip-a";

  return (
    <g transform={`translate(${x} ${y}) ${mirrored ? "scale(-1 1)" : ""}`}>
      <defs>
        <clipPath id={augerClipId}>
          <rect x="205" y="13" width="102" height="34" />
        </clipPath>
      </defs>
      <rect
        x="0"
        y="5"
        width="58"
        height="50"
        rx="6"
        className="mm-feeder-motor"
      />
      <rect
        x="54"
        y="9"
        width="34"
        height="42"
        rx="4"
        className="mm-feeder-coupling"
      />
      <path d="M84 8 H315 V56 H84 Z" className="mm-dosing-channel" />
      <path d="M205 13 H307 V47 H205 Z" className="mm-dosing-window" />
      <g clipPath={`url(#${augerClipId})`}>
        <line x1="207" y1="30" x2="306" y2="30" className="mm-auger-shaft" />
        <g className={`mm-auger-flights ${running ? "is-running" : ""}`}>
          {[198, 210, 222, 234, 246, 258, 270, 282, 294].map((flightX) => (
            <path key={flightX} d={`M${flightX} 13 L${flightX + 12} 47`} />
          ))}
        </g>
      </g>
    </g>
  );
}

function Hopper({
  side,
  material,
  ratio,
  rpm,
  fill,
  contentColor,
  contentOpacity,
  running,
  empty,
  low,
}: {
  side: "A" | "B";
  material: typeof MATERIAL_A;
  ratio: number;
  rpm: number;
  fill: number;
  contentColor: string;
  contentOpacity: number;
  running: boolean;
  empty: boolean;
  low: boolean;
}) {
  const left = side === "A";
  const clipId = `mm-hopper-${side}`;
  const centerX = left ? 215 : 785;
  const hopperPath = left
    ? "M55 78 H270 L338 168 L310 250 H268 L55 174 Z"
    : "M945 78 H730 L662 168 L690 250 H732 L945 174 Z";
  const channelX = left ? 105 : 895;
  const materialLevelY = 250 - fill * 1.55;

  return (
    <g>
      <text x={centerX} y="28" textAnchor="middle" className="mm-svg-label">
        HOPPER {side}
      </text>
      <circle
        cx={centerX - 52}
        cy="51"
        r="7"
        fill={material.color}
        stroke="white"
        strokeWidth="3"
      />
      <text x={centerX} y="56" textAnchor="middle" className="mm-svg-material">
        {material.name}
      </text>
      <defs>
        <clipPath id={clipId}>
          <path d={hopperPath} />
        </clipPath>
      </defs>
      <path d={hopperPath} className="mm-hopper-shell" />
      <g clipPath={`url(#${clipId})`}>
        <rect
          x={left ? 50 : 657}
          y={materialLevelY}
          width="293"
          height={fill * 1.55}
          fill={contentColor}
          fillOpacity={contentOpacity}
        />
        <line
          x1={left ? 50 : 657}
          x2={left ? 343 : 950}
          y1={materialLevelY}
          y2={materialLevelY}
          className="mm-material-level-line"
        />
      </g>
      <path d={hopperPath} className="mm-hopper-outline" />
      <rect
        x={left ? 45 : 720}
        y="69"
        width="235"
        height="22"
        rx="5"
        className="mm-hopper-lid"
      />
      <DosingChannel x={channelX} y={247} mirrored={!left} running={running} />

      <rect
        x={left ? 266 : 690}
        y="246"
        width="44"
        height="52"
        rx="5"
        className="mm-feeder-throat"
      />

      <rect
        x={left ? 105 : 715}
        y="320"
        width="180"
        height="39"
        rx="9"
        className={
          empty
            ? "mm-motor-status is-fault"
            : low
              ? "mm-motor-status is-warning"
              : "mm-motor-status"
        }
      />
      <circle
        cx={left ? 121 : 731}
        cy="339"
        r="5"
        className={
          running
            ? "mm-status-dot is-running"
            : low
              ? "mm-status-dot is-warning"
              : "mm-status-dot"
        }
      />
      <text
        x={left ? 135 : 745}
        y="343"
        className={
          empty ? "mm-svg-fault" : low ? "mm-svg-warning" : "mm-svg-status"
        }
      >
        {empty
          ? "Material empty"
          : low
            ? "Material low"
            : running
              ? "Dosing"
              : "Stopped"}
      </text>
      <text
        x={left ? 268 : 878}
        y="343"
        textAnchor="end"
        className="mm-svg-rpm"
      >
        {running ? rpm : 0} rpm
      </text>
      <text
        x={left ? 195 : 805}
        y="392"
        textAnchor="middle"
        className="mm-svg-ratio"
      >
        {ratio}%
      </text>
    </g>
  );
}

export function MachineOverview({
  phase,
  ratioA,
  feedRate,
  hopperAEmpty,
  hopperBEmpty,
  hopperALow = false,
  hopperBLow = false,
  materialColorA = MATERIAL_A.color,
  materialColorB = MATERIAL_B.color,
  showMaterialColors = false,
}: {
  phase: MachinePhase;
  ratioA: number;
  feedRate: number;
  hopperAEmpty: boolean;
  hopperBEmpty: boolean;
  hopperALow?: boolean;
  hopperBLow?: boolean;
  materialColorA?: string;
  materialColorB?: string;
  showMaterialColors?: boolean;
}) {
  const feeding = phase === "running";
  const mixerRunning =
    phase === "starting" || phase === "running" || phase === "purging";
  const rpmA = Math.round((feedRate * ratioA) / 22);
  const rpmB = Math.round((feedRate * (100 - ratioA)) / 22);
  return (
    <div className="mm-overview">
      <svg
        viewBox="0 0 1000 600"
        role="img"
        aria-label="Two material hoppers feeding a common mixer and extruder"
      >
        <Hopper
          side="A"
          material={{ ...MATERIAL_A, color: materialColorA }}
          contentColor={materialColorA}
          contentOpacity={showMaterialColors ? 0.55 : 0.18}
          ratio={ratioA}
          rpm={rpmA}
          fill={hopperAEmpty ? 0 : hopperALow ? 20 : 76}
          running={feeding && !hopperAEmpty}
          empty={hopperAEmpty}
          low={hopperALow}
        />
        <Hopper
          side="B"
          material={{ ...MATERIAL_B, color: materialColorB }}
          contentColor={materialColorB}
          contentOpacity={showMaterialColors ? 0.55 : 0.18}
          ratio={100 - ratioA}
          rpm={rpmB}
          fill={hopperBEmpty ? 0 : hopperBLow ? 20 : 76}
          running={feeding && !hopperBEmpty}
          empty={hopperBEmpty}
          low={hopperBLow}
        />

        <g transform="translate(420 250)">
          <path
            d="M18 0 H142 V18 H160 V225 H138 V255 H22 V225 H0 V18 H18 Z"
            className="mm-mixer-shell"
          />
          <rect
            x="18"
            y="0"
            width="124"
            height="28"
            rx="4"
            className="mm-mixer-lid"
          />
          <text x="80" y="18" textAnchor="middle" className="mm-mixer-label">
            CENTRE MIXER
          </text>
          <circle cx="80" cy="150" r="50" className="mm-mixer-port" />
          <rect
            x="20"
            y="135"
            width="20"
            height="30"
            rx="4"
            className="mm-port-clamp"
          />
          <rect
            x="120"
            y="135"
            width="20"
            height="30"
            rx="4"
            className="mm-port-clamp"
          />
          <g transform="translate(80 150)">
            <g
              className={`mm-mixer-shaft ${mixerRunning && phase !== "fault" ? "is-running" : ""}`}
            >
              <circle r="35" />
              <circle r="8" />
              <path d="M0 -26 V26 M-26 0 H26 M-18 -18 L18 18 M18 -18 L-18 18" />
            </g>
          </g>
          <rect
            x="5"
            y="255"
            width="150"
            height="16"
            rx="2"
            className="mm-machine-foot"
          />
        </g>

        <path d="M500 530 V550" className="mm-output-arrow" />
        <path d="M490 540 L500 551 L510 540" className="mm-output-arrow" />
        <text x="500" y="580" textAnchor="middle" className="mm-svg-label">
          TO EXTRUDER
        </text>
      </svg>
    </div>
  );
}

export function MixingMachinePreview() {
  const [phase, setPhase] = useState<MachinePhase>("idle");
  const [ratioA, setRatioA] = useState(70);
  const [feedRate, setFeedRate] = useState(12);
  const [recipe, setRecipe] = useState("70 / 30 blend");
  const [hopperAEmpty, setHopperAEmpty] = useState(false);
  const [hopperBEmpty, setHopperBEmpty] = useState(false);
  const [hopperALow, setHopperALow] = useState(false);
  const [hopperBLow, setHopperBLow] = useState(false);
  const [mixerFault, setMixerFault] = useState(false);
  const [extruderLinkState, setExtruderLinkState] =
    useState<ExtruderLinkState>("ready");
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const ratioB = 100 - ratioA;
  const running = phase === "running";
  const busy = phase === "starting" || phase === "purging";
  const canStart =
    phase === "idle" &&
    !hopperAEmpty &&
    !hopperBEmpty &&
    !mixerFault &&
    extruderLinkState === "ready";

  useEffect(
    () => () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    },
    [],
  );

  useEffect(() => {
    if (mixerFault && phase !== "idle") {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      setPhase("fault");
    } else if (!mixerFault && phase === "fault") {
      setPhase("idle");
    }
  }, [mixerFault, phase]);

  useEffect(() => {
    if ((hopperAEmpty || hopperBEmpty) && phase === "running") {
      setPhase("purging");
      timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
    }
  }, [hopperAEmpty, hopperBEmpty, phase]);

  useEffect(() => {
    if (
      extruderLinkState !== "ready" &&
      (phase === "running" || phase === "starting")
    ) {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      setPhase("purging");
      timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
    }
  }, [extruderLinkState, phase]);

  const start = () => {
    if (!canStart) return;
    setPhase("starting");
    timeoutRef.current = setTimeout(() => setPhase("running"), 900);
  };

  const stop = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    if (phase === "starting") {
      setPhase("idle");
      return;
    }
    setPhase("purging");
    timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
  };

  const reset = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setPhase("idle");
    setHopperAEmpty(false);
    setHopperBEmpty(false);
    setHopperALow(false);
    setHopperBLow(false);
    setMixerFault(false);
    setExtruderLinkState("ready");
  };

  const applyRecipe = (name: string) => {
    const selected = recipes.find((item) => item.name === name);
    if (!selected) return;
    setRecipe(selected.name);
    setRatioA(selected.ratioA);
    setFeedRate(selected.feedRate);
  };

  return (
    <Page className="mm-page">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold">Material Mixer</h1>
        </div>
        <div className="flex flex-wrap gap-2">
          {phase === "running" && (
            <StatusBadge variant="success">Mixing</StatusBadge>
          )}
          {phase === "starting" && (
            <StatusBadge variant="success">Starting mixer</StatusBadge>
          )}
          {phase === "purging" && (
            <StatusBadge variant="success">Clearing mixer</StatusBadge>
          )}
          {phase === "fault" && (
            <StatusBadge variant="error">Not Ready</StatusBadge>
          )}
          {phase === "idle" && canStart && (
            <StatusBadge variant="success">Ready</StatusBadge>
          )}
          {phase === "idle" && !canStart && (
            <StatusBadge variant="error">Not Ready</StatusBadge>
          )}
          {mixerFault && <StatusBadge variant="error">Mixer fault</StatusBadge>}
          {hopperAEmpty && (
            <StatusBadge variant="error">Hopper A empty</StatusBadge>
          )}
          {hopperBEmpty && (
            <StatusBadge variant="error">Hopper B empty</StatusBadge>
          )}
          {hopperALow && !hopperAEmpty && (
            <StatusBadge variant="warning">Hopper A low</StatusBadge>
          )}
          {hopperBLow && !hopperBEmpty && (
            <StatusBadge variant="warning">Hopper B low</StatusBadge>
          )}
          {extruderLinkState === "no-demand" && (
            <StatusBadge variant="warning">No extruder demand</StatusBadge>
          )}
          {extruderLinkState === "fault" && (
            <StatusBadge variant="error">Extruder fault</StatusBadge>
          )}
        </div>
      </div>

      <ControlCard title="Overview">
        <MachineOverview
          phase={phase}
          ratioA={ratioA}
          feedRate={feedRate}
          hopperAEmpty={hopperAEmpty}
          hopperBEmpty={hopperBEmpty}
          hopperALow={hopperALow}
          hopperBLow={hopperBLow}
        />
      </ControlCard>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2 2xl:grid-cols-4">
        <ControlCard title="Blend">
          <Label label="Recipe">
            <select
              className="h-12 w-full rounded-md border border-gray-200 bg-white px-3 text-sm disabled:bg-gray-100"
              value={recipe}
              disabled={running || busy}
              onChange={(event) => applyRecipe(event.target.value)}
            >
              {recipe === "Custom" && (
                <option value="Custom">Custom blend</option>
              )}
              {recipes.map((item) => (
                <option key={item.name}>{item.name}</option>
              ))}
            </select>
          </Label>

          <Label label="Material Ratio">
            <div className="mb-2 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span
                  className="size-3 rounded-sm"
                  style={{ backgroundColor: MATERIAL_A.color }}
                />
                <strong>{ratioA}%</strong>
                <span className="text-xs text-gray-500">Hopper A</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500">Hopper B</span>
                <strong>{ratioB}%</strong>
                <span
                  className="size-3 rounded-sm border border-gray-300"
                  style={{ backgroundColor: MATERIAL_B.color }}
                />
              </div>
            </div>
            <TouchSlider
              min={0}
              max={100}
              step={1}
              value={[ratioA]}
              disabled={running || busy}
              minLabel="0% A"
              maxLabel="100% A"
              onValueChange={([value]) => {
                setRatioA(value);
                setRecipe("Custom");
              }}
            />
          </Label>
          <p className="text-xs text-amber-700">
            Displayed ratio is estimated until both feeders are calibrated.
          </p>
        </ControlCard>

        <ControlCard title="Production">
          <Label label="Total Feed Rate">
            <div className="flex items-center gap-3">
              <TouchButton
                variant="outline"
                disabled={running || busy}
                onClick={() =>
                  setFeedRate((value) => clamp(value - 0.5, 1, 25))
                }
              >
                −
              </TouchButton>
              <div className="flex-1 rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 text-center">
                <span className="font-mono text-3xl font-semibold">
                  {feedRate.toFixed(feedRate % 1 === 0 ? 0 : 1)}
                </span>
                <span className="ml-2 text-sm text-gray-500">kg/h</span>
              </div>
              <TouchButton
                variant="outline"
                disabled={running || busy}
                onClick={() =>
                  setFeedRate((value) => clamp(value + 0.5, 1, 25))
                }
              >
                +
              </TouchButton>
            </div>
          </Label>

          <div className="rounded-xl border border-gray-200 bg-gray-50 p-4">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-500">Hopper A</span>
                <strong className="mt-1 block">
                  {((feedRate * ratioA) / 100).toFixed(1)} kg/h
                </strong>
              </div>
              <div>
                <span className="text-gray-500">Hopper B</span>
                <strong className="mt-1 block">
                  {((feedRate * ratioB) / 100).toFixed(1)} kg/h
                </strong>
              </div>
            </div>
          </div>

          {running || busy ? (
            <TouchButton
              variant="destructive"
              icon="lu:OctagonX"
              onClick={stop}
            >
              {phase === "purging" ? "Clearing Mixer…" : "Stop Mixing"}
            </TouchButton>
          ) : (
            <TouchButton
              icon="lu:Play"
              disabled={!canStart}
              onClick={start}
              className="bg-green-600 text-white"
            >
              Start Mixing
            </TouchButton>
          )}
        </ControlCard>

        <ControlCard title="Errors">
          {[
            {
              label: "Hopper A low",
              errorState: "LOW",
              active: hopperALow,
              toggle: () => {
                setHopperALow((value) => !value);
                setHopperAEmpty(false);
              },
            },
            {
              label: "Hopper A empty",
              errorState: "EMPTY",
              active: hopperAEmpty,
              toggle: () => {
                setHopperAEmpty((value) => !value);
                setHopperALow(false);
              },
            },
            {
              label: "Hopper B low",
              errorState: "LOW",
              active: hopperBLow,
              toggle: () => {
                setHopperBLow((value) => !value);
                setHopperBEmpty(false);
              },
            },
            {
              label: "Hopper B empty",
              errorState: "EMPTY",
              active: hopperBEmpty,
              toggle: () => {
                setHopperBEmpty((value) => !value);
                setHopperBLow(false);
              },
            },
            {
              label: "Mixer fault",
              errorState: "FAULT",
              active: mixerFault,
              toggle: () => setMixerFault((value) => !value),
            },
          ].map((signal) => (
            <button
              key={signal.label}
              className={`flex min-h-12 w-full items-center justify-between rounded-lg border px-3 text-sm ${
                signal.active
                  ? "border-red-300 bg-red-50 text-red-700"
                  : "border-gray-200 bg-gray-50"
              }`}
              onClick={signal.toggle}
            >
              <span className="flex items-center gap-2">
                {signal.active ? (
                  <AlertTriangle className="size-4" />
                ) : (
                  <Check className="size-4 text-green-600" />
                )}
                {signal.label}
              </span>
              <span className="text-xs font-semibold">
                {signal.active ? signal.errorState : "OK"}
              </span>
            </button>
          ))}

          <TouchButton variant="outline" icon="lu:RotateCcw" onClick={reset}>
            Reset Simulation
          </TouchButton>
        </ControlCard>

        <ConnectedExtruderCard
          state={extruderLinkState}
          onStateChange={setExtruderLinkState}
        />
      </div>
    </Page>
  );
}
