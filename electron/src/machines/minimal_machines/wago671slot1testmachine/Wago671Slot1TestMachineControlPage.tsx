import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { TouchButton } from "@/components/touch/TouchButton";
import { useWago671Slot1TestMachine } from "./useWago671Slot1TestMachine";

function hex(value: number | undefined) {
  if (value === undefined) return "--";
  return `0x${value.toString(16).toUpperCase().padStart(2, "0")}`;
}

export function Wago671Slot1TestMachineControlPage() {
  const { state, setTargetSpeed, setEnabled, setFreq, setAccFreq } =
    useWago671Slot1TestMachine();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Stepper Speed">
          <Label label="Target Speed">
            <EditValue
              value={state?.target_speed}
              title="Target Speed"
              defaultValue={0}
              min={-25000}
              max={25000}
              step={1}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTargetSpeed}
            />
            <TouchButton
              variant="outline"
              icon="lu:CirclePower"
              onClick={() => setEnabled(true)}
              disabled={false}
              isLoading={false}
            >
              Enable
            </TouchButton>
            <TouchButton
              variant="outline"
              icon="lu:CirclePower"
              onClick={() => setEnabled(false)}
              disabled={false}
              isLoading={false}
            >
              Disable
            </TouchButton>
          </Label>
        </ControlCard>
        <ControlCard height={2} title="Frequency Prescaler">
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(3)}
            disabled={false}
            isLoading={false}
          >
            High (11)
          </TouchButton>
        </ControlCard>
        <ControlCard height={2} title="Acceleration Factor">
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(3)}
            disabled={false}
            isLoading={false}
          >
            High (11)
          </TouchButton>
        </ControlCard>
        <ControlCard title="Live PDO">
          <Label label="Actual velocity">
            {state?.actual_velocity ?? "--"}
          </Label>
          <Label label="Actual steps/s">
            {state
              ? roundToDecimals(state.actual_speed_steps_per_second, 2)
              : "--"}
          </Label>
          <Label label="Raw position">{state?.raw_position ?? "--"}</Label>
          <Label label="Control">
            {hex(state?.control_byte1)} {hex(state?.control_byte2)}{" "}
            {hex(state?.control_byte3)}
          </Label>
          <Label label="Status">
            {hex(state?.status_byte1)} {hex(state?.status_byte2)}{" "}
            {hex(state?.status_byte3)}
          </Label>
          <Label label="Ack">
            speed={String(state?.speed_mode_ack ?? false)} start=
            {String(state?.start_ack ?? false)}
          </Label>
          <Label label="DI">
            di1={String(state?.di1 ?? false)} di2={String(state?.di2 ?? false)}
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
