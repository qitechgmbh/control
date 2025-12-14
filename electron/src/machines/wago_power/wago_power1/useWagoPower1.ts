import { toastError } from "@/components/Toast";
import { wagoPower1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { wagoPower1SerialRoute } from "@/routes/routes";
import { useMemo } from "react";
import { useWagoPower1Namespace } from "./wagoPower1Namespace";

export function useWagoPower1() {
  const { serial: serialString } = wagoPower1SerialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString);

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        machine_identification: {
          vendor: 0,
          machine: 0,
        },
        serial: 0,
      };
    }

    return {
      machine_identification: wagoPower1.machine_identification,
      serial,
    };
  }, [serialString]);

  const { state } = useWagoPower1Namespace(machineIdentification);
  return { state };
}
