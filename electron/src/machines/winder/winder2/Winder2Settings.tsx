import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import React from "react";
import { useWinder2 } from "./useWinder";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { SelectionGroup } from "@/control/SelectionGroup";

export function Winder2SettingPage() {
  const {
    traverseState,
    traverseSetStepSize,
    traverseSetPadding,
    pullerState,
    pullerSetForward,
    pullerStateIsDisabled,
    pullerStateIsLoading,
    spoolSpeedControllerState,
    setRegulationMode,
    setMinMaxMinSpeed,
    setMinMaxMaxSpeed,
    setAdaptiveTensionTarget,
    setAdaptiveRadiusLearningRate,
    setAdaptiveMaxSpeedMultiplier,
    setAdaptiveAccelerationFactor,
    setAdaptiveDeaccelerationUrgencyMultiplier,
    spoolControllerIsDisabled,
    spoolControllerIsLoading,
  } = useWinder2();

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Traverse">
          <Label label="Step Size">
            <EditValue
              value={traverseState?.data.step_size}
              title={"Step Size"}
              unit="mm"
              step={0.05}
              min={0.1}
              max={10}
              defaultValue={1.0}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => traverseSetStepSize(value)}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={traverseState?.data.padding}
              title={"Padding"}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              defaultValue={0.01}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => traverseSetPadding(value)}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Spool">
          <Label label="Speed Algorithm">
            <SelectionGroup
              value={spoolSpeedControllerState?.data.regulation_mode}
              disabled={spoolControllerIsDisabled}
              loading={spoolControllerIsLoading}
              options={{
                MinMax: {
                  children: "Min/Max",
                  icon: "lu:ArrowUpDown",
                },
                Adaptive: {
                  children: "Adaptive (beta)",
                  icon: "lu:Brain",
                },
              }}
              onChange={(value) =>
                setRegulationMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>

          {spoolSpeedControllerState?.data.regulation_mode === "MinMax" && (
            <>
              <Label label="Minimum Speed">
                <EditValue
                  value={spoolSpeedControllerState?.data.minmax_min_speed}
                  title={"Minimum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={50}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setMinMaxMinSpeed(value)}
                />
              </Label>
              <Label label="Maximum Speed">
                <EditValue
                  value={spoolSpeedControllerState?.data.minmax_max_speed}
                  title={"Maximum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={150}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setMinMaxMaxSpeed(value)}
                />
              </Label>
            </>
          )}

          {spoolSpeedControllerState?.data.regulation_mode === "Adaptive" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Tension Target">
                <EditValue
                  value={
                    spoolSpeedControllerState?.data.adaptive_tension_target
                  }
                  title={"Tension Target"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={1}
                  defaultValue={0.7}
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setAdaptiveTensionTarget(value)}
                />
              </Label>
              <Label label="Learning Rate">
                <EditValue
                  value={
                    spoolSpeedControllerState?.data
                      .adaptive_radius_learning_rate
                  }
                  title={"Radius Learning Rate"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={100}
                  defaultValue={0.5}
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setAdaptiveRadiusLearningRate(value)}
                />
              </Label>
              <Label label="Max Speed Multiplier">
                <EditValue
                  value={
                    spoolSpeedControllerState?.data
                      .adaptive_max_speed_multiplier
                  }
                  title={"Max Speed Multiplier"}
                  unit={undefined}
                  step={0.1}
                  min={0.1}
                  max={10}
                  defaultValue={4.0}
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) => setAdaptiveMaxSpeedMultiplier(value)}
                />
              </Label>
              <Label label="Acceleration Factor">
                <EditValue
                  value={
                    spoolSpeedControllerState?.data.adaptive_acceleration_factor
                  }
                  title={"Acceleration Factor"}
                  unit={undefined}
                  step={0.01}
                  min={0.01}
                  max={10}
                  defaultValue={0.3}
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setAdaptiveAccelerationFactor(value)}
                />
              </Label>
              <Label label="Deaccel. Urgency">
                <EditValue
                  value={
                    spoolSpeedControllerState?.data
                      .adaptive_deacceleration_urgency_multiplier
                  }
                  title={"Deacceleration Urgency Multiplier"}
                  unit={undefined}
                  step={1}
                  min={1}
                  max={100}
                  defaultValue={40.0}
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setAdaptiveDeaccelerationUrgencyMultiplier(value)
                  }
                />
              </Label>
            </div>
          )}
        </ControlCard>

        <ControlCard title="Puller">
          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={pullerState?.data.forward}
              disabled={pullerStateIsDisabled}
              loading={pullerStateIsLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => pullerSetForward(value)}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
