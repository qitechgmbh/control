import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { ip20TestMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useIP20TestMachineNamespace,
  StateEvent,
  LiveValuesEvent,
} from "./ip20TestMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { ip20TestMachine } from "@/machines/properties";
import { z } from "zod";

export function useIP20TestMachine() {
  const { serial: serialString } = ip20TestMachineSerialRoute.useParams();

  // Memoize machine identification
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
      machine_identification: ip20TestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state, liveValues } = useIP20TestMachineNamespace(
    machineIdentification,
  );

  // Optimistic state
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  // Generic mutation sender
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
        current.outputs = [on, on, on, on, on, on, on, on];
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
    liveValues,
    setOutput,
    setAllOutputs,
  };
}
