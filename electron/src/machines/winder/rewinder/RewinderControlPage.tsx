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
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    resetRewindProgress,
  } = useRewinder();
  const maxTargetSpeed = 50;

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Mode">
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ArrowRight",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Prepare: {
                children: "Prepare",
                icon: "lu:Crosshair",
                isActiveClassName: "bg-green-600",
                className: "h-full",
                disabled:
                  state?.takeup_tension_arm_state.zeroed !== true ||
                  state?.source_tension_arm_state.zeroed !== true,
              },
              Rewind: {
                children: "Rewind",
                icon: "lu:RefreshCw",
                isActiveClassName: "bg-green-600",
                className: "h-full",
                disabled: state?.mode_state.can_rewind !== true,
              },
            }}
          />
          {state?.mode_state.can_rewind !== true ? (
            <StatusBadge variant="error">Not Ready</StatusBadge>
          ) : null}
        </ControlCard>

        <ControlCard title="Puller">
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

        <ControlCard title="Takeup Spool">
          <Spool rpm={takeupSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Speed"
            unit="rpm"
            timeseries={takeupSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard title="Source Spool">
          <Spool rpm={sourceSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Speed"
            unit="rpm"
            timeseries={sourceSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard title="Takeup Tension Arm">
          <TensionArm degrees={takeupTensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Angle"
            unit="deg"
            timeseries={takeupTensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroTakeupTensionArm}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.takeup_tension_arm_state.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard title="Source Tension Arm">
          <TensionArm degrees={sourceTensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Angle"
            unit="deg"
            timeseries={sourceTensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroSourceTensionArm}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.source_tension_arm_state.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard height={2} title="Traverse">
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
          <Label label="Step Size">
            <EditValue
              value={state?.traverse_state.step_size}
              unit="mm"
              title="Step Size"
              defaultValue={defaultState?.traverse_state.step_size}
              step={0.05}
              min={0.1}
              max={75}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setTraverseStepSize}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={state?.traverse_state.padding}
              unit="mm"
              title="Padding"
              defaultValue={defaultState?.traverse_state.padding}
              step={0.01}
              min={0}
              max={5}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setTraversePadding}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
