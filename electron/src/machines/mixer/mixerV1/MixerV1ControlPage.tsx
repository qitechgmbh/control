import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useMixerV1 } from "./useMixerV1";

export function MixerV1ControlPage() {
  const {
    state,
    defaultState,
    setMixingMotorOn,
    setHopperAEnabled,
    setHopperATargetRpm,
    setHopperAForward,
    setHopperADosingPercent,
    setHopperBEnabled,
    setHopperBTargetRpm,
    setHopperBForward,
    setHopperBDosingPercent,
    setExtruderKgPerRpm,
  } = useMixerV1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Mixing Motor">
          <Label label="Motor">
            <SelectionGroupBoolean
              value={state?.mixing_motor_state.on}
              optionTrue={{ children: "On" }}
              optionFalse={{ children: "Off" }}
              onChange={setMixingMotorOn}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Extruder Link">
          <Label label="kg per RPM">
            <EditValue
              value={state?.extruder_kg_per_rpm}
              defaultValue={defaultState?.extruder_kg_per_rpm}
              title="Extruder kg per RPM"
              min={0}
              step={0.01}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setExtruderKgPerRpm}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Hopper A">
          <Label label="Enabled">
            <SelectionGroupBoolean
              value={state?.hopper_a_state.enabled}
              optionTrue={{ children: "Enabled" }}
              optionFalse={{ children: "Disabled" }}
              onChange={setHopperAEnabled}
            />
          </Label>

          {state?.hopper_a_state.error && (
            <StatusBadge variant="error">Hopper A Error</StatusBadge>
          )}

          <Label label="Direction">
            <SelectionGroupBoolean
              value={state?.hopper_a_state.forward}
              optionTrue={{ children: "Forward" }}
              optionFalse={{ children: "Reverse" }}
              onChange={setHopperAForward}
            />
          </Label>

          <Label label="Target RPM">
            <EditValue
              value={state?.hopper_a_state.target_rpm}
              defaultValue={defaultState?.hopper_a_state.target_rpm}
              unit="rpm"
              title="Hopper A Target RPM"
              min={0}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setHopperATargetRpm}
            />
          </Label>

          <Label label="Dosing Percent">
            <EditValue
              value={state?.hopper_a_state.dosing_percent}
              defaultValue={defaultState?.hopper_a_state.dosing_percent}
              unit="%"
              title="Hopper A Dosing Percent"
              min={0}
              max={100}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setHopperADosingPercent}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Hopper B">
          <Label label="Enabled">
            <SelectionGroupBoolean
              value={state?.hopper_b_state.enabled}
              optionTrue={{ children: "Enabled" }}
              optionFalse={{ children: "Disabled" }}
              onChange={setHopperBEnabled}
            />
          </Label>

          {state?.hopper_b_state.error && (
            <StatusBadge variant="error">Hopper B Error</StatusBadge>
          )}

          <Label label="Direction">
            <SelectionGroupBoolean
              value={state?.hopper_b_state.forward}
              optionTrue={{ children: "Forward" }}
              optionFalse={{ children: "Reverse" }}
              onChange={setHopperBForward}
            />
          </Label>

          <Label label="Target RPM">
            <EditValue
              value={state?.hopper_b_state.target_rpm}
              defaultValue={defaultState?.hopper_b_state.target_rpm}
              unit="rpm"
              title="Hopper B Target RPM"
              min={0}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setHopperBTargetRpm}
            />
          </Label>

          <Label label="Dosing Percent">
            <EditValue
              value={state?.hopper_b_state.dosing_percent}
              defaultValue={defaultState?.hopper_b_state.dosing_percent}
              unit="%"
              title="Hopper B Dosing Percent"
              min={0}
              max={100}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setHopperBDosingPercent}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
