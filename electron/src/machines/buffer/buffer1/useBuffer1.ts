import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { buffer1 } from "@/machines/properties";
import { MachineIdentificationUnique } from "@/machines/types";
import { buffer1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useMemo } from "react";

function useBuffer(machine_identification_unique: MachineIdentificationUnique) {
  const schemaGoUp = z.literal("BufferGoUp");
  const { request: requestGoUp } = useMachineMutation(schemaGoUp);
  const bufferGoUp = async () => {
    requestGoUp({
      machine_identification_unique,
      data: "BufferGoUp",
    });
  };

  const schemaGoDown = z.literal("BufferGoDown");
  const { request: requestGoDown } = useMachineMutation(schemaGoDown);
  const bufferGoDown = async () => {
    requestGoDown({
      machine_identification_unique,
      data: "BufferGoDown",
    });
  };

  return {
    bufferGoDown,
    bufferGoUp,
  };
}

export function useBuffer1() {
  const { serial: serialString } = buffer1SerialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification: MachineIdentificationUnique = useMemo(() => {
    const serial = parseInt(serialString); // Use 0 as fallback if NaN

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
      machine_identification: buffer1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const buffer = useBuffer(machineIdentification);

  return {
    ...buffer,
  };
}
