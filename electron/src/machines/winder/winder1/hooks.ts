import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/hooks/useClient";
import { useStateOptimistic } from "@/hooks/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { winder1SerialRoute } from "@/routes/routes";
import { z } from "zod";

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

  // parse serialString into a number or log error
  const serial = parseInt(serialString);
  if (isNaN(serial)) {
    toastError("Invalid serial", "Serial must be a number");
  }

  const machine_identifiaction_unique = {
    vendor: 1,
    machine: 1,
    serial,
  };

  const _useLaserpointer = useLaserpointer(machine_identifiaction_unique);
  return {
    ..._useLaserpointer,
  };
}
