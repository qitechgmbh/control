import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { winder2SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { Mode, useWinder1Namespace } from "./winder2Namespace";
import { useEffect, useMemo, useState } from "react";

function useLaserpointer(
  machine_identification_unique: MachineIdentificationUnique,
): {
  laserpointer: boolean | undefined;
  setLaserpointer: (value: boolean) => void;
  laserpointerIsLoading: boolean;
  laserpointerIsDisabled: boolean;
} {
  const state = useStateOptimistic<boolean>();

  // Write path
  const schema = z.object({ TraverseEnableLaserpointer: z.boolean() });
  const { request } = useMachineMutation(schema);
  const setLaserpointer = async (value: boolean) => {
    state.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { TraverseEnableLaserpointer: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  // Read path
  const { traverseState } = useWinder1Namespace(machine_identification_unique);
  useEffect(() => {
    if (traverseState?.content.Data) {
      state.setReal(traverseState.content.Data.laserpointer);
    }
  }, [traverseState]);

  return {
    laserpointer: state.value,
    setLaserpointer,
    laserpointerIsLoading: state.isOptimistic || !state.isInitialized,
    laserpointerIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

function useMeasurementTensionArm(
  machine_identification_unique: MachineIdentificationUnique,
): {
  measurementTensionArm: number;
  measurementTensionArmIsLoading: boolean;
} {
  const isLoading = useState(false);

  // Read Path
  const { measurementsTensionArms } = useWinder1Namespace(
    machine_identification_unique,
  );

  return {
    // set last
    measurementTensionArm:
      measurementsTensionArms.at(-1)?.content.Data?.degree ?? 0,
    measurementTensionArmIsLoading: measurementsTensionArms.length === 0,
  };
}

function useMode(machine_identification_unique: MachineIdentificationUnique): {
  mode: Mode | undefined;
  setMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    ModeSet: z.enum(["Standby", "Hold", "Pull", "Wind"]),
  });
  const { request } = useMachineMutation(schema);
  const setMode = async (value: Mode) => {
    state.setOptimistic(value);
    request({
      machine_identification_unique,
      data: { ModeSet: value },
    })
      .then((response) => {
        if (!response.success) state.resetToReal();
      })
      .catch(() => state.resetToReal());
  };

  // Read path
  const { modeState } = useWinder1Namespace(machine_identification_unique);
  useEffect(() => {
    if (modeState?.content.Data) {
      state.setReal(modeState.content.Data.mode);
    }
  }, [modeState]);

  return {
    mode: state.value,
    setMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

export function useWinder1() {
  const { serial: serialString } = winder2SerialRoute.useParams();

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

  const laserpointerControls = useLaserpointer(machineIdentification);
  const measurementTensionArm = useMeasurementTensionArm(machineIdentification);
  const mode = useMode(machineIdentification);

  return {
    ...laserpointerControls,
    ...measurementTensionArm,
    ...mode,
  };
}
