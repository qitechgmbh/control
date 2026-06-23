import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useWago671Slot12TestMachine } from "./useWago671Slot12TestMachine";
import { AxisStateEvent } from "./wago671Slot12TestMachineNamespace";

function hex(value: number | undefined) {
  if (value === undefined) return "--";
  return `0x${value.toString(16).toUpperCase().padStart(2, "0")}`;
}

function AxisCard({
  title,
  axis,
  state,
  setTargetSpeed,
  setEnabled,
  setAcceleration,
}: {
  title: string;
  axis: 1 | 2;
  state: AxisStateEvent | undefined;
  setTargetSpeed: (axis: 1 | 2, target: number) => void;
  setEnabled: (axis: 1 | 2, enabled: boolean) => void;
  setAcceleration: (axis: 1 | 2, acceleration: number) => void;
}) {
  return (
    <ControlCard title={title}>
      <Label label="Target Speed">
        <EditValue
          value={state?.target_speed}
          title={`${title} Target Speed`}
          defaultValue={0}
          min={-25000}
          max={25000}
          step={1}
          renderValue={(value) => roundToDecimals(value, 0)}
          onChange={(value) => setTargetSpeed(axis, value)}
        />
      </Label>
      <Label label="Acceleration">
        <EditValue
          value={state?.acceleration}
          title={`${title} Acceleration`}
          defaultValue={1000}
          min={1}
          max={65535}
          step={1}
          renderValue={(value) => roundToDecimals(value, 0)}
          onChange={(value) => setAcceleration(axis, value)}
        />
      </Label>
      <Label label="Actual velocity">{state?.actual_velocity ?? "--"}</Label>
      <Label label="Actual steps/s">
        {state ? roundToDecimals(state.actual_speed_steps_per_second, 2) : "--"}
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
      <TouchButton
        variant="outline"
        icon="lu:CirclePower"
        onClick={() => setEnabled(axis, true)}
        disabled={false}
        isLoading={false}
      >
        Enable
      </TouchButton>
      <TouchButton
        variant="outline"
        icon="lu:CirclePower"
        onClick={() => setEnabled(axis, false)}
        disabled={false}
        isLoading={false}
      >
        Disable
      </TouchButton>
    </ControlCard>
  );
}

export function Wago671Slot12TestMachineControlPage() {
  const { state, setTargetSpeed, setEnabled, setAcceleration } =
    useWago671Slot12TestMachine();

  return (
    <Page>
      <ControlGrid columns={2}>
        <AxisCard
          title="Slot 1"
          axis={1}
          state={state?.slot1}
          setTargetSpeed={setTargetSpeed}
          setEnabled={setEnabled}
          setAcceleration={setAcceleration}
        />
        <AxisCard
          title="Slot 2"
          axis={2}
          state={state?.slot2}
          setTargetSpeed={setTargetSpeed}
          setEnabled={setEnabled}
          setAcceleration={setAcceleration}
        />
      </ControlGrid>
    </Page>
  );
}
