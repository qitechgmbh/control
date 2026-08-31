import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { TouchSlider } from "@/components/touch/TouchSlider";
import { SelectionGroup } from "@/control/SelectionGroup";
import React, { useRef, useState } from "react";
import { useWago750_553Machine } from "./useWago750_553Machine";

export function Wago750_553MachineControlPage() {
  const { state, setOutput, setAllOutputs } = useWago750_553Machine();

  const safeState = state ?? {
    outputs: [0, 0, 0, 0],
    outputs_ma: [0, 0, 0, 0],
  };

  // Local slider state — moves freely during drag, independent of backend.
  const [localOutputs, setLocalOutputs] = useState<number[]>([0, 0, 0, 0]);

  // Tracks which channels are currently being dragged.
  const dragging = useRef<boolean[]>([false, false, false, false]);

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Analog Outputs (AO1–AO4)">
          <div className="flex flex-col gap-6">
            {safeState.outputs.map((serverVal, index) => {
              const value = dragging.current[index]
                ? localOutputs[index]
                : serverVal;
              return (
                <Label
                  key={index}
                  label={`AO${index + 1} — ${(value * 100).toFixed(1)} %  ·  ${(value * 20).toFixed(2)} mA`}
                >
                  <TouchSlider
                    value={[value]}
                    min={0}
                    max={1}
                    step={0.001}
                    minLabel="0 mA"
                    maxLabel="20 mA"
                    renderValue={(v) => `${(v * 20).toFixed(2)}`}
                    unit="mA"
                    onValueChange={([v]) => {
                      dragging.current[index] = true;
                      setLocalOutputs((prev) =>
                        prev.map((o, i) => (i === index ? v : o)),
                      );
                    }}
                    onValueCommit={([v]) => {
                      dragging.current[index] = false;
                      setOutput(index, v);
                    }}
                  />
                </Label>
              );
            })}
          </div>
        </ControlCard>

        <ControlCard title="Master Control">
          <div className="flex flex-col gap-6">
            <Label label="Set All Outputs">
              <SelectionGroup<"Zero" | "Full">
                value={
                  safeState.outputs.every((v) => v >= 1.0)
                    ? "Full"
                    : safeState.outputs.every((v) => v <= 0.0)
                      ? "Zero"
                      : "Zero"
                }
                orientation="horizontal"
                options={{
                  Zero: { children: "All 0 %" },
                  Full: { children: "All 100 %" },
                }}
                onChange={(value) =>
                  setAllOutputs(value === "Full" ? 1.0 : 0.0)
                }
              />
            </Label>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
