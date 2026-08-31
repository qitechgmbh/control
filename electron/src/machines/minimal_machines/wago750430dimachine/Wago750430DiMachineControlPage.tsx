import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import React from "react";
import { useWago750430DiMachine } from "./useWago750430DiMachine";

export function Wago750430DiMachineControlPage() {
  const { state } = useWago750430DiMachine();

  const safeState = state ?? {
    inputs: [false, false, false, false, false, false, false, false],
  };

  return (
    <Page>
      <ControlCard title="Digital Inputs">
        <div className="grid grid-cols-2 gap-4">
          {safeState.inputs.map((input, index) => (
            <Label key={index} label={`Input ${index + 1}`}>
              <Badge variant={input ? "outline" : "destructive"}>
                {input ? "ON" : "OFF"}
              </Badge>
            </Label>
          ))}
        </div>
      </ControlCard>
    </Page>
  );
}
