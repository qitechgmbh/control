import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { Label } from "@/control/Label";
import { Badge } from "@/components/ui/badge";
import { useUfmFlowInputMachine } from "./useUfmFlowInputMachine";
import { MiniGraph } from "@/components/graph/MiniGraph";

export function UfmFlowInputMachineControlPage() {
  const { state, flowLph } = useUfmFlowInputMachine();

  const flowLph_value = state?.flow_lph?.toFixed(2) ?? "—";
  const totalLiters = state != null
    ? (state.total_volume_m3 * 1000).toFixed(3)
    : "—";
  const totalPulses = state?.total_pulses ?? "—";
  const error = state?.error ?? false;

  return (
    <Page>
      <ControlCard title="UFM-02-05 Flow Sensor">
        <div className="grid grid-cols-2 gap-6">
          <Label label="Flow Rate">
            <span className="text-2xl font-mono">{flowLph_value} l/h</span>
            {flowLph && (
              <div className="mt-2 h-16 w-full">
                <MiniGraph
                  newData={flowLph}
                  width={200}
                  renderValue={(value) => value.toFixed(1)}
                />
              </div>
            )}
          </Label>
          <Label label="Total Volume">
            <span className="text-2xl font-mono">{totalLiters} l</span>
          </Label>
          <Label label="Total Pulses">
            <span className="text-2xl font-mono">{totalPulses}</span>
          </Label>
          <Label label="Sensor Status">
            <Badge variant={error ? "destructive" : "outline"}>
              {error ? "Error" : "OK"}
            </Badge>
          </Label>
        </div>
      </ControlCard>
    </Page>
  );
}
