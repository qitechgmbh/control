import { ControlCard } from "@ui/control/ControlCard";
import { Page } from "@ui/components/Page";
import React from "react";
import { Label } from "@ui/control/Label";
import { useDigitalInputTestMachine } from "./useDigitalInputTestMachine";
import { Badge } from "@ui/components/ui/badge";
import { ControlGrid } from "@ui/control/ControlGrid";
import { SelectionGroup } from "@ui/control/SelectionGroup";
import { toastError } from "@ui/components/Toast";

export function DigitalInputTestMachineControlPage() {
  const { state } = useDigitalInputTestMachine();

  const safeState = state ?? { led_on: [false, false, false, false] };

  return (
    <Page>
      <ControlCard title="Machine LEDs">
        <div className="grid grid-cols-2 gap-6">
          {safeState.led_on.map((led, index) => (
            <Label key={index} label={`LED ${index + 1}`}>
              <Badge variant={led ? "outline" : "destructive"}>
                {led ? "On" : "OFF"}
              </Badge>
            </Label>
          ))}
        </div>
      </ControlCard>
    </Page>
  );
}
