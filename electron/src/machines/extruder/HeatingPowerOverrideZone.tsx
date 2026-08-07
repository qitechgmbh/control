import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { StatusBadge } from "@/control/StatusBadge";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { HeatingPowerOverrideState } from "./extruder2/extruder2Namespace";

export type HeatingZoneName = "front" | "middle" | "back" | "nozzle";

type Props = {
  title: string;
  zone: HeatingZoneName;
  overrideState?: HeatingPowerOverrideState;
  defaultOverrideState?: HeatingPowerOverrideState;
  onChange: (zone: HeatingZoneName, enabled: boolean, watts: number) => void;
};

/**
 * Debug/test control: pins one heating zone to a fixed output power instead of letting the
 * temperature PID regulate it. The zone runs open-loop while this is enabled, so it can overshoot
 * its target temperature — only the maximum temperature cutoff still limits it.
 */
export function HeatingPowerOverrideZone({
  title,
  zone,
  overrideState,
  defaultOverrideState,
  onChange,
}: Props) {
  const enabled = overrideState?.enabled ?? false;
  const watts = overrideState?.watts ?? 0;
  const maxWatts = overrideState?.max_watts ?? 0;

  return (
    <ControlCard className="bg-red" title={title}>
      <Label label="Fixed Power Output">
        <SelectionGroupBoolean
          value={enabled}
          optionTrue={{ children: "Enabled" }}
          optionFalse={{ children: "Disabled" }}
          onChange={(nextEnabled) => onChange(zone, nextEnabled, watts)}
        />
      </Label>
      <Label label="Power">
        <EditValue
          value={watts}
          defaultValue={defaultOverrideState?.watts ?? 0}
          unit="W"
          title={`${title} Fixed Power`}
          description={`Heating power this zone is driven at while the override is enabled (max ${roundToDecimals(
            maxWatts,
            0,
          )} W)`}
          min={0}
          max={maxWatts}
          step={10}
          renderValue={(value) => roundToDecimals(value, 0)}
          onChange={(nextWatts) => onChange(zone, enabled, nextWatts)}
        />
      </Label>
      {enabled && (
        <StatusBadge variant="error">
          Temperature is not regulated — this zone heats at a fixed power. Watch
          the temperature and stop the machine if it exceeds the maximum.
        </StatusBadge>
      )}
    </ControlCard>
  );
}
