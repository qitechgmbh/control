import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { useVacuum } from "./use";

export function VacuumControlPage() {
  const { state, liveValues, setMode } = useVacuum();

  const safeState = state ?? { running: false };
  
  const safeLiveValues = liveValues ?? {};

  return (
    <Page>
      <ControlGrid columns={2}>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Run" | "Auto" | "Interval">
            value={ safeState.mode }
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Wind: {
                children: "Wind",
                icon: "lu:RefreshCcw",
                isActiveClassName: "bg-green-600",
                disabled: !state?.mode_state?.can_wind,
                className: "h-full",
              },
            }}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
