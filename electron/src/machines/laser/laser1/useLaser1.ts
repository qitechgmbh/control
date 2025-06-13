import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { laser1, MachineIdentificationUnique } from "@/machines/types";
import { laser1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useLaser1Namespace } from "./laser1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { FPS_60, useThrottle } from "@/lib/useThrottle";

function useLaser(machine_identification_unique: MachineIdentificationUnique) {
  // Write Path
  const laserStateOptimistic = useStateOptimistic<{
    target_diameter: number;
    lower_tolerance: number;
    higher_tolerance: number;
  }>();
  const schemaTargetDiameter = z.object({
    TargetSetTargetDiameter: z.number(),
  });
  const { request: requestTargetDiameter } =
    useMachineMutation(schemaTargetDiameter);
  const laserSetTargetDiameter = async (target_diameter: number) => {
    if (laserStateOptimistic.value) {
      laserStateOptimistic.setOptimistic({
        ...laserStateOptimistic.value,
        target_diameter: target_diameter,
      });
    }
    requestTargetDiameter({
      machine_identification_unique,
      data: {
        TargetSetTargetDiameter: target_diameter,
      },
    })
      .then((response) => {
        if (!response.success) laserStateOptimistic.resetToReal();
      })
      .catch(() => laserStateOptimistic.resetToReal());
  };

  const schemaLowerTolerance = z.object({
    TargetSetLowerTolerance: z.number(),
  });
  const { request: requestLowerTolerance } =
    useMachineMutation(schemaLowerTolerance);
  const laserSetLowerTolerance = async (lower_tolerance: number) => {
    if (laserStateOptimistic.value) {
      laserStateOptimistic.setOptimistic({
        ...laserStateOptimistic.value,
        lower_tolerance: lower_tolerance,
      });
    }
    requestLowerTolerance({
      machine_identification_unique,
      data: {
        TargetSetLowerTolerance: lower_tolerance,
      },
    })
      .then((response) => {
        if (!response.success) laserStateOptimistic.resetToReal();
      })
      .catch(() => laserStateOptimistic.resetToReal());
  };

  const schemaHigherTolerance = z.object({
    TargetSetHigherTolerance: z.number(),
  });
  const { request: requestHigherTolerance } = useMachineMutation(
    schemaHigherTolerance,
  );
  const laserSetHigherTolerance = async (higher_tolerance: number) => {
    if (laserStateOptimistic.value) {
      laserStateOptimistic.setOptimistic({
        ...laserStateOptimistic.value,
        higher_tolerance: higher_tolerance,
      });
    }
    requestHigherTolerance({
      machine_identification_unique,
      data: {
        TargetSetHigherTolerance: higher_tolerance,
      },
    })
      .then((response) => {
        if (!response.success) laserStateOptimistic.resetToReal();
      })
      .catch(() => laserStateOptimistic.resetToReal());
  };

  // Read Path
  const { laserDiameter, laserState } = useLaser1Namespace(
    machine_identification_unique,
  );

  // Update real values from server
  useEffect(() => {
    if (laserState?.data) {
      laserStateOptimistic.setReal(laserState.data);
    }
  }, [laserState]);

  // throttle to 60fps
  const laserDiameterThrottled = useThrottle(laserDiameter, FPS_60);

  return {
    laserDiameter: laserDiameterThrottled,
    laserState,
    laserSetTargetDiameter: laserSetTargetDiameter,
    laserSetLowerTolerance: laserSetLowerTolerance,
    laserSetHigherTolerance: laserSetHigherTolerance,
    laserStateIsLoading:
      laserStateOptimistic.isOptimistic || !laserStateOptimistic.isInitialized,
    laserStateIsDisabled:
      laserStateOptimistic.isOptimistic || !laserStateOptimistic.isInitialized,
  };
}

export function useLaser1() {
  const { serial: serialString } = laser1SerialRoute.useParams();

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
      machine_identification: laser1.machine_identification,
      serial,
    };
  }, [serialString]); // Only recreate when serialString changes

  const laser = useLaser(machineIdentification);

  return {
    ...laser,
  };
}
