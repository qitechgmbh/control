import { ControlCard } from "@ui/control/ControlCard";
import { Page } from "@ui/components/Page";
import React from "react";
import { ControlGrid } from "@ui/control/ControlGrid";
import { SelectionGroupBoolean } from "@ui/control/SelectionGroup";
import { EditValue } from "@ui/control/EditValue";
import { Label } from "@ui/control/Label";
import { useTestMotor } from "./useTestMotor";

export function TestMotorControlPage() {
  const { state, setMotorOn, setVelocity } = useTestMotor();

  // Fallback in case state is still null
  const safeState = state ?? { motor_enabled: false, motor_velocity: 0 };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Basic control */}
        <ControlCard title="Motor Status">
          {/* On/Off switch */}
          <Label label="Power State">
            <SelectionGroupBoolean
              value={safeState.motor_enabled}
              // Icon mapping for True/False
              optionTrue={{ children: "Enabled", icon: "lu:Play" }}
              optionFalse={{ children: "Disabled", icon: "lu:CirclePause" }}
              onChange={(val) => setMotorOn(val)}
            />
          </Label>
        </ControlCard>

        {/* Velocity */}
        <ControlCard title="Settings">
          {/* Velocity input with unit */}
          <Label label="Target Velocity">
            <EditValue
              title="Velocity"
              value={safeState.motor_velocity}
              unit="rpm"
              min={0}
              max={1000} // Limit
              step={1}
              onChange={(val) => setVelocity(val)}
              // Displays the value (as integer)
              renderValue={(v) => v.toFixed(0)}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
