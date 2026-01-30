import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { useVacuum } from "./use";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";

export function VacuumConfigPage() {
  const { state, liveValues, setIntervalTimeOff, setIntervalTimeOn } = useVacuum();

  const safeState = state ?? { mode: "Standby", interval_time_off: 0, interval_time_on: 0 };
  
  const safeLiveValues = liveValues ?? {};

  return (
    <Page>
      <ControlGrid columns={2}>

        <ControlCard className="bg-red" title="Interval Settings">
            <Label label="Interval (OFF) duration">
                <EditValue
                value={safeState.interval_time_off}
                defaultValue={30}
                unit="s"
                title="Interval (OFF) duration"
                min={0}
                max={60}
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={setIntervalTimeOff}
                />
            </Label>
            <Label label="Interval (ON) duration">
                <EditValue
                value={safeState.interval_time_on}
                defaultValue={30}
                unit="s"
                title="Interval (ON) duration"
                min={0}
                max={60}
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={setIntervalTimeOn}
                />
            </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
