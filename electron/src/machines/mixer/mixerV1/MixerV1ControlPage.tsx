import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { StatusBadge } from "@/control/StatusBadge";
import React from "react";
import "./mixing-machine-preview.css";
import { useMixerV1 } from "./useMixerV1";

const HOPPER_MAX_RPM = 100;

const MATERIAL_A = {
  name: "Material A",
  color: "#2563eb",
};

const MATERIAL_B = {
  name: "Material B",
  color: "#d97706",
};

function DosingChannel({
  x,
  y,
  mirrored = false,
  running,
  forward,
}: {
  x: number;
  y: number;
  mirrored?: boolean;
  running: boolean;
  forward: boolean;
}) {
  const augerClipId = mirrored ? "mm-auger-clip-b" : "mm-auger-clip-a";
  const reverseAnimation = forward;

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
        <g
          className={`mm-auger-flights ${running ? "is-running" : ""} ${reverseAnimation ? "is-reversed" : ""}`}
        >
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
  label,
  material,
  rpm,
  fill,
  contentColor,
  contentOpacity,
  running,
  forward,
}: {
  side: "A" | "B";
  label: string;
  material: typeof MATERIAL_A;
  rpm: number | null;
  fill: number;
  contentColor: string;
  contentOpacity: number;
  running: boolean;
  forward: boolean;
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
        {label}
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
      <DosingChannel
        x={channelX}
        y={247}
        mirrored={!left}
        running={running}
        forward={forward}
      />

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
        className="mm-motor-status"
      />
      <circle
        cx={left ? 121 : 731}
        cy="339"
        r="5"
        className={running ? "mm-status-dot is-running" : "mm-status-dot"}
      />
      <text x={left ? 135 : 745} y="343" className="mm-svg-status">
        {running ? "Dosing" : "Stopped"}
      </text>
      <text
        x={left ? 268 : 878}
        y="343"
        textAnchor="end"
        className="mm-svg-rpm"
      >
        {running ? (rpm === null ? "—" : rpm.toFixed(1)) : "0"} rpm
      </text>
    </g>
  );
}

function CentralMaterialHopper() {
  const hopperPath = "M390 10 H610 V142 L540 235 H460 L390 142 Z";

  return (
    <g>
      <defs>
        <clipPath id="mm-central-hopper-clip">
          <path d={hopperPath} />
        </clipPath>
      </defs>
      <path d={hopperPath} className="mm-hopper-shell" />
      <g clipPath="url(#mm-central-hopper-clip)">
        <rect
          x="386"
          y="112"
          width="228"
          height="128"
          fill="#64748b"
          fillOpacity="0.16"
        />
        <line
          x1="386"
          x2="614"
          y1="112"
          y2="112"
          className="mm-material-level-line"
        />
      </g>
      <path d={hopperPath} className="mm-hopper-outline" />
      <rect
        x="382"
        y="1"
        width="236"
        height="20"
        rx="5"
        className="mm-hopper-lid"
      />
      <text x="500" y="42" textAnchor="middle" className="mm-svg-label">
        CENTRAL HOPPER
      </text>
      <text x="500" y="65" textAnchor="middle" className="mm-svg-material">
        Main material
      </text>
      <rect
        x="460"
        y="234"
        width="80"
        height="28"
        rx="5"
        className="mm-central-hopper-throat"
      />
    </g>
  );
}

function MachineOverview({
  mixingMotorOn,
  hopperAEnabled,
  hopperAForward,
  hopperARpm,
  hopperBEnabled,
  hopperBForward,
  hopperBRpm,
}: {
  mixingMotorOn: boolean;
  hopperAEnabled: boolean;
  hopperAForward: boolean;
  hopperARpm: number | null;
  hopperBEnabled: boolean;
  hopperBForward: boolean;
  hopperBRpm: number | null;
}) {
  return (
    <div className="mm-overview">
      <svg
        viewBox="0 0 1000 600"
        role="img"
        aria-label="Two dosing dispensers and a central material hopper feeding a common mixer"
      >
        <CentralMaterialHopper />
        <Hopper
          side="A"
          label="DISPENSER 2"
          material={MATERIAL_A}
          contentColor={MATERIAL_A.color}
          contentOpacity={0.18}
          rpm={hopperARpm}
          fill={76}
          running={hopperAEnabled}
          forward={hopperAForward}
        />
        <Hopper
          side="B"
          label="DISPENSER 1"
          material={MATERIAL_B}
          contentColor={MATERIAL_B.color}
          contentOpacity={0.18}
          rpm={hopperBRpm}
          fill={76}
          running={hopperBEnabled}
          forward={hopperBForward}
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
            <g className={`mm-mixer-shaft ${mixingMotorOn ? "is-running" : ""}`}>
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
          OUTPUT
        </text>
      </svg>
    </div>
  );
}

