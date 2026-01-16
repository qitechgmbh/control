import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { useDigitalInputTestMachine } from "./useDigitalInputTestMachine";
import { Badge } from "@/components/ui/badge";

export function DigitalInputTestMachineControlPage() {
  const { state } = useDigitalInputTestMachine();

  const safeState = state ?? { led_on: [false, false, false, false] };

  return (
  <Page>
    <ControlCard title="Machine LEDs">
              <div className="grid grid-cols-2 gap-6">
                {safeState.led_on.map((led, index) => (
                  <Label key={index} label={`LED ${index + 1}`} 
                  children={<Badge variant={led? "outline":"destructive"}>{led? "On":"OFF"}</Badge>}
                  >
                  </Label>
                ))}
              </div>
            </ControlCard>
  </Page>
  );
}
