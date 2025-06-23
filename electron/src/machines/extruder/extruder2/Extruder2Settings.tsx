import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React from "react";
import { useExtruder2 } from "./useExtruder";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { TouchButton } from "@/components/touch/TouchButton";

export function Extruder2SettingsPage() {
  const {
    inverterSetRotation,
    inverterReset,
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

        <Label label="Reset Inverter">
          <button
            onClick={inverterReset}
            className="inline-block w-fit max-w-max rounded bg-red-600 px-4 py-4 text-base whitespace-nowrap text-white hover:bg-red-700"
            style={{ minWidth: "auto", width: "fit-content" }}
          >
            Reset Inverter
          </button>
        </Label>
      </ControlCard>

      <ControlCard className="bg-red" title="Extruder Settings">
        <Label label="Nozzle Pressure Limit">
          <EditValue
            value={pressureLimitState}
            defaultValue={0}
            unit="bar"
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
