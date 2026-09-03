import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { wago750460MachineSerialRoute } from "@/routes/routes";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  StateEvent,
  useWago750460MachineNamespace,
} from "./Wago750460MachineNamespace";
import { useEffect, useMemo } from "react";
import { wago750460Machine } from "@/machines/properties";

export function useWago750460Machine() {
  const { serial: serialString } = wago750460MachineSerialRoute.useParams();

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
      machine_identification: wago750460Machine.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWago750460MachineNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) stateOptimistic.setReal(state);
  }, [state, stateOptimistic]);

  return {
    state: stateOptimistic.value,
  };
}
