import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";

import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";

export function Buffer1ControlPage() {
  const { state, defaultState, setBufferMode, setCurrentInputSpeed, pullerSpeed, setPullerRegulationMode, setPullerTargetSpeed, isLoading, isDisabled } =
    useBuffer1();

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

        <ControlCard className="bg-red" title="Puller">
          <TimeSeriesValueNumeric
            label="Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          <Label label="Regulation">
            <SelectionGroup
              value={state?.puller_state?.regulation}
              options={{
                Speed: {
                  children: "Speed",
                  icon: "lu:Gauge",
                },
                Diameter: {
                  children: "Diameter",
                  icon: "lu:Sun",
                  disabled: true,
                },
              }}
              onChange={setPullerRegulationMode}
              disabled={isDisabled}
              loading={isLoading}
            />
          </Label>
          <Label label="Target Speed">
            <EditValue
              value={state?.puller_state?.target_speed}
              unit="m/min"
              title="Target Speed"
              defaultValue={defaultState?.puller_state?.target_speed}
              min={0}
              max={75}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setPullerTargetSpeed}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
