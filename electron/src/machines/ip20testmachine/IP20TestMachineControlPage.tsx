import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { useIP20TestMachine } from "./useIP20TestMachine";

export function IP20TestMachineControlPage() {
  const { state, liveValues, setOutput, setAllOutputs } = useIP20TestMachine();

  const safeState = state ?? {
    outputs: [false, false, false, false, false, false, false, false],
  };
  const safeLiveValues = liveValues ?? {
    inputs: [false, false, false, false, false, false, false, false],
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Digital Output Controls */}
        <ControlCard title="Digital Outputs">
          <div className="grid grid-cols-2 gap-6">
            {safeState.outputs.map((output, index) => (
              <Label key={index} label={`Output ${index + 1}`}>
                <SelectionGroup<"On" | "Off">
                  value={output ? "On" : "Off"}
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
                  onChange={(value) => setOutput(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Digital Input Display */}
        <ControlCard title="Digital Inputs">
          <div className="grid grid-cols-2 gap-6">
            {safeLiveValues.inputs.map((input, index) => (
              <Label key={index} label={`Input ${index + 1}`}>
                <div className="flex h-full items-center justify-center">
                  <Badge
                    className={`text-md ${input ? "bg-green-600" : "bg-gray-400"}`}
                  >
                    {input ? "HIGH" : "LOW"}
                  </Badge>
                </div>
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Master Output Control */}
        <ControlCard title="Master Output Control">
          <SelectionGroup<"On" | "Off">
            value={safeState.outputs.every(Boolean) ? "On" : "Off"}
            orientation="horizontal"
            options={{
              Off: {
                children: "Turn All Outputs Off",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-red-600",
              },
              On: {
                children: "Turn All Outputs On",
                icon: "lu:CirclePlay",
                isActiveClassName: "bg-green-600",
              },
            }}
            onChange={(value) => setAllOutputs(value === "On")}
          />
        </ControlCard>

        {/* Input Summary */}
        <ControlCard title="Input Summary">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Active Inputs:</span>
              <Badge className="bg-blue-500">
                {safeLiveValues.inputs.filter(Boolean).length} / 8
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Inactive Inputs:</span>
              <Badge className="bg-gray-500">
                {safeLiveValues.inputs.filter((i) => !i).length} / 8
              </Badge>
            </div>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
