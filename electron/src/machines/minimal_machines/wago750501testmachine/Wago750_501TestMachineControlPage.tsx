import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useWago750_501TestMachine } from "./useWago750_501TestMachine";

const PORT_LABELS = ["Do1", "Do2"] as const;

export function Wago750_501TestMachineControlPage() {
  const { state, setOutput, setAllOutputs } = useWago750_501TestMachine();

  const safeState = state ?? {
    outputs: Array(2).fill(false),
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Digital Outputs (750-501)">
          <div className="grid grid-cols-2 gap-6">
            {safeState.outputs.map((on, index) => (
              <Label key={index} label={PORT_LABELS[index]}>
                <SelectionGroup<"On" | "Off">
                  value={on ? "On" : "Off"}
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

        <ControlCard title="Master Output Control">
          <SelectionGroup<"On" | "Off">
            value={safeState.outputs.every(Boolean) ? "On" : "Off"}
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
