import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { StatusBadge } from "@/control/StatusBadge";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import "./mixing-machine-preview.css";
import { useMixerV1 } from "./useMixerV1";

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
  material,
  ratio,
  rpm,
  fill,
  contentColor,
  contentOpacity,
  running,
  forward,
}: {
  side: "A" | "B";
  material: typeof MATERIAL_A;
  ratio: number;
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
      <text
        x={left ? 195 : 805}
        y="392"
        textAnchor="middle"
        className="mm-svg-ratio"
      >
        {ratio.toFixed(1)}%
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
  hopperADosingPercent,
  hopperARpm,
  hopperBEnabled,
  hopperBForward,
  hopperBDosingPercent,
  hopperBRpm,
}: {
  mixingMotorOn: boolean;
  hopperAEnabled: boolean;
  hopperAForward: boolean;
  hopperADosingPercent: number;
  hopperARpm: number | null;
  hopperBEnabled: boolean;
  hopperBForward: boolean;
  hopperBDosingPercent: number;
  hopperBRpm: number | null;
}) {
  return (
    <div className="mm-overview">
      <svg
        viewBox="0 0 1000 600"
        role="img"
        aria-label="Two dosing hoppers and a central material hopper feeding a common mixer and extruder"
      >
        <CentralMaterialHopper />
        <Hopper
          side="A"
          material={MATERIAL_A}
          contentColor={MATERIAL_A.color}
          contentOpacity={0.18}
          ratio={hopperADosingPercent}
          rpm={hopperARpm}
          fill={76}
          running={hopperAEnabled}
          forward={hopperAForward}
        />
        <Hopper
          side="B"
          material={MATERIAL_B}
          contentColor={MATERIAL_B.color}
          contentOpacity={0.18}
          ratio={hopperBDosingPercent}
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
          TO EXTRUDER
        </text>
      </svg>
    </div>
  );
}

