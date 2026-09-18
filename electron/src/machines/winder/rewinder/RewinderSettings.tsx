import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useRewinder } from "./useRewinder";

const DEFAULT_SPOOL_DIAMETER_MM = 100;
const MIN_SPOOL_DIAMETER_MM = 10;
const MAX_SPOOL_DIAMETER_MM = 500;

const MIN_ARM_MIN_ANGLE_DEG = -45;
const MAX_ARM_MIN_ANGLE_DEG = 120;
const MIN_ARM_MAX_ANGLE_DEG = -45;
const MAX_ARM_MAX_ANGLE_DEG = 135;

const MIN_PREPARE_TOLERANCE_DEG = 1;
const MAX_PREPARE_TOLERANCE_DEG = 20;
const MIN_PREPARE_RATE_DEG_PER_S = 0.1;
const MAX_PREPARE_RATE_DEG_PER_S = 30;

export function RewinderSettingsPage() {
  const {
    state,
    defaultState,
    isDisabled,
    isLoading,
    settingsEditPermitted,
    setTraverseStepSize,
    setTraversePadding,
    setTraverseStart,
    setTraverseStartPosition,
    setTakeupSpoolDiameter,
    setSourceSpoolDiameter,
    setTakeupTensionArmControl,
    setSourceTensionArmControl,
    setPrepareControl,
  } = useRewinder();
  const settingsDisabled = isDisabled || isLoading || !settingsEditPermitted;

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Takeup Spool">
          <Label label="Diameter">
            <EditValue
              value={
                state?.takeup_spool_state.diameter_mm ??
                DEFAULT_SPOOL_DIAMETER_MM
              }
              title="Takeup Spool Diameter"
              unit="mm"
              step={1}
              min={MIN_SPOOL_DIAMETER_MM}
              max={MAX_SPOOL_DIAMETER_MM}
              disabled={settingsDisabled}
              defaultValue={
                defaultState?.takeup_spool_state.diameter_mm ??
                DEFAULT_SPOOL_DIAMETER_MM
              }
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTakeupSpoolDiameter}
            />
            {state?.takeup_spool_state.diameter_mm == null ? (
              <span className="text-sm text-amber-600">
                Not set yet. Confirm for better takeup feed-forward.
              </span>
            ) : null}
          </Label>
        </ControlCard>

        <ControlCard title="Source Spool">
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Diameter">
              <EditValue
                value={
                  state?.source_spool_state.diameter_mm ??
                  DEFAULT_SPOOL_DIAMETER_MM
                }
                title="Source Spool Diameter"
                unit="mm"
                step={1}
                min={MIN_SPOOL_DIAMETER_MM}
                max={MAX_SPOOL_DIAMETER_MM}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.source_spool_state.diameter_mm ??
                  DEFAULT_SPOOL_DIAMETER_MM
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
                min={MIN_ARM_MIN_ANGLE_DEG}
                max={MAX_ARM_MIN_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_ARM_MIN_ANGLE_DEG}
                max={MAX_ARM_MIN_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_ARM_MIN_ANGLE_DEG}
                max={MAX_ARM_MIN_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_ARM_MIN_ANGLE_DEG}
                max={MAX_ARM_MIN_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_ARM_MAX_ANGLE_DEG}
                max={MAX_ARM_MAX_ANGLE_DEG}
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
                min={MIN_PREPARE_TOLERANCE_DEG}
                max={MAX_PREPARE_TOLERANCE_DEG}
                disabled={settingsDisabled}
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
                min={MIN_PREPARE_RATE_DEG_PER_S}
                max={MAX_PREPARE_RATE_DEG_PER_S}
                disabled={settingsDisabled}
                defaultValue={defaultState?.prepare_control_state.settle_rate}
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={(value) => setPrepareControl("settle_rate", value)}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Traverse">
          <Label label="Start Side">
            <SelectionGroup
              value={state?.traverse_state.start}
              disabled={settingsDisabled}
              loading={isLoading}
              options={{
                Left: { children: "Left Side", icon: "lu:ArrowLeftToLine" },
                Right: {
                  children: "Right Side",
                  icon: "lu:ArrowRightToLine",
                },
                Custom: { children: "Custom", icon: "lu:MapPin" },
              }}
              onChange={setTraverseStart}
            />
          </Label>
          {state?.traverse_state.start === "Custom" ? (
            <Label label="Custom Start">
              <EditValue
                value={state.traverse_state.custom_start_position}
                title="Custom Start Position"
                unit="mm"
                step={1}
                min={state.traverse_state.limit_inner}
                max={state.traverse_state.limit_outer}
                disabled={settingsDisabled}
                defaultValue={
                  defaultState?.traverse_state.custom_start_position
                }
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setTraverseStartPosition}
              />
            </Label>
          ) : null}
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
