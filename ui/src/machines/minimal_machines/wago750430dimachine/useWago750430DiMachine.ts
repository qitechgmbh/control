import { toastError } from "@ui/components/Toast";
import { useStateOptimistic } from "@ui/lib/useStateOptimistic";
import { wago750430DiMachineSerialRoute } from "@ui/routes/routes";
import { MachineIdentificationUnique } from "@ui/machines/types";
import {
  StateEvent,
  useWago750430DiMachineNamespace,
} from "./wago750430DiMachineNamespace";
import { useEffect, useMemo } from "react";
import { wago750430DiMachine } from "@ui/machines/properties";

export function useWago750430DiMachine() {
  const { serial: serialString } = wago750430DiMachineSerialRoute.useParams();

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
      machine_identification: wago750430DiMachine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago750430DiMachineNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  return {
    state: stateOptimistic.value,
  };
}
