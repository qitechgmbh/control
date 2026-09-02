import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useTestMachine } from "./useTestMachine";

export function TestMachineControlPage() {
  const { state, setLed, setAllLeds } = useTestMachine();

  const safeState = state ?? { led_on: [false, false, false, false] };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* LED Controls */}
        <ControlCard title="Machine LEDs">
          <div className="grid grid-cols-2 gap-6">
            {safeState.led_on.map((led, index) => (
              <Label key={index} label={`LED ${index + 1}`}>
                <SelectionGroup<"On" | "Off">
                  value={led ? "On" : "Off"}
                  orientation="vertical"
                  className="grid h-full grid-cols-2 gap-2"
                  options={{
                    Off: {
                      children: "Off",
                      icon: "lu:CirclePause",
                      isActiveClassName: "bg-red-600",
                      className: "h-full",
                    },
                    On: {
                      children: "On",
                      icon: "lu:CirclePlay",
                      isActiveClassName: "bg-green-600",
                      className: "h-full",
                    },
                  }}
                  onChange={(value) => setLed(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Master LED Control */}
        <ControlCard title="Master LED Control">
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
