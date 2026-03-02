import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import React, { useState } from "react";
import { useWinder2 } from "./useWinder";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { SelectionGroup } from "@/control/SelectionGroup";
import { MachineSelector } from "@/components/MachineConnectionDropdown";
import {
  getWinder2XLMode,
  setWinder2XLMode,
  WINDER2_TRAVERSE_MAX_STANDARD,
  WINDER2_TRAVERSE_MAX_XL,
} from "./winder2Config";

export function Winder2SettingPage() {
  const [xlMode, setXlMode] = useState(getWinder2XLMode());

  const {
    state,
    defaultState,
    isLoading,
    isDisabled,
    // setters
    setMode,
    setSpoolDirection,
    setSpoolSpeedControlMode,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setPullerDirection,
    setPullerGearRatio,
    setPullerSpeedControlMode,
    setPullerFixedTargetSpeed,
    setPullerAdaptiveBaseSpeed,
    setPullerAdaptiveDeviationMax,
    setPullerAdaptiveReferenceMachine,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseHome,
    setTraverseLaserpointerEnabled,
    calibrateTensionArmAngle,
    setSpoolLengthTaskTargetLength,
    resetSpoolLengthTaskProgress,
    setOnSpoolLengthTaskCompletedAction,
  } = useWinder2();

  const handleXlModeChange = (enabled: boolean) => {
    setWinder2XLMode(enabled);
    setXlMode(enabled);

    // When switching from XL to normal mode, reset traverse limits to default values
    if (!enabled && defaultState) {
      // Only reset if current values exceed the standard max
      const currentOuter = state?.traverse_state?.limit_outer ?? 0;
      const currentInner = state?.traverse_state?.limit_inner ?? 0;
      const defaultOuter = defaultState.traverse_state?.limit_outer;
      const defaultInner = defaultState.traverse_state?.limit_inner;

      if (
        currentOuter > WINDER2_TRAVERSE_MAX_STANDARD &&
        defaultOuter !== undefined
      ) {
        setTraverseLimitOuter(defaultOuter);
        setTraverseLimitInner(defaultInner);
      }
      if (
        currentInner > WINDER2_TRAVERSE_MAX_STANDARD &&
        defaultInner !== undefined
      ) {
        setTraverseLimitOuter(defaultOuter);
        setTraverseLimitInner(defaultInner);
      }

      // Home the traverse when switching from XL to normal mode
      gotoTraverseHome();
    }
  };

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Traverse">
          <Label label="Traverse Size">
            <SelectionGroupBoolean
              value={xlMode}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: `Standard (${WINDER2_TRAVERSE_MAX_STANDARD}mm)`,
                icon: "lu:Settings",
              }}
              optionTrue={{
                children: `XL (${WINDER2_TRAVERSE_MAX_XL}mm)`,
                icon: "lu:Maximize2",
              }}
              onChange={handleXlModeChange}
            />
          </Label>
          <Label label="Step Size">
            <EditValue
              value={state?.traverse_state?.step_size}
              title={"Step Size"}
              unit="mm"
              step={0.05}
              min={0.1}
              max={75}
              defaultValue={defaultState?.traverse_state?.step_size}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => setTraverseStepSize(value)}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={state?.traverse_state?.padding}
              title={"Padding"}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              defaultValue={defaultState?.traverse_state?.padding}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => setTraversePadding(value)}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Spool">
          <Label label="Speed Algorithm">
            <SelectionGroup
              value={state?.spool_state?.speed_control_mode}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                MinMax: {
                  children: "Min/Max",
                  icon: "lu:ArrowUpDown",
                },
                Adaptive: {
                  children: "Adaptive",
                  icon: "lu:Brain",
                },
              }}
              onChange={(value) =>
                setSpoolSpeedControlMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>

          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.spool_state?.direction == "Forward"}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setSpoolDirection(value ? "Forward" : "Reverse")}
            />
          </Label>

          {state?.spool_state?.speed_control_mode ===
            "MinMax" && (
            <>
              <Label label="Minimum Speed">
                <EditValue
                  value={state?.spool_state?.minmax_min_speed}
                  title={"Minimum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_state?.minmax_min_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMinSpeed(value)}
                />
              </Label>
              <Label label="Maximum Speed">
                <EditValue
                  value={state?.spool_state?.minmax_max_speed}
                  title={"Maximum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_state?.minmax_max_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMaxSpeed(value)}
                />
              </Label>
            </>
          )}

          {state?.spool_state?.speed_control_mode ===
            "Adaptive" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Tension Target">
                <EditValue
                  value={
                    state?.spool_state?.adaptive_tension_target
                  }
                  title={"Tension Target"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={1}
                  defaultValue={
                    defaultState?.spool_state
                      ?.adaptive_tension_target
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setSpoolAdaptiveTensionTarget(value)}
                />
              </Label>
              <Label label="Learning Rate">
                <EditValue
                  value={
                    state?.spool_state
                      ?.adaptive_radius_learning_rate
                  }
                  title={"Radius Learning Rate"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={100}
                  defaultValue={
                    defaultState?.spool_state
                      ?.adaptive_radius_learning_rate
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) =>
                    setSpoolAdaptiveRadiusLearningRate(value)
                  }
                />
              </Label>
              <Label label="Max Speed Multiplier">
                <EditValue
                  value={
                    state?.spool_state
                      ?.adaptive_max_speed_multiplier
                  }
                  title={"Max Speed Multiplier"}
                  unit={undefined}
                  step={0.1}
                  min={0.1}
                  max={10}
                  defaultValue={
                    defaultState?.spool_state
                      ?.adaptive_max_speed_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setSpoolAdaptiveMaxSpeedMultiplier(value)
                  }
                />
              </Label>
              <Label label="Acceleration Factor">
                <EditValue
                  value={
                    state?.spool_state
                      ?.adaptive_acceleration_factor
                  }
                  title={"Acceleration Factor"}
                  unit={undefined}
                  step={0.01}
                  min={0.01}
                  max={100}
                  defaultValue={
                    defaultState?.spool_state
                      ?.adaptive_acceleration_factor
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) =>
                    setSpoolAdaptiveAccelerationFactor(value)
                  }
                />
              </Label>
              <Label label="Deaccel. Urgency">
                <EditValue
                  value={
                    state?.spool_state
                      ?.adaptive_deacceleration_urgency_multiplier
                  }
                  title={"Deacceleration Urgency Multiplier"}
                  unit={undefined}
                  step={0.5}
                  min={1}
                  max={100}
                  defaultValue={
                    defaultState?.spool_state
                      ?.adaptive_deacceleration_urgency_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setSpoolAdaptiveDeaccelerationUrgencyMultiplier(value)
                  }
                />
              </Label>
            </div>
          )}
        </ControlCard>

        <ControlCard title="Puller">
          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.puller_state?.direction == "Forward"}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setPullerDirection(value ? "Forward" : "Reverse")}
            />
          </Label>
          <Label label="Gear Ratio">
            <SelectionGroup
              value={state?.puller_state?.gear_ratio}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                OneToOne: {
                  children: "1:1",
                  icon: "lu:Circle",
                },
                FiveToOne: {
                  children: "5:1",
                  icon: "lu:Cog",
                },
                TenToOne: {
                  children: "10:1",
                  icon: "lu:Cog",
                },
              }}
              onChange={(value) =>
                setPullerGearRatio(
                  value as "OneToOne" | "FiveToOne" | "TenToOne",
                )
              }
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
