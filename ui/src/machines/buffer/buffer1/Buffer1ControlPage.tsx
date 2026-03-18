import { Page } from "@ui/components/Page";
import { ControlGrid } from "@ui/control/ControlGrid";
import React from "react";

import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@ui/control/ControlCard";
import { SelectionGroup } from "@ui/control/SelectionGroup";

export function Buffer1ControlPage() {
  const { state, setBufferMode } = useBuffer1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "FillingBuffer" | "EmptyingBuffer">
            value={state?.mode_state.mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              FillingBuffer: {
                children: "FillingBuffer",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              EmptyingBuffer: {
                children: "EmptyingBuffer",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={setBufferMode}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
