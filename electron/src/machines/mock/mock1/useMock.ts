import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { mock1, MachineIdentificationUnique } from "@/machines/types";
import { mock1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useMock1Namespace, Mode } from "./mock1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { FPS_60, useThrottle } from "@/lib/useThrottle";

function useMock(machine_identification_unique: MachineIdentificationUnique) {
  // Write Path
  const mockStateOptimistic = useStateOptimistic<{
    frequency: number;
  }>();

  const modeStateOptimistic = useStateOptimistic<{
    mode: Mode;
  }>();

  const schemaSetFrequency = z.object({ SetFrequency: z.number() });
  const { request: requestSetFrequency } =
    useMachineMutation(schemaSetFrequency);
  const mockSetFrequency = async (frequency: number) => {
    if (mockStateOptimistic.value) {
      mockStateOptimistic.setOptimistic({
        ...mockStateOptimistic.value,
        frequency: frequency,
      });
    }
    requestSetFrequency({
      machine_identification_unique,
      data: {
        SetFrequency: frequency,
      },
    })
      .then((response) => {
        if (!response.success) mockStateOptimistic.resetToReal();
      })
      .catch(() => mockStateOptimistic.resetToReal());
  };

  const schemaSetMode = z.object({ SetMode: z.enum(["Standby", "Running"]) });
  const { request: requestSetMode } = useMachineMutation(schemaSetMode);
  const mockSetMode = async (mode: Mode) => {
    if (modeStateOptimistic.value) {
      modeStateOptimistic.setOptimistic({
        ...modeStateOptimistic.value,
        mode: mode,
      });
    }
    requestSetMode({
      machine_identification_unique,
      data: {
        SetMode: mode,
      },
    })
      .then((response) => {
        if (!response.success) modeStateOptimistic.resetToReal();
      })
      .catch(() => modeStateOptimistic.resetToReal());
  };

  // Read Path
  const { sineWave, mockState, modeState } = useMock1Namespace(
    machine_identification_unique,
  );

  // Throttle UI updates to 60 FPS
  const debouncedSineWave = useThrottle(sineWave, FPS_60); // 60fps

  // Update real values from server
  useEffect(() => {
    if (mockState?.data) {
      mockStateOptimistic.setReal(mockState.data);
    }
  }, [mockState]);

  useEffect(() => {
    if (modeState?.data) {
      modeStateOptimistic.setReal(modeState.data);
    }
  }, [modeState]);

  return {
    sineWave: debouncedSineWave,
    mockState,
    modeState,
    mockSetFrequency,
    mockSetMode,
    mockStateIsLoading:
      mockStateOptimistic.isOptimistic || !mockStateOptimistic.isInitialized,
    mockStateIsDisabled:
      mockStateOptimistic.isOptimistic || !mockStateOptimistic.isInitialized,
    modeStateIsLoading:
      modeStateOptimistic.isOptimistic || !modeStateOptimistic.isInitialized,
    modeStateIsDisabled:
      modeStateOptimistic.isOptimistic || !modeStateOptimistic.isInitialized,
  };
}

export function useMock1() {
  const { serial: serialString } = mock1SerialRoute.useParams();

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
      machine_identification: mock1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const mock = useMock(machineIdentification);

  return {
    ...mock,
  };
}
