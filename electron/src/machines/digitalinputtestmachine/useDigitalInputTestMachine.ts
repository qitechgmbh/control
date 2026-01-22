import { Toast, toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { digitalInputTestMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  useDigitalInputTestMachineNamespace,
  StateEvent,
} from "./digitalInputTestMachineNamespace";
import { useMachineMutate } from "@/client/useClient";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { digitalInputTestMachine } from "@/machines/properties";
import { z } from "zod";
export function useDigitalInputTestMachine() {
  const { serial: serialString } =
    digitalInputTestMachineSerialRoute.useParams();

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
      machine_identification: digitalInputTestMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  // Namespace state from backend
  const { state } = useDigitalInputTestMachineNamespace(machineIdentification);

  // Optimistic state
  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  return {
    state: stateOptimistic.value,
  };
}
