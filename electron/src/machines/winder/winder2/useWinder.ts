import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique, winder2 } from "@/machines/types";
import { winder2SerialRoute } from "@/routes/routes";
import { z } from "zod";
import {
  Mode,
  PullerStateEvent,
  TensionArmStateEvent,
  ModeStateEvent,
  useWinder2Namespace,
} from "./winder2Namespace";
import { useEffect, useMemo } from "react";

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
  const tensionArmStateOptimistic =
    useStateOptimistic<TensionArmStateEvent["data"]>();

  const schema = z.literal("TensionArmAngleZero");
  const { request } = useMachineMutation(schema);
  const tensionArmAngleZero = async () => {
    // Update optimistic state
    if (tensionArmStateOptimistic.value) {
      tensionArmStateOptimistic.setOptimistic({
        ...tensionArmStateOptimistic.value,
        zeroed: true,
      });
    }

    request({
      machine_identification_unique,
      data: "TensionArmAngleZero",
    })
      .then((response) => {
        if (!response.success) tensionArmStateOptimistic.resetToReal();
      })
      .catch(() => tensionArmStateOptimistic.resetToReal());
  };

  // Read Path
  const { tensionArmAngle, tensionArmState } = useWinder2Namespace(
    machine_identification_unique,
  );

  // Update real values from server
  useEffect(() => {
    if (tensionArmState?.data) {
      tensionArmStateOptimistic.setReal({
        zeroed: tensionArmState.data.zeroed,
      });
    }
  }, [tensionArmState]);

  return {
    tensionArmAngle,
    tensionArmState,
    tensionArmAngleZero,
    tensionArmStateIsLoading:
      tensionArmStateOptimistic.isOptimistic ||
      !tensionArmStateOptimistic.isInitialized,
    tensionArmStateIsDisabled:
      tensionArmStateOptimistic.isOptimistic ||
      !tensionArmStateOptimistic.isInitialized,
  };
}

