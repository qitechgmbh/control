import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { useVacuum } from "./use";
import { Spool } from "../winder/Spool";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundToDecimals } from "@/lib/decimal";

export function VacuumControlPage() {
  const { state, liveValues, remaining_time, spin_shitter, setMode } = useVacuum();

  const safeState = state ?? { mode: "Idle", interval_time_off: 0, interval_time_on: 0, running: false };
  
  const safeLiveValues = liveValues ?? { remaining_time: 0 };

  return (
    <Page>
      <ControlGrid columns={2}>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Idle" | "On" | "Auto" | "Interval">
            value={ safeState.mode }
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Idle: {
                children: "Idle",
                icon: "lu:Power",
                isActiveClassName: "bg-red-600",
                className: "h-full",
              },
              On: {
                children: "On",
                icon: "lu:CirclePlay",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Auto: {
                children: "Auto",
                icon: "lu:BrainCog",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Interval: {
                children: "Interval",
                icon: "lu:Activity",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
          />
        </ControlCard>

        <ControlCard title="Motor">
          <Badge className={safeState.running ? "bg-green-500" : "bg-red-500"}>
            {safeState.running ? "Running" : "Idle" }
          </Badge>
          <Spool rpm={spin_shitter.current?.value} />
          <TimeSeriesValueNumeric
            label="Remaining Time"
            unit="s"
            timeseries={remaining_time}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

      </ControlGrid>
    </Page>
  );
}