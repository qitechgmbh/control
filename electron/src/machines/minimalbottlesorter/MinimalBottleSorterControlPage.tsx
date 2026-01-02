import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Button } from "@/components/ui/button";
import { EditValue } from "@/control/EditValue";
import { Badge } from "@/components/ui/badge";
import { useMinimalBottleSorter } from "./useMinimalBottleSorter";
import React from "react";

export function MinimalBottleSorterControlPage() {
  const {
    state,
    liveValues,
    setStepperSpeed,
    setStepperDirection,
    setStepperEnabled,
    pulseOutput,
  } = useMinimalBottleSorter();

  const safeState = state ?? {
    stepper_enabled: false,
    stepper_speed: 0,
    stepper_direction: true,
    outputs: [false, false, false, false, false, false, false, false],
  };
  const safeLiveValues = liveValues ?? {
    stepper_actual_speed: 0,
    stepper_position: 0,
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        {/* Stepper Motor Control */}
        <ControlCard title="Stepper Motor">
          <div className="space-y-6">
            {/* Enable/Disable */}
            <Label label="Motor Enable">
              <SelectionGroup<"Enabled" | "Disabled">
                value={safeState.stepper_enabled ? "Enabled" : "Disabled"}
                orientation="vertical"
                className="grid h-full grid-cols-2 gap-2"
                options={{
                  Disabled: {
                    children: "Disabled",
                    icon: "lu:CirclePause",
                    isActiveClassName: "bg-red-600",
                    className: "h-full",
                  },
                  Enabled: {
                    children: "Enabled",
                    icon: "lu:CirclePlay",
                    isActiveClassName: "bg-green-600",
                    className: "h-full",
                  },
                }}
                onChange={(value) => setStepperEnabled(value === "Enabled")}
              />
            </Label>

            {/* Direction */}
            <Label label="Direction">
              <SelectionGroup<"Forward" | "Backward">
                value={safeState.stepper_direction ? "Forward" : "Backward"}
                orientation="vertical"
                className="grid h-full grid-cols-2 gap-2"
                options={{
                  Forward: {
                    children: "Forward",
                    icon: "lu:ArrowRight",
                    isActiveClassName: "bg-blue-600",
                    className: "h-full",
                  },
                  Backward: {
                    children: "Backward",
                    icon: "lu:ArrowLeft",
                    isActiveClassName: "bg-blue-600",
                    className: "h-full",
                  },
                }}
                onChange={(value) => setStepperDirection(value === "Forward")}
              />
            </Label>

            {/* Speed Control */}
            <Label label="Speed (mm/s)">
              <EditValue
                title="Speed"
                value={safeState.stepper_speed}
                onChange={setStepperSpeed}
                renderValue={(v) => v.toFixed(1)}
                min={0}
                max={100}
                step={0.1}
              />
            </Label>

            {/* Live Values */}
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm text-gray-400">Actual Speed:</span>
                <Badge className="bg-blue-600">
                  {safeLiveValues.stepper_actual_speed.toFixed(1)} steps/s
                </Badge>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-gray-400">Position:</span>
                <Badge className="bg-purple-600">
                  {safeLiveValues.stepper_position.toLocaleString()} steps
                </Badge>
              </div>
            </div>
          </div>
        </ControlCard>

        {/* Digital Output Pulses */}
        <ControlCard title="Digital Output Pulses">
          <div className="grid grid-cols-2 gap-4">
            {safeState.outputs.map((output, index) => (
              <div key={index} className="space-y-2">
                <Label label={`Output ${index + 1}`}>
                  <Button
                    onClick={() => pulseOutput(index, 100)}
                    className={`w-full ${output ? "bg-green-600" : "bg-gray-600"}`}
                    disabled={output}
                  >
                    {output ? "Active" : "Pulse"}
                  </Button>
                </Label>
              </div>
            ))}
          </div>
          <div className="mt-4 text-sm text-gray-400">
            Click to pulse output for 100ms
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
