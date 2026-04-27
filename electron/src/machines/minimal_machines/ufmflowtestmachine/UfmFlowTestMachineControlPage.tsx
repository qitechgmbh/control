import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { Page } from "@/components/Page";
import { Badge } from "@/components/ui/badge";
import React from "react";
import { useUfmFlowTestMachine } from "./useUfmFlowTestMachine";

export function UfmFlowTestMachineControlPage() {
  const { state } = useUfmFlowTestMachine();

  const safeState = state ?? {
    flow_lph: 0,
    total_volume_m3: 0,
    sensor_error: false,
  };

  return (
    <Page>
      <ControlCard title="UFM Flow Sensor">
        <div className="grid grid-cols-1 gap-4">
          <Label label="Measured flow">
            <div className="text-2xl font-semibold">
              {safeState.flow_lph.toFixed(2)} lph
            </div>
          </Label>
          <Label label="Sensor error (DI2)">
            <Badge variant={safeState.sensor_error ? "destructive" : "outline"}>
              {safeState.sensor_error ? "ERROR" : "OK"}
            </Badge>
          </Label>
        </div>
      </ControlCard>
    </Page>
  );
}
