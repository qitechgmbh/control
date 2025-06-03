import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React from "react";
import { useExtruder2 } from "./useExtruder";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";

export function Extruder2SettingsPage() {
  const {
    inverterSetRotation,
    rotationState,
    extruderSetPressureLimit,
    extruderSetPressureLimitIsEnabled,
    pressureLimitState,
    pressureLimitEnabledState,
  } = useExtruder2();
  return (
    <Page>
      <ControlCard className="bg-red" title="Inverter Settings">
        <Label label="Rotation Direction">
          <SelectionGroupBoolean
            value={rotationState}
            optionTrue={{ children: "Forward" }}
            optionFalse={{ children: "Backward" }}
            onChange={inverterSetRotation}
          />
        </Label>
      </ControlCard>

      <ControlCard className="bg-red" title="Extruder Settings">
        <Label label="Nozzle Pressure Limit">
          <EditValue
            value={pressureLimitState}
            defaultValue={0}
            unit="rpm"
            title="Nozzle Pressure Limit"
            min={0}
            max={350}
            renderValue={(value) => roundToDecimals(value, 0)}
            onChange={extruderSetPressureLimit}
          />
        </Label>
        <Label label="Nozzle Pressure Limit Enabled">
          <SelectionGroupBoolean
            value={pressureLimitEnabledState}
            optionTrue={{ children: "Enabled" }}
            optionFalse={{ children: "Disabled" }}
            onChange={extruderSetPressureLimitIsEnabled}
          />
        </Label>
      </ControlCard>
    </Page>
  );
}
