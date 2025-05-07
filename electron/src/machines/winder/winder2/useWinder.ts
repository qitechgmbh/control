import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, winder2 } from "@/machines/types";
import { winder2SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { Mode, useWinder2Namespace } from "./winder2Namespace";
import { useEffect, useMemo } from "react";
import { TimeSeries } from "@/lib/timeseries";

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
  const { traverseState } = useWinder2Namespace(machine_identification_unique);
  useEffect(() => {
    if (traverseState?.data) {
      state.setReal(traverseState.data.laserpointer);
    }
  }, [traverseState]);

  return {
    laserpointer: state.value,
    setLaserpointer,
    laserpointerIsLoading: state.isOptimistic || !state.isInitialized,
    laserpointerIsDisabled: state.isOptimistic || !state.isInitialized,
  };
}

function useTensionArm(
  machine_identification_unique: MachineIdentificationUnique,
) {
  // Write Path
  const schema = z.literal("TensionArmAngleZero");
  const { request } = useMachineMutation(schema);
  const tensionArmAngleZero = async () => {
    request({
      machine_identification_unique,
      data: "TensionArmAngleZero",
    });
  };

  // Read Path
  const { tensionArmAngle, tensionArmState } = useWinder2Namespace(
    machine_identification_unique,
  );

  return { tensionArmAngle, tensionArmState, tensionArmAngleZero };
}

function useSpool(machine_identification_unique: MachineIdentificationUnique) {
  // Write Path
  const schemaMin = z.object({ SpoolSetSpeedMin: z.number() });
  const { request: requestMin } = useMachineMutation(schemaMin);
  const spoolSetSpeedMin = async (speedMin: number) => {
    requestMin({
      machine_identification_unique,
      data: {
        SpoolSetSpeedMin: speedMin,
      },
    });
  };

  const schemaMax = z.object({ SpoolSetSpeedMax: z.number() });
  const { request: requestMax } = useMachineMutation(schemaMax);
  const spoolSetSpeedMax = async (speedMax: number) => {
    requestMax({
      machine_identification_unique,
      data: {
        SpoolSetSpeedMax: speedMax,
      },
    });
  };

  // Read Path
  const { spoolRpm, spoolState } = useWinder2Namespace(
    machine_identification_unique,
  );

  return { spoolRpm, spoolState, spoolSetSpeedMin, spoolSetSpeedMax };
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
  const { modeState } = useWinder2Namespace(machine_identification_unique);
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

export function useWinder2() {
  const { serial: serialString } = winder2SerialRoute.useParams();

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
      machine_identification: winder2.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const laserpointerControls = useLaserpointer(machineIdentification);
  const tensionArm = useTensionArm(machineIdentification);
  const spool = useSpool(machineIdentification);
  const mode = useMode(machineIdentification);

  return {
    ...laserpointerControls,
    ...mode,
    ...tensionArm,
    ...spool,
  };
}