export function MixerV1ControlPage() {
  const {
    state,
    hopperARpm,
    hopperBRpm,
    setMixingMotorOn,
    setHopperAEnabled,
    setHopperAForward,
    setHopperATargetRpm,
    setHopperBEnabled,
    setHopperBForward,
    setHopperBTargetRpm,
  } = useMixerV1();

  const mixingMotorOn = state?.mixing_motor_state.on ?? false;
  const hopperAEnabled = state?.hopper_a_state.enabled ?? false;
  const hopperAError = state?.hopper_a_state.error ?? false;
  const hopperAForward = state?.hopper_a_state.forward ?? true;
  const hopperATargetRpm = state?.hopper_a_state.target_rpm ?? 0;
  const hopperBEnabled = state?.hopper_b_state.enabled ?? false;
  const hopperBError = state?.hopper_b_state.error ?? false;
  const hopperBForward = state?.hopper_b_state.forward ?? true;
  const hopperBTargetRpm = state?.hopper_b_state.target_rpm ?? 0;

  const hasError = hopperAError || hopperBError;

  return (
    <Page className="mm-page">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold">Mixer</h1>
        </div>
        <div className="flex flex-wrap gap-2">
          {hasError && <StatusBadge variant="error">Error</StatusBadge>}
          {!hasError && mixingMotorOn && (
            <StatusBadge variant="success">Mixing</StatusBadge>
          )}
          {!hasError && !mixingMotorOn && (
            <StatusBadge variant="success">Ready</StatusBadge>
          )}
        </div>
      </div>

      <ControlCard title="Overview">
        <MachineOverview
          mixingMotorOn={mixingMotorOn}
          hopperAEnabled={hopperAEnabled}
          hopperAForward={hopperAForward}
          hopperARpm={hopperARpm.current?.value ?? null}
          hopperBEnabled={hopperBEnabled}
          hopperBForward={hopperBForward}
          hopperBRpm={hopperBRpm.current?.value ?? null}
        />
      </ControlCard>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-3">
        <ControlCard title="Dispenser 2">
          <Label label="Target Speed">
            <div className="mb-3 flex items-center justify-between">
              <span className="flex items-center gap-2 text-sm text-gray-500">
                <span
                  className="size-3 rounded-sm"
                  style={{ backgroundColor: MATERIAL_A.color }}
                />
                {MATERIAL_A.name}
              </span>
              <strong className="font-mono text-2xl">
                {hopperATargetRpm.toFixed(1)} rpm
              </strong>
            </div>
            <TouchSlider
              min={0}
              max={HOPPER_MAX_RPM}
              step={0.1}
              value={[hopperATargetRpm]}
              minLabel="0 rpm"
              maxLabel={`${HOPPER_MAX_RPM} rpm`}
              onValueChange={([value]) => setHopperATargetRpm(value)}
            />
          </Label>
          <TouchButton
            icon={hopperAEnabled ? "lu:Pause" : "lu:Play"}
            onClick={() => setHopperAEnabled(!hopperAEnabled)}
          >
            {hopperAEnabled ? "Disable" : "Enable"}
          </TouchButton>
        </ControlCard>

        <ControlCard title="Dispenser 1">
          <Label label="Target Speed">
            <div className="mb-3 flex items-center justify-between">
              <span className="flex items-center gap-2 text-sm text-gray-500">
                <span
                  className="size-3 rounded-sm"
                  style={{ backgroundColor: MATERIAL_B.color }}
                />
                {MATERIAL_B.name}
              </span>
              <strong className="font-mono text-2xl">
                {hopperBTargetRpm.toFixed(1)} rpm
              </strong>
            </div>
            <TouchSlider
              min={0}
              max={HOPPER_MAX_RPM}
              step={0.1}
              value={[hopperBTargetRpm]}
              minLabel="0 rpm"
              maxLabel={`${HOPPER_MAX_RPM} rpm`}
              onValueChange={([value]) => setHopperBTargetRpm(value)}
            />
          </Label>
          <TouchButton
            icon={hopperBEnabled ? "lu:Pause" : "lu:Play"}
            onClick={() => setHopperBEnabled(!hopperBEnabled)}
          >
            {hopperBEnabled ? "Disable" : "Enable"}
          </TouchButton>
        </ControlCard>

        <ControlCard title="Operation">
          <StatusBadge variant={hasError ? "error" : "success"}>
            {hasError ? "Error" : "Ready"}
          </StatusBadge>
          <p className="text-sm text-gray-600">
            The mixer starts and stops with this button. Motor direction is
            configured in Settings.
          </p>
          {mixingMotorOn ? (
            <TouchButton
              variant="destructive"
              icon="lu:OctagonX"
              onClick={() => setMixingMotorOn(false)}
            >
              Stop Mixer
            </TouchButton>
          ) : (
            <TouchButton
              icon="lu:Play"
              onClick={() => setMixingMotorOn(true)}
              className="bg-green-600 text-white"
            >
              Start Mixer
            </TouchButton>
          )}
        </ControlCard>
      </div>
    </Page>
  );
}
