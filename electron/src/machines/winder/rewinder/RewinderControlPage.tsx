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
import { RewinderOverview } from "./RewinderOverview";
import { useRewinder } from "./useRewinder";

const TRAVERSE_MAX_MM = 180;
const MAX_TARGET_SPEED_M_PER_MIN = 50;

function formatEta(seconds: number): string {
  if (seconds < 60) {
    return `${Math.ceil(seconds)} s`;
  }

  const totalMinutes = Math.ceil(seconds / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) {
    return `${totalMinutes} min`;
  }

  return minutes === 0 ? `${hours} h` : `${hours} h ${minutes} min`;
}

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
    isDecelerating,
    progressResetPermitted,
    setMode,
    setPullerTargetSpeed,
    zeroTakeupTensionArm,
    zeroSourceTensionArm,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStartPosition,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseStartPosition,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    resetRewindProgress,
    enableTraverseLaserpointer,
    hardStop,
  } = useRewinder();

  const tensionArmsZeroed =
    state?.takeup_tension_arm_state.zeroed === true &&
    state?.source_tension_arm_state.zeroed === true;

  const zeroTensionArms = () => {
    zeroTakeupTensionArm();
    zeroSourceTensionArm();
  };

  // Laserpointer controlled via EL2002 digital output — same as winder2
  const laserOn = state?.traverse_state.laserpointer ?? false;

  const isReady = state?.mode_state.can_rewind === true;
  const settingsEditable =
    !isDecelerating &&
    (state?.mode_state.mode === "Standby" || state?.mode_state.mode === "Hold");
  const commandsDisabled = isDisabled || isLoading;
  const modeSelectionDisabled = commandsDisabled;
  const motionCommandDisabled = commandsDisabled || isDecelerating;
  const settingsDisabled = commandsDisabled || !settingsEditable;
  const traverseSettingsDisabled = settingsDisabled;
  const manualTraverseAllowed =
    !isDecelerating &&
    state?.mode_state.mode === "Hold" &&
    state?.traverse_state.is_homed === true;
  const traverseMoveDisabled =
    isDisabled || isLoading || !manualTraverseAllowed;
  const requiredMeters =
    state?.rewind_automatic_action_state.required_meters ?? 0;
  const progressMeters = rewindProgress.current?.value ?? 0;
  const remainingMeters = Math.max(requiredMeters - progressMeters, 0);
  const lineSpeedMPerMin = Math.abs(pullerSpeed.current?.value ?? 0);
  const targetSpeedMPerMin = Math.abs(state?.puller_state.target_speed ?? 0);
  const autoStopEnabled =
    state?.rewind_automatic_action_state.mode !== "NoAction" &&
    requiredMeters > 0;
  const etaSpeedMPerMin =
    targetSpeedMPerMin > 0.01 ? targetSpeedMPerMin : lineSpeedMPerMin;
  const etaSeconds =
    autoStopEnabled && remainingMeters > 0 && etaSpeedMPerMin > 0.01
      ? (remainingMeters / etaSpeedMPerMin) * 60
      : null;
  const etaText = !autoStopEnabled
    ? "Auto stop off"
    : requiredMeters <= 0
      ? "Set target length"
      : remainingMeters <= 0
        ? "Target reached"
        : etaSeconds == null
          ? "Set line speed"
          : formatEta(etaSeconds);

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
        />
      </ControlCard>

      {/* Bottom: 3-column row */}
      <div className="grid grid-cols-3 gap-4">
        {/* Run */}
        <ControlCard>
          <div className="flex items-start justify-between gap-2">
            <h2 className="text-2xl font-bold">Run</h2>
            <div className="flex flex-wrap justify-end gap-2">
              {isDecelerating ? (
                <StatusBadge variant="warning">Decelerating</StatusBadge>
              ) : isReady ? (
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
              {(state?.takeup_spool_state.diameter_mm == null ||
                state?.source_spool_state.diameter_mm == null) && (
                <StatusBadge variant="error">Set Spool Diameters</StatusBadge>
              )}
            </div>
          </div>
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={modeSelectionDisabled}
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
                disabled: isDecelerating,
              },
              Prepare: {
                children: "Prepare",
                icon: "lu:Crosshair",
                isActiveClassName: isReady ? "bg-green-600" : "bg-amber-500",
                className: "min-h-16",
                disabled: isDecelerating || !tensionArmsZeroed,
              },
              Rewind: {
                children: "Rewind",
                icon: "lu:RefreshCw",
                isActiveClassName: "bg-green-600",
                className: "col-span-2 min-h-16",
                disabled: isDecelerating || !isReady,
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
            max={MAX_TARGET_SPEED_M_PER_MIN}
            renderValue={(value) => roundToDecimals(value, 2)}
            disabled={motionCommandDisabled}
            onChange={setPullerTargetSpeed}
          />
          <TouchButton
            variant="outline"
            icon="lu:RotateCcw"
            onClick={zeroTensionArms}
            disabled={settingsDisabled}
            isLoading={isLoading}
          >
            Zero Tension Arms
          </TouchButton>
          <TouchButton
            variant="destructive"
            icon="lu:OctagonX"
            onClick={hardStop}
            disabled={
              commandsDisabled ||
              (!isDecelerating && state?.mode_state.mode !== "Rewind")
            }
            isLoading={isLoading}
          >
            Hard Stop
          </TouchButton>
        </ControlCard>

        <ControlCard title="Automatic Stop">
          <TimeSeriesValueNumeric
            label="Progress"
            unit="m"
            timeseries={rewindProgress}
            renderValue={(value) => roundToDecimals(value, 2)}
          />
          <div className="rounded-xl border border-gray-200 bg-gray-50 px-4 py-3">
            <div className="text-sm text-gray-500">Estimated Time</div>
            <div className="font-mono text-2xl font-semibold text-gray-900">
              {etaText}
            </div>
          </div>
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
            disabled={motionCommandDisabled}
            onChange={setRewindAutomaticRequiredMeters}
          />
          <Label label="After Length">
            <SelectionGroup
              value={state?.rewind_automatic_action_state.mode}
              disabled={motionCommandDisabled}
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
            disabled={commandsDisabled || !progressResetPermitted}
          >
            Reset Progress
          </TouchButton>
        </ControlCard>

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
              outside={TRAVERSE_MAX_MM}
              min={state.traverse_state.limit_inner}
              max={state.traverse_state.limit_outer}
              current={traversePosition.current?.value ?? 0}
            />
          )}
          <Label label="Outer Limit">
            <div className="flex items-center gap-2">
              <div className="min-w-0 flex-1">
                <EditValue
                  value={state?.traverse_state.limit_outer}
                  unit="mm"
                  title="Outer Limit"
                  defaultValue={defaultState?.traverse_state.limit_outer}
                  min={Math.max(
                    0,
                    (state?.traverse_state.limit_inner ?? 0) + 1,
                  )}
                  max={TRAVERSE_MAX_MM}
                  disabled={traverseSettingsDisabled}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={setTraverseLimitOuter}
                />
              </div>
              <TouchButton
                variant="outline"
                icon="lu:ArrowLeftToLine"
                onClick={gotoTraverseLimitOuter}
                disabled={traverseMoveDisabled}
                isLoading={isLoading}
              >
                Go
              </TouchButton>
            </div>
          </Label>
          <Label label="Start Position">
            <div className="flex items-center gap-2">
              <div className="min-w-0 flex-1">
                <EditValue
                  value={state?.traverse_state.start_position}
                  unit="mm"
                  title="Start Position"
                  defaultValue={defaultState?.traverse_state.start_position}
                  min={state?.traverse_state.limit_inner ?? 0}
                  max={state?.traverse_state.limit_outer ?? TRAVERSE_MAX_MM}
                  disabled={traverseSettingsDisabled}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={setTraverseStartPosition}
                />
              </div>
              <TouchButton
                variant="outline"
                icon="lu:MapPin"
                onClick={gotoTraverseStartPosition}
                disabled={
                  traverseMoveDisabled ||
                  state?.traverse_state.is_homed !== true
                }
                isLoading={isLoading}
              >
                Go
              </TouchButton>
            </div>
          </Label>
          <Label label="Inner Limit">
            <div className="flex items-center gap-2">
              <div className="min-w-0 flex-1">
                <EditValue
                  value={state?.traverse_state.limit_inner}
                  unit="mm"
                  title="Inner Limit"
                  defaultValue={defaultState?.traverse_state.limit_inner}
                  min={0}
                  max={Math.min(
                    TRAVERSE_MAX_MM,
                    (state?.traverse_state.limit_outer ?? TRAVERSE_MAX_MM) - 1,
                  )}
                  disabled={traverseSettingsDisabled}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={setTraverseLimitInner}
                />
              </div>
              <TouchButton
                variant="outline"
                icon="lu:ArrowRightToLine"
                onClick={gotoTraverseLimitInner}
                disabled={traverseMoveDisabled}
                isLoading={isLoading}
              >
                Go
              </TouchButton>
            </div>
          </Label>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={laserOn}
              onChange={enableTraverseLaserpointer}
              disabled={settingsDisabled}
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
              disabled={commandsDisabled || state?.mode_state.mode !== "Hold"}
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
