import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { wago750430UfmMachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  StateEvent,
  useWago750430UfmMachineNamespace,
} from "./wago750430UfmMachineNamespace";
import { useEffect, useMemo } from "react";
import { wago750430UfmMachine } from "@/machines/properties";

export function useWago750430UfmMachine() {
  const { serial: serialString } = wago750430UfmMachineSerialRoute.useParams();

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
      machine_identification: wago750430UfmMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago750430UfmMachineNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  return {
    state: stateOptimistic.value,
  };
}
