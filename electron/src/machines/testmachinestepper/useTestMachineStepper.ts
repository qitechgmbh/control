import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { testMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import { useTestMachineStepperNamespace, StateEvent } from "./testMachineStepperNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { testmachine } from "@/machines/properties";
import { z } from "zod";
export function useTestMachineStepper() {
  const { serial: serialString } = testMachineSerialRoute.useParams();

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
      machine_identification: testmachine.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state } = useTestMachineStepperNamespace(machineIdentification);

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

  return {
    state: stateOptimistic.value,
  };
}
