import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useWagoWinderSmokeTestMachine } from "./useWagoWinderSmokeTestMachine";

const defaultState = {
  axes: [
    {
      enabled: false,
      target_velocity: 0,
      target_acceleration: 10000,
      freq_range_sel: 0,
      acc_range_sel: 0,
      mode: null,
      speed_mode_ack: false,
      di1: false,
      di2: false,
      status_byte1: 0,
      status_byte2: 0,
      status_byte3: 0,
    },
  ],
  digital_output1: false,
  digital_output2: false,
};

export function WagoWinderSmokeTestMachineControlPage() {
  const {
    state,
    setStepperEnabled,
    setStepperVelocity,
    setStepperFreqRange,
    setStepperAccRange,
    setDigitalOutput,
  } = useWagoWinderSmokeTestMachine();

  const safeState = state ?? defaultState;

  return (
    <Page>
      <ControlGrid columns={2}>
        {safeState.axes.map((axis, index) => (
          <ControlCard key={index} title="671 Stepper">
            <div className="space-y-4">
              <Label label="Enable">
                <SelectionGroup<"Enabled" | "Disabled">
                  value={axis.enabled ? "Enabled" : "Disabled"}
                  orientation="horizontal"
                  options={{
                    Disabled: { children: "Disabled" },
                    Enabled: { children: "Enabled" },
                  }}
                  onChange={(value) =>
                    setStepperEnabled(index, value === "Enabled")
                  }
                />
              </Label>

              <Label label="Velocity">
                <EditValue
                  value={axis.target_velocity}
                  title={`Axis ${index + 1} Velocity`}
                  defaultValue={0}
                  min={-25000}
                  max={25000}
                  step={1}
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setStepperVelocity(index, value)}
                />
              </Label>

              <Label label="Frequency Range">
                <SelectionGroup<"0" | "1" | "2" | "3">
                  value={String(axis.freq_range_sel) as "0" | "1" | "2" | "3"}
                  orientation="horizontal"
                  options={{
                    "0": { children: "0" },
                    "1": { children: "1" },
                    "2": { children: "2" },
                    "3": { children: "3" },
                  }}
                  onChange={(value) =>
                    setStepperFreqRange(index, Number(value))
                  }
                />
              </Label>

              <Label label="Acceleration Range">
                <SelectionGroup<"0" | "1" | "2" | "3">
                  value={String(axis.acc_range_sel) as "0" | "1" | "2" | "3"}
                  orientation="horizontal"
                  options={{
                    "0": { children: "0" },
                    "1": { children: "1" },
                    "2": { children: "2" },
                    "3": { children: "3" },
                  }}
                  onChange={(value) =>
                    setStepperAccRange(index, Number(value))
                  }
                />
              </Label>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>Mode: {axis.mode ?? "None"}</div>
                <div
                  className={`rounded px-3 py-2 text-white ${axis.speed_mode_ack ? "bg-green-600" : "bg-red-600"}`}
                >
                  S1.3 Speed Ack: {axis.speed_mode_ack ? "On" : "Off"}
                </div>
                <div className={`rounded px-3 py-2 text-white ${axis.di1 ? "bg-green-600" : "bg-red-600"}`}>
                  S3.0: {axis.di1 ? "On" : "Off"}
                </div>
                <div>DI2: {axis.di2 ? "On" : "Off"}</div>
                <div>S1: 0x{axis.status_byte1.toString(16).padStart(2, "0")}</div>
                <div>S2: 0x{axis.status_byte2.toString(16).padStart(2, "0")}</div>
                <div>S3: 0x{axis.status_byte3.toString(16).padStart(2, "0")}</div>
              </div>
            </div>
          </ControlCard>
        ))}

        <ControlCard title="750-501 Digital Outputs">
          <div className="grid grid-cols-2 gap-6">
            {[
              { label: "DO1", value: safeState.digital_output1, port: 1 },
              { label: "DO2", value: safeState.digital_output2, port: 2 },
            ].map((output) => (
              <Label key={output.port} label={output.label}>
                <SelectionGroup<"On" | "Off">
                  value={output.value ? "On" : "Off"}
                  orientation="vertical"
                  className="flex flex-col gap-3"
                  options={{
                    Off: {
                      children: "Off",
                      isActiveClassName: "bg-red-600",
                      className: "flex-1",
                    },
                    On: {
                      children: "On",
                      isActiveClassName: "bg-green-600",
                      className: "flex-1",
                    },
                  }}
                  onChange={(value) => setDigitalOutput(output.port, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
