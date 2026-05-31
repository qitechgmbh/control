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
    isLoading,
    isDisabled,
    setMode,
    setPullerTargetSpeed,
    setPullerGearRatio,
    setTakeupSpoolRegulationMode,
    setTakeupSpoolMinMaxMinSpeed,
    setTakeupSpoolMinMaxMaxSpeed,
    setTakeupTensionTarget,
    setTakeupSpoolAdaptiveRadiusLearningRate,
    setTakeupSpoolAdaptiveMaxSpeedMultiplier,
    setTakeupSpoolAdaptiveAccelerationFactor,
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setSourceTensionTarget,
    zeroTakeupTensionArm,
    zeroSourceTensionArm,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
  } = useRewinder();

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
            max={10}
            renderValue={(value) => roundToDecimals(value, 2)}
            onChange={setPullerTargetSpeed}
          />
          <Label label="Gear Ratio">
            <SelectionGroup
              value={state?.puller_state.gear_ratio}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                OneToOne: { children: "1:1", icon: "lu:Circle" },
                OneToFive: { children: "1:5", icon: "lu:Cog" },
                OneToTen: { children: "1:10", icon: "lu:Cog" },
              }}
              onChange={(value) =>
                setPullerGearRatio(
                  value as "OneToOne" | "OneToFive" | "OneToTen",
                )
              }
            />
          </Label>
        </ControlCard>

        <ControlCard title="Takeup Spool">
          <Spool rpm={takeupSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Speed"
            unit="rpm"
            timeseries={takeupSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
          <Label label="Speed Algorithm">
            <SelectionGroup
              value={state?.takeup_spool_state.regulation_mode}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                MinMax: { children: "Min/Max", icon: "lu:ArrowUpDown" },
                Adaptive: { children: "Adaptive", icon: "lu:Brain" },
              }}
              onChange={(value) =>
                setTakeupSpoolRegulationMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>
          {state?.takeup_spool_state.regulation_mode === "MinMax" && (
            <>
              <EditValue
                value={state?.takeup_spool_state.minmax_min_speed}
                title="Minimum Speed"
                unit="rpm"
                step={10}
                min={0}
                max={600}
                defaultValue={defaultState?.takeup_spool_state.minmax_min_speed}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setTakeupSpoolMinMaxMinSpeed}
              />
              <EditValue
                value={state?.takeup_spool_state.minmax_max_speed}
                title="Maximum Speed"
                unit="rpm"
                step={10}
                min={0}
                max={600}
                defaultValue={defaultState?.takeup_spool_state.minmax_max_speed}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setTakeupSpoolMinMaxMaxSpeed}
              />
            </>
          )}
          <EditValue
            value={state?.takeup_spool_state.adaptive_tension_target}
            title="Tension Target"
            defaultValue={defaultState?.takeup_spool_state.adaptive_tension_target}
            min={0}
            max={1}
            renderValue={(value) => roundToDecimals(value, 2)}
            onChange={setTakeupTensionTarget}
          />
          {state?.takeup_spool_state.regulation_mode === "Adaptive" && (
            <>
              <EditValue
                value={
                  state?.takeup_spool_state.adaptive_radius_learning_rate
                }
                title="Radius Learning Rate"
                step={0.001}
                min={0}
                max={100}
                defaultValue={
                  defaultState?.takeup_spool_state
                    .adaptive_radius_learning_rate
                }
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={setTakeupSpoolAdaptiveRadiusLearningRate}
              />
              <EditValue
                value={
                  state?.takeup_spool_state.adaptive_max_speed_multiplier
                }
                title="Max Speed Multiplier"
                step={0.1}
                min={0.1}
                max={10}
                defaultValue={
                  defaultState?.takeup_spool_state
                    .adaptive_max_speed_multiplier
                }
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={setTakeupSpoolAdaptiveMaxSpeedMultiplier}
              />
              <EditValue
                value={state?.takeup_spool_state.adaptive_acceleration_factor}
                title="Acceleration Factor"
                step={0.01}
                min={0.01}
                max={100}
                defaultValue={
                  defaultState?.takeup_spool_state
                    .adaptive_acceleration_factor
                }
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={setTakeupSpoolAdaptiveAccelerationFactor}
              />
              <EditValue
                value={
                  state?.takeup_spool_state
                    .adaptive_deacceleration_urgency_multiplier
                }
                title="Deaccel. Urgency"
                step={0.5}
                min={1}
                max={100}
                defaultValue={
                  defaultState?.takeup_spool_state
                    .adaptive_deacceleration_urgency_multiplier
                }
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={
                  setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier
                }
              />
            </>
          )}
        </ControlCard>

        <ControlCard title="Source Spool">
          <Spool rpm={sourceSpoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Speed"
            unit="rpm"
            timeseries={sourceSpoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
          <EditValue
            value={state?.source_spool_state.adaptive_tension_target}
            title="Tension Target"
            defaultValue={defaultState?.source_spool_state.adaptive_tension_target}
            min={0}
            max={1}
            renderValue={(value) => roundToDecimals(value, 2)}
            onChange={setSourceTensionTarget}
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
