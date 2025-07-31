import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";

import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";

export function Buffer1ControlPage() {
  const { state, setBufferMode, setCurrentInputSpeed } = useBuffer1();

  const current_input_speed =
    state?.current_input_speed_state.current_input_speed ?? 0.0;
  const default_speed = 0.0;

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
        <ControlCard title="Set Current Input Speed">
          <EditValue
            title="Input Speed"
            unit="m/min"
            value={current_input_speed}
            defaultValue={default_speed}
            min={0.0}
            max={60.0}
            step={0.1}
            renderValue={(value) => value.toFixed(0.0)}
            onChange={setCurrentInputSpeed}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