function useTraverse(
  machine_identification_unique: MachineIdentificationUnique,
) {
  // Write Path
  const traverseStateOptimistic = useStateOptimistic<{
    limit_inner: number;
    limit_outer: number;
    step_size: number;
    padding: number;
  }>();

  const schemaSetLimitInner = z.object({
    TraverseSetLimitInner: z.number(),
  });
  const { request: requestSetLimitInner } =
    useMachineMutation(schemaSetLimitInner);
  const traverseSetLimitInner = async (limitInner: number) => {
    if (traverseStateOptimistic.value) {
      traverseStateOptimistic.setOptimistic({
        ...traverseStateOptimistic.value,
        limit_inner: limitInner,
      });
    }
    requestSetLimitInner({
      machine_identification_unique,
      data: { TraverseSetLimitInner: limitInner },
    })
      .then((response) => {
        if (!response.success) traverseStateOptimistic.resetToReal();
      })
      .catch(() => traverseStateOptimistic.resetToReal());
  };

  const schemaSetLimitOuter = z.object({
    TraverseSetLimitOuter: z.number(),
  });
  const { request: requestSetLimitOuter } =
    useMachineMutation(schemaSetLimitOuter);
  const traverseSetLimitOuter = async (limitOuter: number) => {
    if (traverseStateOptimistic.value) {
      traverseStateOptimistic.setOptimistic({
        ...traverseStateOptimistic.value,
        limit_outer: limitOuter,
      });
    }
    requestSetLimitOuter({
      machine_identification_unique,
      data: { TraverseSetLimitOuter: limitOuter },
    })
      .then((response) => {
        if (!response.success) traverseStateOptimistic.resetToReal();
      })
      .catch(() => traverseStateOptimistic.resetToReal());
  };

  const schemaGotoLimitInner = z.literal("TraverseGotoLimitInner");
  const { request: requestGotoLimitInner } =
    useMachineMutation(schemaGotoLimitInner);
  const traverseGotoLimitInner = async () => {
    requestGotoLimitInner({
      machine_identification_unique,
      data: "TraverseGotoLimitInner",
    });
  };

  const schemaGotoLimitOuter = z.literal("TraverseGotoLimitOuter");
  const { request: requestGotoLimitOuter } =
    useMachineMutation(schemaGotoLimitOuter);
  const traverseGotoLimitOuter = async () => {
    requestGotoLimitOuter({
      machine_identification_unique,
      data: "TraverseGotoLimitOuter",
    });
  };

  const schemaGotoHome = z.literal("TraverseGotoHome");
  const { request: requestGotoHome } = useMachineMutation(schemaGotoHome);
  const traverseGotoHome = async () => {
    requestGotoHome({
      machine_identification_unique,
      data: "TraverseGotoHome",
    });
  };

  const schemaSetStepSize = z.object({
    TraverseSetStepSize: z.number(),
  });
  const { request: requestSetStepSize } = useMachineMutation(schemaSetStepSize);
  const traverseSetStepSize = async (stepSize: number) => {
    if (traverseStateOptimistic.value) {
      traverseStateOptimistic.setOptimistic({
        ...traverseStateOptimistic.value,
        step_size: stepSize,
      });
    }
    requestSetStepSize({
      machine_identification_unique,
      data: { TraverseSetStepSize: stepSize },
    })
      .then((response) => {
        if (!response.success) traverseStateOptimistic.resetToReal();
      })
      .catch(() => traverseStateOptimistic.resetToReal());
  };

  const schemaSetPadding = z.object({
    TraverseSetPadding: z.number(),
  });
  const { request: requestSetPadding } = useMachineMutation(schemaSetPadding);
  const traverseSetPadding = async (padding: number) => {
    if (traverseStateOptimistic.value) {
      traverseStateOptimistic.setOptimistic({
        ...traverseStateOptimistic.value,
        padding: padding,
      });
    }
    requestSetPadding({
      machine_identification_unique,
      data: { TraverseSetPadding: padding },
    })
      .then((response) => {
        if (!response.success) traverseStateOptimistic.resetToReal();
      })
      .catch(() => traverseStateOptimistic.resetToReal());
  };

  // Read Path
  const { traversePosition, traverseState } = useWinder2Namespace(
    machine_identification_unique,
  );

  // Update real values from server
  useEffect(() => {
    if (traverseState?.data) {
      traverseStateOptimistic.setReal({
        limit_inner: traverseState.data.limit_inner,
        limit_outer: traverseState.data.limit_outer,
        step_size: traverseState.data.step_size,
        padding: traverseState.data.padding,
      });
    }
  }, [traverseState]);

  return {
    traversePosition,
    traverseState,
    traverseSetLimitInner,
    traverseSetLimitOuter,
    traverseGotoLimitInner,
    traverseGotoLimitOuter,
    traverseGotoHome,
    traverseSetStepSize,
    traverseSetPadding,
    traverseStateIsLoading:
      traverseStateOptimistic.isOptimistic ||
      !traverseStateOptimistic.isInitialized,
    traverseStateIsDisabled:
      traverseStateOptimistic.isOptimistic ||
      !traverseStateOptimistic.isInitialized,
  };
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

function usePuller(machine_identification_unique: MachineIdentificationUnique) {
  // Write Path
  const pullerStateOptimistic = useStateOptimistic<PullerStateEvent["data"]>();

  const schemaSetTargetSpeed = z.object({
    PullerSetTargetSpeed: z.number(),
  });
  const { request: requestSetTargetSpeed } =
    useMachineMutation(schemaSetTargetSpeed);
  const pullerSetTargetSpeed = async (targetSpeed: number) => {
    if (pullerStateOptimistic.value) {
      pullerStateOptimistic.setOptimistic({
        ...pullerStateOptimistic.value,
        target_speed: targetSpeed,
      });
    }
    requestSetTargetSpeed({
      machine_identification_unique,
      data: { PullerSetTargetSpeed: targetSpeed },
    })
      .then((response) => {
        if (!response.success) pullerStateOptimistic.resetToReal();
      })
      .catch(() => pullerStateOptimistic.resetToReal());
  };

  const schemaSetTargetDiameter = z.object({
    PullerSetTargetDiameter: z.number(),
  });
  const { request: requestSetTargetDiameter } = useMachineMutation(
    schemaSetTargetDiameter,
  );
  const pullerSetTargetDiameter = async (targetDiameter: number) => {
    if (pullerStateOptimistic.value) {
      pullerStateOptimistic.setOptimistic({
        ...pullerStateOptimistic.value,
        target_diameter: targetDiameter,
      });
    }
    requestSetTargetDiameter({
      machine_identification_unique,
      data: { PullerSetTargetDiameter: targetDiameter },
    })
      .then((response) => {
        if (!response.success) pullerStateOptimistic.resetToReal();
      })
      .catch(() => pullerStateOptimistic.resetToReal());
  };

  const schemaSetRegulationMode = z.object({
    PullerSetRegulationMode: z.enum(["Speed", "Diameter"]),
  });
  const { request: requestSetRegulationMode } = useMachineMutation(
    schemaSetRegulationMode,
  );
  const pullerSetRegulationMode = async (
    regulationMode: "Speed" | "Diameter",
  ) => {
    if (pullerStateOptimistic.value) {
      pullerStateOptimistic.setOptimistic({
        ...pullerStateOptimistic.value,
        regulation: regulationMode,
      });
    }
    requestSetRegulationMode({
      machine_identification_unique,
      data: { PullerSetRegulationMode: regulationMode },
    })
      .then((response) => {
        if (!response.success) pullerStateOptimistic.resetToReal();
      })
      .catch(() => pullerStateOptimistic.resetToReal());
  };

  const schemaSetForward = z.object({
    PullerSetForward: z.boolean(),
  });
  const { request: requestSetForward } = useMachineMutation(schemaSetForward);
  const pullerSetForward = async (forward: boolean) => {
    if (pullerStateOptimistic.value) {
      pullerStateOptimistic.setOptimistic({
        ...pullerStateOptimistic.value,
        forward: forward,
      });
    }
    requestSetForward({
      machine_identification_unique,
      data: { PullerSetForward: forward },
    })
      .then((response) => {
        if (!response.success) pullerStateOptimistic.resetToReal();
      })
      .catch(() => pullerStateOptimistic.resetToReal());
  };

  // Read Path
  const { pullerState, pullerSpeed } = useWinder2Namespace(
    machine_identification_unique,
  );

  // Update real values from server
  useEffect(() => {
    if (pullerState?.data) {
      pullerStateOptimistic.setReal(pullerState.data);
    }
  }, [pullerState]);

  return {
    pullerState,
    pullerSpeed,
    pullerSetTargetSpeed,
    pullerSetTargetDiameter,
    pullerSetRegulationMode,
    pullerSetForward,
    pullerStateIsLoading:
      pullerStateOptimistic.isOptimistic ||
      !pullerStateOptimistic.isInitialized,
    pullerStateIsDisabled:
      pullerStateOptimistic.isOptimistic ||
      !pullerStateOptimistic.isInitialized,
  };
}

function useMode(machine_identification_unique: MachineIdentificationUnique): {
  mode: Mode | undefined;
  ExtruderSetMode: (value: Mode) => void;
  modeIsLoading: boolean;
  modeIsDisabled: boolean;
  modeState: ModeStateEvent | null;
} {
  const state = useStateOptimistic<Mode>();

  // Write path
  const schema = z.object({
    ModeSet: z.enum(["Standby", "Hold", "Pull", "Wind"]),
  });
  const { request } = useMachineMutation(schema);

  const ExtruderSetMode = async (value: Mode) => {
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
    ExtruderSetMode,
    modeIsLoading: state.isOptimistic || !state.isInitialized,
    modeIsDisabled: state.isOptimistic || !state.isInitialized,
    modeState,
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
  const puller = usePuller(machineIdentification);
  const mode = useMode(machineIdentification);
  const traverse = useTraverse(machineIdentification);

  return {
    ...laserpointerControls,
    ...mode,
    ...tensionArm,
    ...spool,
    ...puller,
    ...traverse,
  };
}
