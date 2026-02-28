import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useWago8chDioTestMachine } from "./useWago8chDioTestMachine";
import { Badge } from "@/components/ui/badge";

export function Wago8chDioTestMachineControlRoute() {
  const { state, setOutputAtIndex } = useWago8chDioTestMachine();

  const safeState = state ?? {
    digital_input: [false, false, false, false, false, false, false, false],
    digital_output: [false, false, false, false, false, false, false, false],
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Digital Inputs */}
        <ControlCard title="Digital Inputs">
          <div className="grid grid-cols-2 gap-4">
            {safeState.digital_input.map((input, index) => (
              <Label key={index} label={`Input ${index + 1}`}>
                <Badge variant={input ? "outline" : "destructive"}>
                  {input ? "ON" : "OFF"}
                </Badge>
              </Label>
            ))}
          </div>
        </ControlCard>

        {/* Digital Outputs */}
        <ControlCard title="Digital Outputs">
          <div className="grid grid-cols-2 gap-4">
            {safeState.digital_output.map((output, index) => (
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
                  onChange={(value) => setOutputAtIndex(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
