import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { wago750_501TestMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useWago750_501TestMachineNamespace,
  StateEvent,
} from "./Wago750_501TestMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { wago750_501TestMachine } from "@/machines/properties";
import { z } from "zod";

export function useWago750_501TestMachine() {
  const { serial: serialString } =
    wago750_501TestMachineSerialRoute.useParams();

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
      machine_identification: wago750_501TestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago750_501TestMachineNamespace(machineIdentification);

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

  const setOutput = (index: number, on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.outputs[index] = on;
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetOutput", value: { index, on } },
        }),
    );
  };

  const setAllOutputs = (on: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.outputs = Array(2).fill(on);
      },
      () =>
        sendMutation({
          machine_identification_unique: machineIdentification,
          data: { action: "SetAllOutputs", value: { on } },
        }),
    );
  };

  return {
    state: stateOptimistic.value,
    setOutput,
    setAllOutputs,
  };
}
