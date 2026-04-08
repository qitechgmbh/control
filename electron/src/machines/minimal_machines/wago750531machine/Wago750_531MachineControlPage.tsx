import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useWago750_531Machine } from "./useWago750_531Machine";

export function Wago750_531MachineControlPage() {
  const { state, setOutput, setAllOutputs } = useWago750_531Machine();

  const safeState = state ?? {
    outputs_on: Array(4).fill(false),
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Individual Output Controls */}
        <ControlCard title="Digital Outputs (DO1–DO4)">
          <div className="grid grid-cols-2 gap-6">
            {safeState.outputs_on.map((output, index) => (
              <Label key={index} label={`DO${index + 1}`}>
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
                  onChange={(value) => setOutput(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Master Output Control */}
        <ControlCard title="Master Output Control">
          <SelectionGroup<"On" | "Off">
            value={
              safeState.outputs_on.every(Boolean)
                ? "On"
                : safeState.outputs_on.every((v) => !v)
                  ? "Off"
                  : undefined
            }
            orientation="horizontal"
            options={{
              Off: { children: "Turn All Off" },
              On: { children: "Turn All On" },
            }}
            onChange={(value) => setAllOutputs(value === "On")}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
