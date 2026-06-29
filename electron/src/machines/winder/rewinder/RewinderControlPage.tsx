import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundToDecimals } from "@/lib/decimal";
import { TraverseBar } from "../TraverseBar";
import { Mode } from "./rewinderNamespace";
import { RewinderOverview } from "./RewinderOverview";
import { useRewinder } from "./useRewinder";
import React from "react";

const traverseMax = 120;

export function RewinderControlPage() {
  const {
    state,
    defaultState,
    traversePosition,
    pullerSpeed,
    takeupSpoolRpm,
    sourceSpoolRpm,
    takeupTensionArmAngle,
    sourceTensionArmAngle,
    rewindProgress,
    isLoading,
    isDisabled,
    setMode,
    setPullerTargetSpeed,
    zeroTakeupTensionArm,
    zeroSourceTensionArm,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    resetRewindProgress,
    enableTraverseLaserpointer,
  } = useRewinder();

  const maxTargetSpeed = 50;
  const tensionArmsZeroed =
    state?.takeup_tension_arm_state.zeroed === true &&
    state?.source_tension_arm_state.zeroed === true;

  const zeroTensionArms = () => {
    zeroTakeupTensionArm();
    zeroSourceTensionArm();
  };

  // Laserpointer controlled via EL2002 digital output — same as winder2
  const laserOn = state?.traverse_state.laserpointer ?? false;

  // Debug: preview filament line without a live machine connection
  const [debugFilamentMode, setDebugFilamentMode] = React.useState<Mode | null>(
    null,
  );
  const [debugCanRewind, setDebugCanRewind] = React.useState(false);
  const DEBUG_MODES: Mode[] = ["Standby", "Hold", "Pull", "Prepare", "Rewind"];

  const isReady = state?.mode_state.can_rewind === true;

  return (
    <Page>
      {/* Top: machine overview schematic */}
      <ControlCard title="Overview">
        <RewinderOverview
          state={state}
          takeupSpoolRpm={takeupSpoolRpm}
          sourceSpoolRpm={sourceSpoolRpm}
          traversePosition={traversePosition}
          pullerSpeed={pullerSpeed}
          takeupTensionArmAngle={takeupTensionArmAngle}
          sourceTensionArmAngle={sourceTensionArmAngle}
          modeOverride={debugFilamentMode ?? undefined}
          canRewindOverride={debugFilamentMode ? debugCanRewind : undefined}
        />
        {/* Debug row: preview filament appearance per mode */}
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs text-gray-400">Filament debug:</span>
          <button
            onClick={() => setDebugFilamentMode(null)}
            className={`rounded-full px-3 py-0.5 text-xs font-medium transition-colors ${debugFilamentMode === null ? "bg-gray-800 text-white" : "bg-gray-100 text-gray-500 hover:bg-gray-200"}`}
          >
            Live
          </button>
          {DEBUG_MODES.map((m) => (
            <button
              key={m}
              onClick={() => setDebugFilamentMode(m)}
              className={`rounded-full px-3 py-0.5 text-xs font-medium transition-colors ${debugFilamentMode === m ? "bg-gray-800 text-white" : "bg-gray-100 text-gray-500 hover:bg-gray-200"}`}
            >
              {m}
            </button>
          ))}
          {debugFilamentMode === "Prepare" && (
            <button
              onClick={() => setDebugCanRewind((v) => !v)}
              className={`rounded-full px-3 py-0.5 text-xs font-medium transition-colors ${debugCanRewind ? "bg-green-600 text-white" : "bg-amber-500 text-white"}`}
            >
              {debugCanRewind ? "can_rewind: true" : "can_rewind: false"}
            </button>
          )}
        </div>
      </ControlCard>

      {/* Bottom: 3-column row */}
      <div className="grid grid-cols-3 gap-4">
        {/* Run */}
        <ControlCard>
          <div className="flex items-start justify-between gap-2">
            <h2 className="text-2xl font-bold">Run</h2>
            <div className="flex flex-wrap justify-end gap-2">
              {isReady ? (
                <StatusBadge variant="success">Ready</StatusBadge>
              ) : (
                <StatusBadge variant="error">Not Ready</StatusBadge>
              )}
              {!tensionArmsZeroed && (
                <StatusBadge variant="error">Arms Not Zeroed</StatusBadge>
              )}
              {state?.traverse_state.is_homed !== true && (
                <StatusBadge variant="error">Traverse Not Homed</StatusBadge>
              )}
            </div>
          </div>
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setMode}
            orientation="vertical"
            className="grid grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "min-h-16",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "min-h-16",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ArrowRight",
                isActiveClassName: "bg-green-600",
                className: "min-h-16",
              },
              Prepare: {
                children: "Prepare",
                icon: "lu:Crosshair",
                isActiveClassName: "bg-green-600",
                className: "min-h-16",
                disabled: !tensionArmsZeroed,
              },
              Rewind: {
                children: "Rewind",
                icon: "lu:RefreshCw",
                isActiveClassName: "bg-green-600",
                className: "col-span-2 min-h-16",
                disabled: !isReady,
              },
            }}
          />
          <TimeSeriesValueNumeric
            label="Line Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 2)}
          />
          <EditValue
            value={state?.puller_state.target_speed}
            unit="m/min"
            title="Target Speed"
            defaultValue={defaultState?.puller_state.target_speed}
            min={0}
            max={maxTargetSpeed}
            renderValue={(value) => roundToDecimals(value, 2)}
            onChange={setPullerTargetSpeed}
          />
        </ControlCard>

        {/* Middle column: Automatic Stop + Actions stacked */}
        <div className="flex flex-col gap-4">
          <ControlCard title="Automatic Stop">
            <TimeSeriesValueNumeric
              label="Progress"
              unit="m"
              timeseries={rewindProgress}
              renderValue={(value) => roundToDecimals(value, 2)}
            />
            <EditValue
              value={state?.rewind_automatic_action_state.required_meters}
              unit="m"
              title="Required Length"
              defaultValue={
                defaultState?.rewind_automatic_action_state.required_meters
              }
              min={0}
              max={10000}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setRewindAutomaticRequiredMeters}
            />
            <Label label="After Length">
              <SelectionGroup
                value={state?.rewind_automatic_action_state.mode}
                disabled={isDisabled}
                loading={isLoading}
                options={{
                  NoAction: { children: "No Action", icon: "lu:Minus" },
                  Hold: { children: "Hold", icon: "lu:CirclePause" },
                }}
                onChange={(value) =>
                  setRewindAutomaticAction(value as "NoAction" | "Hold")
                }
              />
            </Label>
            <TouchButton
              variant="outline"
              icon="lu:RotateCcw"
              onClick={resetRewindProgress}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Reset Progress
            </TouchButton>
          </ControlCard>

          <ControlCard title="Actions">
            <TouchButton
              variant="outline"
              icon="lu:RotateCcw"
              onClick={zeroTensionArms}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Zero Tension Arms
            </TouchButton>
            <TouchButton
              variant="outline"
              icon="lu:MapPin"
              onClick={() => {
                // TODO: implement go to start position mutation
              }}
              disabled={true}
            >
              Go to Start Position
            </TouchButton>
          </ControlCard>
        </div>

        {/* Traverse */}
        <ControlCard title="Traverse">
          <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            timeseries={traversePosition}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          {state?.traverse_state && (
            <TraverseBar
              inside={0}
              outside={traverseMax}
              min={state.traverse_state.limit_inner}
              max={state.traverse_state.limit_outer}
              current={traversePosition.current?.value ?? 0}
            />
          )}
          <Label label="Outer Limit">
            <EditValue
              value={state?.traverse_state.limit_outer}
              unit="mm"
              title="Outer Limit"
              defaultValue={defaultState?.traverse_state.limit_outer}
              min={Math.max(0, (state?.traverse_state.limit_inner ?? 0) + 1)}
              max={traverseMax}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTraverseLimitOuter}
            />
            <TouchButton
              variant="outline"
              icon="lu:ArrowLeftToLine"
              onClick={gotoTraverseLimitOuter}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Go to Outer Limit
            </TouchButton>
          </Label>
          <Label label="Inner Limit">
            <EditValue
              value={state?.traverse_state.limit_inner}
              unit="mm"
              title="Inner Limit"
              defaultValue={defaultState?.traverse_state.limit_inner}
              min={0}
              max={Math.min(
                traverseMax,
                (state?.traverse_state.limit_outer ?? traverseMax) - 1,
              )}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTraverseLimitInner}
            />
            <TouchButton
              variant="outline"
              icon="lu:ArrowRightToLine"
              onClick={gotoTraverseLimitInner}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Go to Inner Limit
            </TouchButton>
          </Label>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={laserOn}
              onChange={enableTraverseLaserpointer}
              optionTrue={{
                children: "On",
                icon: "lu:Lightbulb",
                isActiveClassName: "bg-green-600",
              }}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
            />
          </Label>
          <Label label="Home">
            <TouchButton
              variant="outline"
              icon="lu:House"
              onClick={gotoTraverseHome}
              disabled={isDisabled}
              isLoading={isLoading}
            >
              Go to Home
            </TouchButton>
            {state?.traverse_state.is_homed !== true ? (
              <StatusBadge variant="error">Not Homed</StatusBadge>
            ) : null}
          </Label>
        </ControlCard>
      </div>
    </Page>
  );
}
