import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { Spool } from "../Spool";
import { TensionArm } from "../TensionArm";
import { TraverseBar } from "../TraverseBar";
import { Mode } from "./rewinderNamespace";
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
  } = useRewinder();
  const maxTargetSpeed = 50;
  const tensionArmsZeroed =
    state?.takeup_tension_arm_state.zeroed === true &&
    state?.source_tension_arm_state.zeroed === true;
  const zeroTensionArms = () => {
    zeroTakeupTensionArm();
    zeroSourceTensionArm();
  };

  return (
    <Page>
      <ControlGrid>
        <ControlCard width={2} title="Run">
          <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
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
                  disabled: state?.mode_state.can_rewind !== true,
                },
              }}
            />
            <div className="flex flex-col gap-4">
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
              <div className="flex flex-wrap gap-2">
                {state?.mode_state.can_rewind !== true ? (
                  <StatusBadge variant="error">Not Ready</StatusBadge>
                ) : (
                  <StatusBadge variant="success">Ready</StatusBadge>
                )}
                {!tensionArmsZeroed ? (
                  <StatusBadge variant="error">Arms Not Zeroed</StatusBadge>
                ) : null}
                {state?.traverse_state.is_homed !== true ? (
                  <StatusBadge variant="error">Traverse Not Homed</StatusBadge>
                ) : null}
              </div>
            </div>
          </div>
        </ControlCard>

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
            defaultValue={defaultState?.rewind_automatic_action_state.required_meters}
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

        <ControlCard title="Source Side">
          <Spool rpm={sourceSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Source Spool"
            unit="rpm"
            timeseries={sourceSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard title="Tension Arms">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="flex flex-col gap-3">
              <h3 className="text-lg font-semibold">Source</h3>
              <TensionArm degrees={sourceTensionArmAngle.current?.value} />
              <TimeSeriesValueNumeric
                label="Angle"
                unit="deg"
                timeseries={sourceTensionArmAngle}
                renderValue={(value) => roundDegreesToDecimals(value, 0)}
              />
              {!state?.source_tension_arm_state.zeroed && (
                <StatusBadge variant="error">Not Zeroed</StatusBadge>
              )}
            </div>
            <div className="flex flex-col gap-3">
              <h3 className="text-lg font-semibold">Takeup</h3>
              <TensionArm degrees={takeupTensionArmAngle.current?.value} />
              <TimeSeriesValueNumeric
                label="Angle"
                unit="deg"
                timeseries={takeupTensionArmAngle}
                renderValue={(value) => roundDegreesToDecimals(value, 0)}
              />
              {!state?.takeup_tension_arm_state.zeroed && (
                <StatusBadge variant="error">Not Zeroed</StatusBadge>
              )}
            </div>
          </div>
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroTensionArms}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Both Zero Points
          </TouchButton>
        </ControlCard>

        <ControlCard title="Takeup Side">
          <Spool rpm={takeupSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Takeup Spool"
            unit="rpm"
            timeseries={takeupSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard width={3} title="Traverse">
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
      </ControlGrid>
    </Page>
  );
}
