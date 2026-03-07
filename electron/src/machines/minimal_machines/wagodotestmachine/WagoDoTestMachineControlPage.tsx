import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useWagoDoTestMachine } from "./useWagoDoTestMachine";

export function WagoDoTestMachineControlPage() {
  const { state, setLed, setAllLeds } = useWagoDoTestMachine();

  const safeState = state ?? {
    led_on: Array(8).fill(false),
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Digital Output Controls */}
        <ControlCard title="Digital Outputs (Do1-Do8)">
          <div className="grid grid-cols-2 gap-6">
            {safeState.led_on.map((output, index) => (
              <Label key={index} label={`Do${index + 1}`}>
                <SelectionGroup<"On" | "Off">
                  value={output ? "On" : "Off"}
                  orientation="vertical"
                  className="flex flex-col gap-3"
                  options={{
                    Off: {
                      children: "Off",
                      icon: "lu:CirclePause",
                      isActiveClassName: "bg-red-600",
                      className: "flex-1",
                    },
                    On: {
                      children: "On",
                      icon: "lu:CirclePlay",
                      isActiveClassName: "bg-green-600",
                      className: "flex-1",
                    },
                  }}
                  onChange={(value) => setLed(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Master Output Control */}
        <ControlCard title="Master Output Control">
          <SelectionGroup<"On" | "Off">
            value={safeState.led_on.every(Boolean) ? "On" : "Off"}
            orientation="horizontal"
            options={{
              Off: { children: "Turn All Off" },
              On: { children: "Turn All On" },
            }}
            onChange={(value) => setAllLeds(value === "On")}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
