import { toastError } from "@/components/Toast";
import { wago750467Machine } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { wago750467MachineSerialRoute } from "@/routes/routes";
import { useMemo } from "react";
import { useWago750467MachineNamespace } from "./Wago750467MachineNamespace";

export function useWago750467Machine() {
  const { serial: serialString } = wago750467MachineSerialRoute.useParams();

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
      machine_identification: wago750467Machine.machine_identification,
      serial,
    };
  }, [serialString]);

  return useWago750467MachineNamespace(machineIdentification);
}