export function MixerV1ControlPage() {
  const {
    state,
    defaultState,
    hopperARpm,
    hopperBRpm,
    setMixingMotorOn,
    setHopperAEnabled,
    setHopperAForward,
    setHopperADosingPercent,
    setHopperBEnabled,
    setHopperBForward,
    setHopperBDosingPercent,
    setExtruderKgPerRpm,
  } = useMixerV1();

  const mixingMotorOn = state?.mixing_motor_state.on ?? false;
  const hopperAEnabled = state?.hopper_a_state.enabled ?? false;
  const hopperAError = state?.hopper_a_state.error ?? false;
  const hopperAForward = state?.hopper_a_state.forward ?? true;
  const hopperADosingPercent = state?.hopper_a_state.dosing_percent ?? 0;
  const hopperACalibrated =
    (state?.hopper_a_state.calibration_steps_per_kgh ?? 0) > 0;
  const hopperBEnabled = state?.hopper_b_state.enabled ?? false;
  const hopperBError = state?.hopper_b_state.error ?? false;
  const hopperBForward = state?.hopper_b_state.forward ?? true;
  const hopperBDosingPercent = state?.hopper_b_state.dosing_percent ?? 0;
  const hopperBCalibrated =
    (state?.hopper_b_state.calibration_steps_per_kgh ?? 0) > 0;

  const hasError = hopperAError || hopperBError;
  const automaticReady =
    (hopperADosingPercent === 0 || hopperACalibrated) &&
    (hopperBDosingPercent === 0 || hopperBCalibrated);

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
          {!hasError && !mixingMotorOn && automaticReady && (
            <StatusBadge variant="success">Ready</StatusBadge>
          )}
          {!hasError && !mixingMotorOn && !automaticReady && (
            <StatusBadge variant="error">Not Ready</StatusBadge>
          )}
        </div>
      </div>

      <ControlCard title="Overview">
        <MachineOverview
          mixingMotorOn={mixingMotorOn}
          hopperAEnabled={hopperAEnabled}
          hopperAForward={hopperAForward}
          hopperADosingPercent={hopperADosingPercent}
          hopperARpm={hopperARpm.current?.value ?? null}
          hopperBEnabled={hopperBEnabled}
          hopperBForward={hopperBForward}
          hopperBDosingPercent={hopperBDosingPercent}
          hopperBRpm={hopperBRpm.current?.value ?? null}
        />
      </ControlCard>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-2 2xl:grid-cols-4">
        <ControlCard title="Left Doser">
          <Label label="Addition relative to main material">
            <div className="mb-3 flex items-center justify-between">
              <span className="flex items-center gap-2 text-sm text-gray-500">
                <span
                  className="size-3 rounded-sm"
                  style={{ backgroundColor: MATERIAL_A.color }}
                />
                Left material
              </span>
              <strong className="font-mono text-2xl">
                {hopperADosingPercent.toFixed(1)}%
              </strong>
            </div>
            <TouchSlider
              min={0}
              max={100}
              step={0.1}
              value={[hopperADosingPercent]}
              minLabel="0%"
              maxLabel="100% of main"
              onValueChange={([value]) => setHopperADosingPercent(value)}
            />
          </Label>
          <p className="text-xs text-amber-700">
            {hopperACalibrated
              ? `Calibration: ${state?.hopper_a_state.calibration_steps_per_kgh.toFixed(2)} steps/kgh`
              : "Calibration required for automatic dosing."}
          </p>
          <TouchButton
            icon={hopperAEnabled ? "lu:Pause" : "lu:Play"}
            onClick={() => setHopperAEnabled(!hopperAEnabled)}
          >
            {hopperAEnabled ? "Disable" : "Enable"}
          </TouchButton>
        </ControlCard>

        <ControlCard title="Extruder Link">
          <StatusBadge variant="success">Follows extruder</StatusBadge>
          <Label label="Extruder kg per RPM">
            <EditValue
              value={state?.extruder_kg_per_rpm}
              defaultValue={defaultState?.extruder_kg_per_rpm}
              title="Extruder kg per RPM"
              min={0}
              step={0.01}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setExtruderKgPerRpm}
            />
          </Label>
          <div className="grid grid-cols-2 gap-2 text-center text-sm">
            <div>
              <span className="text-gray-500">Left / main</span>
              <strong className="block">
                {hopperADosingPercent.toFixed(1)}%
              </strong>
            </div>
            <div>
              <span className="text-gray-500">Right / main</span>
              <strong className="block">
                {hopperBDosingPercent.toFixed(1)}%
              </strong>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Right Doser">
          <Label label="Addition relative to main material">
            <div className="mb-3 flex items-center justify-between">
              <span className="flex items-center gap-2 text-sm text-gray-500">
                <span
                  className="size-3 rounded-sm"
                  style={{ backgroundColor: MATERIAL_B.color }}
                />
                Right material
              </span>
              <strong className="font-mono text-2xl">
                {hopperBDosingPercent.toFixed(1)}%
              </strong>
            </div>
            <TouchSlider
              min={0}
              max={100}
              step={0.1}
              value={[hopperBDosingPercent]}
              minLabel="0%"
              maxLabel="100% of main"
              onValueChange={([value]) => setHopperBDosingPercent(value)}
            />
          </Label>
          <p className="text-xs text-amber-700">
            {hopperBCalibrated
              ? `Calibration: ${state?.hopper_b_state.calibration_steps_per_kgh.toFixed(2)} steps/kgh`
              : "Calibration required for automatic dosing."}
          </p>
          <TouchButton
            icon={hopperBEnabled ? "lu:Pause" : "lu:Play"}
            onClick={() => setHopperBEnabled(!hopperBEnabled)}
          >
            {hopperBEnabled ? "Disable" : "Enable"}
          </TouchButton>
        </ControlCard>

        <ControlCard title="Operation">
          <StatusBadge variant={automaticReady ? "success" : "error"}>
            {automaticReady ? "Ready" : "Calibration needed"}
          </StatusBadge>
          <p className="text-sm text-gray-600">
            The mixer starts and stops with this button. Motor direction and
            calibration are configured in Settings.
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
