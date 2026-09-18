import { useMemo } from "react";
import {
  MachineIdentificationUnique,
  MachineProperties,
} from "@/machines/types";

/// Builds the `MachineIdentificationUnique` every dryer leaf page needs from its route's
/// serial param.
export function useDryerMachineIdentification(
  machineProperties: MachineProperties,
  serialString: string,
): MachineIdentificationUnique {
  return useMemo(
    () => ({
      machine_identification: machineProperties.machine_identification,
      serial: Number(serialString),
    }),
    [machineProperties, serialString],
  );
}
