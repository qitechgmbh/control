import { useMachineMutate } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { wagoTraverseTestMachine } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { z } from "zod";
import { wagoTraverseTestMachineSerialRoute } from "@/routes/routes";
import {
  StateEvent,
  useWagoTraverseTestMachineNamespace,
} from "./wagoTraverseTestMachineNamespace";

export function useWagoTraverseTestMachine() {
  const { serial: serialString } =
    wagoTraverseTestMachineSerialRoute.useParams();

  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: { vendor: 0, machine: 0 },
        serial: 0,
      };
    }

    return {
      machine_identification: wagoTraverseTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWagoTraverseTestMachineNamespace(machineIdentification);
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any().optional(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest?.();
  };

  const send = (action: string, value?: unknown) =>
    sendMutation({
      machine_identification_unique: machineIdentification,
      data: value === undefined ? { action } : { action, value },
    });

  return {
    state: stateOptimistic.value,
    setMode: (mode: "Standby" | "Hold") =>
      updateStateOptimistically(
        (current) => {
          current.mode = mode;
          current.enabled = mode === "Hold";
          current.control_mode = "Idle";
        },
        () => send("SetMode", mode),
      ),
    setEnabled: (enabled: boolean) =>
      updateStateOptimistically(
        (current) => {
          current.enabled = enabled;
        },
        () => send("SetEnabled", enabled),
      ),
    setManualSpeedMmPerSecond: (speed: number) =>
      updateStateOptimistically(
        (current) => {
          current.manual_speed_mm_per_second = speed;
          current.control_mode = "ManualMmPerSecond";
        },
        () => send("SetManualSpeedMmPerSecond", speed),
      ),
    setManualVelocityRegister: (velocity: number) =>
      updateStateOptimistically(
        (current) => {
          current.manual_velocity_register = velocity;
          current.control_mode = "ManualVelocityRegister";
        },
        () => send("SetManualVelocityRegister", velocity),
      ),
    jogRawPositive: (velocity = 1000) =>
      send("SetManualVelocityRegister", velocity),
    jogRawNegative: (velocity = -1000) =>
      send("SetManualVelocityRegister", velocity),
    jogRawPositiveFast: (velocity = 4000) =>
      send("SetManualVelocityRegister", velocity),
    jogRawNegativeFast: (velocity = -4000) =>
      send("SetManualVelocityRegister", velocity),
    jogMmPositive: (speed = 20) => send("SetManualSpeedMmPerSecond", speed),
    jogMmNegative: (speed = -20) => send("SetManualSpeedMmPerSecond", speed),
    jogMmPositiveFast: (speed = 80) => send("SetManualSpeedMmPerSecond", speed),
    jogMmNegativeFast: (speed = -80) =>
      send("SetManualSpeedMmPerSecond", speed),
    stop: () =>
      updateStateOptimistically(
        (current) => {
          current.control_mode = "Idle";
        },
        () => send("Stop"),
      ),
    setSwitchOutput: (on: boolean) =>
      updateStateOptimistically(
        (current) => {
          current.switch_output_on = on;
          current.di2 = on;
        },
        () => send("SetSwitchOutput", on),
      ),
    gotoHome: () => send("GotoHome"),
    gotoLimitInner: () => send("GotoLimitInner"),
    gotoLimitOuter: () => send("GotoLimitOuter"),
    forceNotHomed: () =>
      updateStateOptimistically(
        (current) => {
          current.control_mode = "Idle";
          current.controller_state = "NotHomed";
          current.is_homed = false;
          current.manual_speed_mm_per_second = 0;
          current.manual_velocity_register = 0;
        },
        () => send("ForceNotHomed"),
      ),
    setPositionMm: (position: number) => send("SetPositionMm", position),
    setLimitInnerMm: (limit: number) => send("SetLimitInnerMm", limit),
    setLimitOuterMm: (limit: number) => send("SetLimitOuterMm", limit),
    setSpeedScale: (scale: number) => send("SetSpeedScale", scale),
    setDirectionMultiplier: (direction: number) =>
      send("SetDirectionMultiplier", direction),
    setFreqRangeSel: (value: number) => send("SetFreqRangeSel", value),
    setAccRangeSel: (value: number) => send("SetAccRangeSel", value),
    setAcceleration: (value: number) => send("SetAcceleration", value),
  };
}
