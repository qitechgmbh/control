import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { bottlecapsTestMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useBottlecapsTestMachineNamespace,
  StateEvent,
} from "./BottlecapsTestMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { bottlecapsTestMachine } from "@/machines/properties";
import { z } from "zod";

export function useBottlecapsTestMachine() {
  const { serial: serialString } = bottlecapsTestMachineSerialRoute.useParams();

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
      machine_identification: bottlecapsTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useBottlecapsTestMachineNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  const { request: sendMutation } = useMachineMutate(
    z.object({
      action: z.string(),
      value: z.any(),
    }),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest?: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState)
      stateOptimistic.setOptimistic(produce(currentState, producer));
    serverRequest?.();
  };

  const setOverrideInput = (index: number, on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.override_inputs[index] = on;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetOverrideInput", value: { index, on } },
        }),
    );
  };

  const setStepperTargetSpeed = (target: number) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_target_speed = target;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperTargetSpeed", value: { target } },
        }),
    );
  };

  const setStepperEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_enabled = enabled;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperEnabled", value: { enabled } },
        }),
    );
  };

  const setStepperFreq = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_freq = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperFreq", value: { factor } },
        }),
    );
  };

  const setStepperAccFreq = (factor: number) => {
    updateStateOptimistically(
      (current) => {
        current.stepper_acc_freq = factor;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetStepperAccFreq", value: { factor } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setOverrideInput,
    setStepperTargetSpeed,
    setStepperEnabled,
    setStepperFreq,
    setStepperAccFreq,
  };
}
