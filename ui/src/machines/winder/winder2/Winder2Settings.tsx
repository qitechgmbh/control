import { Page } from "@ui/components/Page";
import { ControlCard } from "@ui/control/ControlCard";
import { ControlGrid } from "@ui/control/ControlGrid";
import { EditValue } from "@ui/control/EditValue";
import React, { useState } from "react";
import { useWinder2 } from "./useWinder";
import { roundToDecimals } from "@ui/lib/decimal";
import { Label } from "@ui/control/Label";
import { SelectionGroupBoolean } from "@ui/control/SelectionGroup";
import { SelectionGroup } from "@ui/control/SelectionGroup";
import { MachineSelector } from "../MachineSelector";
import {
  getWinder2XLMode,
  setWinder2XLMode,
  WINDER2_TRAVERSE_MAX_STANDARD,
  WINDER2_TRAVERSE_MAX_XL,
  getWinder2AdaptivePullerSpeed,
  setWinder2AdaptivePullerSpeed,
} from "./winder2Config";
import { Alert, AlertDescription, AlertTitle } from "@ui/components/ui/alert";

function fractionToPercent(value: number | null | undefined) {
  return value == null ? undefined : value * 100;
}

export function Winder2SettingPage() {
  const [xlMode, setXlMode] = useState(getWinder2XLMode());
  const [adaptivePullerSpeed, setAdaptivePullerSpeedState] = useState(
    getWinder2AdaptivePullerSpeed(),
  );

  const {
    state,
    defaultState,
    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseHome,
    setPullerForward,
    setPullerGearRatio,
    setSpoolRegulationMode,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolForward,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setPullerAdaptiveMaxSpeedChangePercent,
    setPullerAdaptiveAdjustmentIntervalMeters,
    setPullerAdaptiveStepPercent,
    setPullerAdaptiveAcceptedDifference,
    setPullerAdaptiveReferenceMachine,
    filteredMachines,
    selectedMachine,
    isLoading,
    isDisabled,
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
              value={state?.spool_speed_controller_state?.regulation_mode}
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
                setSpoolRegulationMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>

          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.spool_speed_controller_state?.forward}
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
              onChange={(value) => setSpoolForward(value)}
            />
          </Label>

          {state?.spool_speed_controller_state?.regulation_mode ===
            "MinMax" && (
            <>
              <Label label="Minimum Speed">
                <EditValue
                  value={state?.spool_speed_controller_state?.minmax_min_speed}
                  title={"Minimum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_speed_controller_state?.minmax_min_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMinSpeed(value)}
                />
              </Label>
              <Label label="Maximum Speed">
                <EditValue
                  value={state?.spool_speed_controller_state?.minmax_max_speed}
                  title={"Maximum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_speed_controller_state?.minmax_max_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMaxSpeed(value)}
                />
              </Label>
            </>
          )}

          {state?.spool_speed_controller_state?.regulation_mode ===
            "Adaptive" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Tension Target">
                <EditValue
                  value={
                    state?.spool_speed_controller_state?.adaptive_tension_target
                  }
                  title={"Tension Target"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={1}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_tension_target
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setSpoolAdaptiveTensionTarget(value)}
                />
              </Label>
              <Label label="Learning Rate">
                <EditValue
                  value={
                    state?.spool_speed_controller_state
                      ?.adaptive_radius_learning_rate
                  }
                  title={"Radius Learning Rate"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
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
                    state?.spool_speed_controller_state
                      ?.adaptive_max_speed_multiplier
                  }
                  title={"Max Speed Multiplier"}
                  unit={undefined}
                  step={0.1}
                  min={0.1}
                  max={10}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
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
                    state?.spool_speed_controller_state
                      ?.adaptive_acceleration_factor
                  }
                  title={"Acceleration Factor"}
                  unit={undefined}
                  step={0.01}
                  min={0.01}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
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
                    state?.spool_speed_controller_state
                      ?.adaptive_deacceleration_urgency_multiplier
                  }
                  title={"Deacceleration Urgency Multiplier"}
                  unit={undefined}
                  step={0.5}
                  min={1}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
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
              value={state?.puller_state?.forward}
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
              onChange={(value) => setPullerForward(value)}
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
                OneToFive: {
                  children: "1:5",
                  icon: "lu:Cog",
                },
                OneToTen: {
                  children: "1:10",
                  icon: "lu:Cog",
                },
              }}
              onChange={(value) =>
                setPullerGearRatio(
                  value as "OneToOne" | "OneToFive" | "OneToTen",
                )
              }
            />
          </Label>

          <Label label="Adaptive Speed (Experimental)">
            <SelectionGroupBoolean
              value={adaptivePullerSpeed}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Disabled",
                icon: "lu:X",
                disabled:
                  adaptivePullerSpeed &&
                  state?.puller_state?.regulation === "Diameter",
              }}
              optionTrue={{
                children: "Enabled",
                icon: "lu:FlaskConical",
              }}
              onChange={(value) => {
                setWinder2AdaptivePullerSpeed(value);
                setAdaptivePullerSpeedState(value);
              }}
            />
            {adaptivePullerSpeed && (
              <Alert className="mt-2 border-yellow-500/50 bg-yellow-500/10">
                <AlertTitle className="text-yellow-600">
                  Experimental Feature
                </AlertTitle>
                <AlertDescription>
                  This feature is still in development and may cause unexpected
                  behavior. It will be improved in future updates. Please read
                  section 2.3.1 in the manual on how to use this feature and
                  provide feedback to help us improve it.
                </AlertDescription>
              </Alert>
            )}
          </Label>

          {adaptivePullerSpeed && (
            <ControlCard title="Adaptive Speed">
              <Label label="Allowed Diameter Deviation">
                <EditValue
                  value={state?.puller_state?.allowed_diameter_deviation}
                  title={"Allowed Diameter Deviation"}
                  unit="mm"
                  step={0.01}
                  min={0}
                  max={5}
                  defaultValue={
                    defaultState?.puller_state?.allowed_diameter_deviation
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) =>
                    setPullerAdaptiveAcceptedDifference(value)
                  }
                />
              </Label>
              <Label label="Max Speed Deviation">
                {state?.puller_state?.regulation === "Diameter" && (
                  <span className="text-muted-foreground text-sm">
                    Only changeable in fixed mode
                  </span>
                )}
                <EditValue
                  value={fractionToPercent(
                    state?.puller_state?.adaptive_speed_delta_max,
                  )}
                  title={"Max Speed Deviation"}
                  unit="%"
                  step={0.5}
                  min={0}
                  max={50}
                  defaultValue={fractionToPercent(
                    defaultState?.puller_state?.adaptive_speed_delta_max,
                  )}
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setPullerAdaptiveMaxSpeedChangePercent(value / 100)
                  }
                  disabled={state?.puller_state?.regulation === "Diameter"}
                />
              </Label>
              <Label label="Distance Between Steps">
                <EditValue
                  value={state?.puller_state?.adaptive_adjustment_distance}
                  title={"Distance Between Steps"}
                  unit="m"
                  step={0.1}
                  min={0}
                  max={200}
                  defaultValue={
                    defaultState?.puller_state?.adaptive_adjustment_distance
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setPullerAdaptiveAdjustmentIntervalMeters(value)
                  }
                />
              </Label>
              <Label label="Change Per Step">
                <EditValue
                  value={fractionToPercent(
                    state?.puller_state?.adaptive_change_per_step,
                  )}
                  title={"Increase Per Step"}
                  unit="%"
                  step={0.1}
                  min={0.1}
                  max={10}
                  defaultValue={fractionToPercent(
                    defaultState?.puller_state?.adaptive_change_per_step,
                  )}
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setPullerAdaptiveStepPercent(value / 100)
                  }
                />
              </Label>
              <Label label="Reference Machine">
                <MachineSelector
                  machines={filteredMachines}
                  selectedMachine={selectedMachine}
                  connectedMachineState={
                    state?.puller_state.adaptive_reference_machine
                  }
                  setConnectedMachine={(machine) => {
                    setPullerAdaptiveReferenceMachine(machine);
                  }}
                  clearConnectedMachine={() => {
                    if (!selectedMachine) return;
                    setPullerAdaptiveReferenceMachine(null);
                  }}
                />
              </Label>
            </ControlCard>
          )}
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
