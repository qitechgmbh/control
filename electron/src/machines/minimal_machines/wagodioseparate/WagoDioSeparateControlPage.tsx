import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import React from "react";
import { useWagoDioSeparate } from "./useWagoDioSeparate";

export function WagoDioSeparateControlPage() {
  const { state, setLed, setAllLeds } = useWagoDioSeparate();

  const safeState = state ?? {
    inputs: Array(8).fill(false),
    led_on: Array(8).fill(false),
  };

  return (
    <Page>
      {/* Digital Inputs — read only, from 750-430 */}
      <ControlCard title="Digital Inputs (750-430)">
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

      {/* Digital Outputs — controllable, from 750-530 */}
      <ControlCard title="Digital Outputs (750-530)">
        <div className="grid grid-cols-2 gap-4">
          {safeState.led_on.map((on, index) => (
            <Label key={index} label={`Output ${index + 1}`}>
              <Button
                variant={on ? "default" : "outline"}
                onClick={() => setLed(index, !on)}
              >
                {on ? "ON" : "OFF"}
              </Button>
            </Label>
          ))}
        </div>
        <div className="flex gap-2 mt-4">
          <Button onClick={() => setAllLeds(true)}>All ON</Button>
          <Button variant="outline" onClick={() => setAllLeds(false)}>
            All OFF
          </Button>
        </div>
      </ControlCard>
    </Page>
  );
}