import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { winder1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useWinder1Room } from "./winder1Room";
import { useMemo } from "react";

export function useLaserpointer(
  machine_identification_unique: MachineIdentificationUnique,
): {
  laserpointer: boolean | undefined;
  setLaserpointer: (value: boolean) => void;
  laserpointerIsLoading: boolean;
  laserpointerIsDisabled: boolean;
} {
  const state = useStateOptimistic<boolean>();
  const schema = z.object({ laserpointer: z.boolean() });
  const { request } = useMachineMutation(schema);

  return {
    laserpointer: state.value,
    setLaserpointer: async (value) => {
      state.setOptimistic(value);
      request({
        machine_identification_unique,
        data: { laserpointer: value },
      })
        .then((response) => {
          if (!response.success) state.resetToReal();
        })
        .catch(() => state.resetToReal());
    },
    // TODO read path
    // laserpointerIsLoading: state.isOptimistic || !state.isInitialized,
    // laserpointerIsDisabled: state.isOptimistic || !state.isInitialized,
    laserpointerIsLoading: false,
    laserpointerIsDisabled: false,
  };
}

export function useWinder1() {
  const { serial: serialString } = winder1SerialRoute.useParams();

  // Memoize the machine identification to keep it stable between renders
  const machineIdentification = useMemo(() => {
    const serial = parseInt(serialString); // Use 0 as fallback if NaN

    if (isNaN(serial)) {
      toastError(
        "Invalid Serial Number",
        `"${serialString}" is not a valid serial number.`,
      );

      return {
        vendor: 0,
        machine: 0,
        serial: 0,
      };
    }

    return {
      vendor: 1,
      machine: 1,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const room = useWinder1Room(machineIdentification);
  const laserpointerControls = useLaserpointer(machineIdentification);

  return {
    ...laserpointerControls,
    ...room,
  };
}
