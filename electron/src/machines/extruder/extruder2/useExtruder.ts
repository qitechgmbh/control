import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, extruder2 } from "@/machines/types";
import { extruder2Route } from "@/routes/routes";
import { z } from "zod";
import { Mode, useExtruder2Namespace } from "./extruder2Namespace";
import { useEffect, useMemo, useState } from "react";
import { TimeSeries } from "@/lib/timeseries";

export function useExtruder2() {
  const { serial: serialString } = extruder2Route.useParams();

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
      ...extruder2.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const inverter = useInverter(machineIdentification);
  const mode = useMode(machineIdentification);

  return {
    ...inverter,
    ...mode,
  };
}

export function useInverter(
  machine_identification_unique: MachineIdentificationUnique,
) {
  const state = useStateOptimistic<boolean>();

  const schema = z.object({ SetRotation: z.boolean() });
  const { request: requestRotation } = useMachineMutation(schema);
  const inverterSetRotation = async (forward: boolean) => {
    state.setReal(forward);
    requestRotation({
      machine_identification_unique,
      data: { SetRotation: forward },
    });
  };
  return { inverterSetRotation };
}

export function useMode(
  machine_identification_unique: MachineIdentificationUnique,
): {
  mode: Mode | undefined;
  setMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    ModeSet: z.enum(["Heat", "Extrude", "Standby"]),
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
  const { modeState } = useExtruder2Namespace(machine_identification_unique);
  useEffect(() => {
    if (modeState?.data) {
      state.setReal(modeState.data.mode);
    }
  }, [modeState]);

  return {
    mode: state.value,
    setMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}
