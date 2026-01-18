import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useBottleSorter } from "./useBottleSorter";

export function BottleSorterControlPage() {
  const {
    state,
    liveValues,
    setStepperSpeed,
    setStepperEnabled,
    pulseOutput,
  } = useBottleSorter();

  const safeState = state ?? {
    outputs: [false, false, false, false, false, false, false, false],
    stepper_speed_mm_s: 0,
    stepper_enabled: false,
  };

  const safeLiveValues = liveValues ?? {
    stepper_position: 0,
  };

  const handleSpeedChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value)) {
      setStepperSpeed(value);
    }
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Stepper Motor Control */}
        <ControlCard title="Stepper Motor Control">
          <div className="flex flex-col gap-4">
            <Label label="Speed (mm/s)">
              <Input
                type="number"
                value={safeState.stepper_speed_mm_s}
                onChange={handleSpeedChange}
                step="0.1"
                min="-100"
                max="100"
              />
            </Label>

            <Label label="Direction">
              <SelectionGroup<"Forward" | "Backward">
                value={safeState.stepper_speed_mm_s >= 0 ? "Forward" : "Backward"}
                orientation="horizontal"
                options={{
                  Forward: {
                    children: "Forward",
                    icon: "lu:ArrowRight",
                    isActiveClassName: "bg-blue-600",
                  },
                  Backward: {
                    children: "Backward",
                    icon: "lu:ArrowLeft",
                    isActiveClassName: "bg-purple-600",
                  },
                }}
                onChange={(value) =>
                  setStepperSpeed(
                    value === "Forward"
                      ? Math.abs(safeState.stepper_speed_mm_s)
                      : -Math.abs(safeState.stepper_speed_mm_s),
                  )
                }
              />
            </Label>

            <Label label="Motor State">
              <SelectionGroup<"Enabled" | "Disabled">
                value={safeState.stepper_enabled ? "Enabled" : "Disabled"}
                orientation="horizontal"
                options={{
                  Disabled: {
                    children: "Disabled",
                    icon: "lu:CirclePause",
                    isActiveClassName: "bg-red-600",
                  },
                  Enabled: {
                    children: "Enabled",
                    icon: "lu:CirclePlay",
                    isActiveClassName: "bg-green-600",
                  },
                }}
                onChange={(value) => setStepperEnabled(value === "Enabled")}
              />
            </Label>
          </div>
        </ControlCard>

        {/* Stepper Motor Status */}
        <ControlCard title="Stepper Motor Status">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Current Position:</span>
              <Badge className="bg-blue-500">
                {safeLiveValues.stepper_position.toFixed(2)} mm
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Target Speed:</span>
              <Badge className="bg-purple-500">
                {safeState.stepper_speed_mm_s.toFixed(2)} mm/s
              </Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Motor Status:</span>
              <Badge
                className={
                  safeState.stepper_enabled ? "bg-green-600" : "bg-gray-500"
                }
              >
                {safeState.stepper_enabled ? "ENABLED" : "DISABLED"}
              </Badge>
            </div>
          </div>
        </ControlCard>

        {/* Digital Output Pulse Controls */}
        <ControlCard title="Digital Output Pulse Controls" className="col-span-2">
          <div className="grid grid-cols-4 gap-4">
            {safeState.outputs.map((output, index) => (
              <div key={index} className="flex flex-col gap-2">
                <Label label={`Output ${index + 1}`}>
                  <Button
                    onClick={() => pulseOutput(index)}
                    className={`w-full ${output ? "bg-green-600 hover:bg-green-700" : ""}`}
                    variant={output ? "default" : "outline"}
                  >
                    {output ? "PULSING" : "PULSE"}
                  </Button>
                </Label>
              </div>
            ))}
          </div>
          <div className="mt-4 text-sm text-muted-foreground">
            Click a button to pulse the output for 100ms
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
