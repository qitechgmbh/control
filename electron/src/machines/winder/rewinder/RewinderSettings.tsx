import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useRewinder } from "./useRewinder";

export function RewinderSettingsPage() {
  const {
    state,
    defaultState,
    isDisabled,
    isLoading,
    settingsEditPermitted,
    prepareSettingsEditPermitted,
    setTraverseStepSize,
    setTraversePadding,
    setTakeupSpoolRegulationMode,
    setTakeupSpoolMinMaxMinSpeed,
    setTakeupSpoolMinMaxMaxSpeed,
    setTakeupTensionTarget,
    setTakeupSpoolAdaptiveRadiusLearningRate,
    setTakeupSpoolAdaptiveMaxSpeedMultiplier,
    setTakeupSpoolAdaptiveAccelerationFactor,
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setTakeupSpoolDiameter,
    setSourceSpoolDiameter,
    setSourceTensionTarget,
    setTakeupTensionArmControl,
    setSourceTensionArmControl,
    setPrepareControl,
  } = useRewinder();
  const settingsDisabled = isDisabled || isLoading || !settingsEditPermitted;
  const prepareSettingsDisabled =
    isDisabled || isLoading || !prepareSettingsEditPermitted;

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Takeup Spool">
          <Label label="Diameter">
            <EditValue
              value={state?.takeup_spool_state.diameter_mm ?? 100}
              title="Takeup Spool Diameter"
              unit="mm"
              step={1}
              min={10}
              max={500}
              disabled={settingsDisabled}
              defaultValue={defaultState?.takeup_spool_state.diameter_mm ?? 100}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTakeupSpoolDiameter}
            />
            {state?.takeup_spool_state.diameter_mm == null ? (
              <span className="text-sm text-amber-600">
                Not set yet. Confirm for better takeup feed-forward.
              </span>
            ) : null}
          </Label>
          <Label label="Speed Algorithm">
            <SelectionGroup
              value={state?.takeup_spool_state.regulation_mode}
              disabled={settingsDisabled}
              loading={isLoading}
              options={{
                Adaptive: { children: "Adaptive", icon: "lu:Brain" },
                MinMax: { children: "Min/Max", icon: "lu:ArrowUpDown" },
              }}
              onChange={(value) =>
                setTakeupSpoolRegulationMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>

          {state?.takeup_spool_state.regulation_mode === "Adaptive" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Tension Target">
                <EditValue
                  value={state?.takeup_spool_state.adaptive_tension_target}
                  title="Takeup Tension Target"
                  step={0.01}
                  min={0}
                  max={1}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state.adaptive_tension_target
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={setTakeupTensionTarget}
                />
              </Label>
              <Label label="Learning Rate">
                <EditValue
                  value={
                    state?.takeup_spool_state.adaptive_radius_learning_rate
                  }
                  title="Radius Learning Rate"
                  step={0.001}
                  min={0}
                  max={100}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state
                      .adaptive_radius_learning_rate
                  }
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={setTakeupSpoolAdaptiveRadiusLearningRate}
                />
              </Label>
              <Label label="Max Speed Multiplier">
                <EditValue
                  value={
                    state?.takeup_spool_state.adaptive_max_speed_multiplier
                  }
                  title="Max Speed Multiplier"
                  step={0.1}
                  min={0.1}
                  max={10}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state
                      .adaptive_max_speed_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={setTakeupSpoolAdaptiveMaxSpeedMultiplier}
                />
              </Label>
              <Label label="Acceleration Factor">
                <EditValue
                  value={state?.takeup_spool_state.adaptive_acceleration_factor}
                  title="Acceleration Factor"
                  step={0.01}
                  min={0.01}
                  max={100}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state
                      .adaptive_acceleration_factor
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={setTakeupSpoolAdaptiveAccelerationFactor}
                />
              </Label>
              <Label label="Deaccel. Urgency">
                <EditValue
                  value={
                    state?.takeup_spool_state
                      .adaptive_deacceleration_urgency_multiplier
                  }
                  title="Deacceleration Urgency"
                  step={0.5}
                  min={1}
                  max={100}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state
                      .adaptive_deacceleration_urgency_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={
                    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier
                  }
                />
              </Label>
            </div>
          )}

          {state?.takeup_spool_state.regulation_mode === "MinMax" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Min Speed">
                <EditValue
                  value={state?.takeup_spool_state.minmax_min_speed}
                  title="Takeup Min Speed"
                  unit="rpm"
                  step={1}
                  min={0}
                  max={120}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state.minmax_min_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={setTakeupSpoolMinMaxMinSpeed}
                />
              </Label>
              <Label label="Max Speed">
                <EditValue
                  value={state?.takeup_spool_state.minmax_max_speed}
                  title="Takeup Max Speed"
                  unit="rpm"
                  step={1}
                  min={5}
                  max={180}
                  disabled={settingsDisabled}
                  defaultValue={
                    defaultState?.takeup_spool_state.minmax_max_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={setTakeupSpoolMinMaxMaxSpeed}
                />
              </Label>
            </div>
          )}
        </ControlCard>

        <ControlCard title="Source Spool">
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Diameter">
              <EditValue
                value={state?.source_spool_state.diameter_mm ?? 100}
                title="Source Spool Diameter"
                unit="mm"
                step={1}
                min={10}
                max={500}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_spool_state.diameter_mm ?? 100
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setSourceSpoolDiameter}
              />
              {state?.source_spool_state.diameter_mm == null ? (
                <span className="text-sm text-amber-600">
                  Not set yet. Confirm for better source feed-forward.
                </span>
              ) : null}
            </Label>
            <Label label="Tension Target">
              <EditValue
                value={state?.source_spool_state.adaptive_tension_target}
                title="Source Tension Target"
                step={0.01}
                min={0}
                max={1}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_spool_state.adaptive_tension_target
                }
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={setSourceTensionTarget}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Takeup Tension Arm">
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Hard Min">
              <EditValue
                value={state?.takeup_tension_arm_control_state.hard_min_angle}
                title="Takeup Hard Min"
                unit="deg"
                step={1}
                min={-45}
                max={120}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.takeup_tension_arm_control_state.hard_min_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setTakeupTensionArmControl("hard_min_angle", value)
                }
              />
            </Label>
            <Label label="Hard Max">
              <EditValue
                value={state?.takeup_tension_arm_control_state.hard_max_angle}
                title="Takeup Hard Max"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.takeup_tension_arm_control_state.hard_max_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setTakeupTensionArmControl("hard_max_angle", value)
                }
              />
            </Label>
            <Label label="Start Min">
              <EditValue
                value={state?.takeup_tension_arm_control_state.start_min_angle}
                title="Takeup Start Min"
                unit="deg"
                step={1}
                min={-45}
                max={120}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.takeup_tension_arm_control_state.start_min_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setTakeupTensionArmControl("start_min_angle", value)
                }
              />
            </Label>
            <Label label="Start Max">
              <EditValue
                value={state?.takeup_tension_arm_control_state.start_max_angle}
                title="Takeup Start Max"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.takeup_tension_arm_control_state.start_max_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setTakeupTensionArmControl("start_max_angle", value)
                }
              />
            </Label>
            <Label label="Target">
              <EditValue
                value={state?.takeup_tension_arm_control_state.target_angle}
                title="Takeup Target"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.takeup_tension_arm_control_state.target_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setTakeupTensionArmControl("target_angle", value)
                }
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Source Tension Arm">
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Hard Min">
              <EditValue
                value={state?.source_tension_arm_control_state.hard_min_angle}
                title="Source Hard Min"
                unit="deg"
                step={1}
                min={-45}
                max={120}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_tension_arm_control_state.hard_min_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setSourceTensionArmControl("hard_min_angle", value)
                }
              />
            </Label>
            <Label label="Hard Max">
              <EditValue
                value={state?.source_tension_arm_control_state.hard_max_angle}
                title="Source Hard Max"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_tension_arm_control_state.hard_max_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setSourceTensionArmControl("hard_max_angle", value)
                }
              />
            </Label>
            <Label label="Start Min">
              <EditValue
                value={state?.source_tension_arm_control_state.start_min_angle}
                title="Source Start Min"
                unit="deg"
                step={1}
                min={-45}
                max={120}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_tension_arm_control_state.start_min_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setSourceTensionArmControl("start_min_angle", value)
                }
              />
            </Label>
            <Label label="Start Max">
              <EditValue
                value={state?.source_tension_arm_control_state.start_max_angle}
                title="Source Start Max"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_tension_arm_control_state.start_max_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setSourceTensionArmControl("start_max_angle", value)
                }
              />
            </Label>
            <Label label="Target">
              <EditValue
                value={state?.source_tension_arm_control_state.target_angle}
                title="Source Target"
                unit="deg"
                step={1}
                min={-45}
                max={135}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_tension_arm_control_state.target_angle
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) =>
                  setSourceTensionArmControl("target_angle", value)
                }
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Prepare">
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Tolerance">
              <EditValue
                value={state?.prepare_control_state.tolerance_angle}
                title="Prepare Tolerance"
                unit="deg"
                step={0.5}
                min={1}
                max={20}
                disabled={prepareSettingsDisabled}
                defaultValue={
                  defaultState?.prepare_control_state.tolerance_angle
                }
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={(value) =>
                  setPrepareControl("tolerance_angle", value)
                }
              />
            </Label>
            <Label label="Settle Rate">
              <EditValue
                value={state?.prepare_control_state.settle_rate}
                title="Prepare Settle Rate"
                unit="deg/s"
                step={0.5}
                min={0.1}
                max={30}
                disabled={prepareSettingsDisabled}
                defaultValue={defaultState?.prepare_control_state.settle_rate}
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={(value) => setPrepareControl("settle_rate", value)}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Traverse">
          <Label label="Step Size">
            <EditValue
              value={state?.traverse_state.step_size}
              title="Step Size"
              unit="mm"
              step={0.05}
              min={0.1}
              max={75}
              disabled={settingsDisabled}
              defaultValue={defaultState?.traverse_state.step_size}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setTraverseStepSize}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={state?.traverse_state.padding}
              title="Padding"
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              disabled={settingsDisabled}
              defaultValue={defaultState?.traverse_state.padding}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setTraversePadding}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
