import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { useTestMotor } from "./useTestMotor";

export function TestMotorControlPage() {
  const { state, setMotorOn, setVelocity } = useTestMotor();

  // Fallback, falls state noch null ist
  const safeState = state ?? { motor_enabled: false, motor_velocity: 0 };

  return (
    <Page>
      <ControlGrid columns={2}>

        {/* rundsteuerung */}
        <ControlCard title="Motor Status">
          {/* An/Aus Schalter */}
          <Label label="Power State">
            <SelectionGroupBoolean
              value={safeState.motor_enabled}
              // Icon Mapping für True/False
              optionTrue={{ children: "Enabled", icon: "lu:Play" }}
              optionFalse={{ children: "Disabled", icon: "lu:CirclePause" }}
              onChange={(val) => setMotorOn(val)}
            />
          </Label>
        </ControlCard>

        {/* Geschwindigkeit */}
        <ControlCard title="Settings">
          {/* Velocity Eingabe mit Einheit */}
          <Label label="Target Velocity">
            <EditValue
              title="Velocity"
              value={safeState.motor_velocity}
              unit="rpm"
              min={0}
              max={1000} // Limit
              step={1}
              onChange={(val) => setVelocity(val)}
              // Zeigt den Wert (hier ganze Zahl)
              renderValue={(v) => v.toFixed(0)}
            />
          </Label>
        </ControlCard>

      </ControlGrid>
    </Page>
  );
}