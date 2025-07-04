import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { winder2 } from "@/machines/properties";
import { winder2SerialRoute } from "@/routes/routes";
import { z } from "zod";
import {
  SpoolRegulationMode,
  StateEvent,
  useWinder2Namespace,
  modeSchema,
  Mode,
  spoolRegulationModeSchema,
  pullerRegulationSchema,
  PullerRegulation,
} from "./winder2Namespace";
import { useEffect, useMemo } from "react";
import { produce } from "immer";

function useSpoolSpeedController(
  machine_identification_unique: MachineIdentificationUnique,
): {
  spoolSpeedControllerState: SpoolSpeedControllerStateEvent | null;
  setRegulationMode: (mode: SpoolRegulationMode) => void;
  setMinMaxMinSpeed: (speed: number) => void;
  setMinMaxMaxSpeed: (speed: number) => void;
  setAdaptiveTensionTarget: (value: number) => void;
  setAdaptiveRadiusLearningRate: (value: number) => void;
  setAdaptiveMaxSpeedMultiplier: (value: number) => void;
  setAdaptiveAccelerationFactor: (value: number) => void;
  setAdaptiveDeaccelerationUrgencyMultiplier: (value: number) => void;
  spoolControllerIsLoading: boolean;
  spoolControllerIsDisabled: boolean;
} {
  const spoolStateOptimistic =
    useStateOptimistic<SpoolSpeedControllerStateEvent["data"]>();

  // Write path - Set regulation mode
  const regulationSchema = z.object({
    SpoolSetRegulationMode: z.enum(["Adaptive", "MinMax"]),
  });
  const { request: requestRegulation } = useMachineMutation(regulationSchema);
  const setRegulationMode = async (mode: SpoolRegulationMode) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        regulation_mode: mode,
      });
    }
    requestRegulation({
      machine_identification_unique,
      data: { SpoolSetRegulationMode: mode },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  // Write path - Set minmax min speed
  const minSpeedSchema = z.object({
    SpoolSetMinMaxMinSpeed: z.number(),
  });
  const { request: requestMinSpeed } = useMachineMutation(minSpeedSchema);
  const setMinMaxMinSpeed = async (speed: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        minmax_min_speed: speed,
      });
    }
    requestMinSpeed({
      machine_identification_unique,
      data: { SpoolSetMinMaxMinSpeed: speed },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  // Write path - Set minmax max speed
  const maxSpeedSchema = z.object({
    SpoolSetMinMaxMaxSpeed: z.number(),
  });
  const { request: requestMaxSpeed } = useMachineMutation(maxSpeedSchema);
  const setMinMaxMaxSpeed = async (speed: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        minmax_max_speed: speed,
      });
    }
    requestMaxSpeed({
      machine_identification_unique,
      data: { SpoolSetMinMaxMaxSpeed: speed },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  // Write path - Set adaptive parameters
  const adaptiveTensionTargetSchema = z.object({
    SpoolSetAdaptiveTensionTarget: z.number(),
  });
  const { request: requestAdaptiveTensionTarget } = useMachineMutation(
    adaptiveTensionTargetSchema,
  );
  const setAdaptiveTensionTarget = async (value: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        adaptive_tension_target: value,
      });
    }
    requestAdaptiveTensionTarget({
      machine_identification_unique,
      data: { SpoolSetAdaptiveTensionTarget: value },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  const adaptiveRadiusLearningRateSchema = z.object({
    SpoolSetAdaptiveRadiusLearningRate: z.number(),
  });
  const { request: requestAdaptiveRadiusLearningRate } = useMachineMutation(
    adaptiveRadiusLearningRateSchema,
  );
  const setAdaptiveRadiusLearningRate = async (value: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        adaptive_radius_learning_rate: value,
      });
    }
    requestAdaptiveRadiusLearningRate({
      machine_identification_unique,
      data: { SpoolSetAdaptiveRadiusLearningRate: value },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  const adaptiveMaxSpeedMultiplierSchema = z.object({
    SpoolSetAdaptiveMaxSpeedMultiplier: z.number(),
  });
  const { request: requestAdaptiveMaxSpeedMultiplier } = useMachineMutation(
    adaptiveMaxSpeedMultiplierSchema,
  );
  const setAdaptiveMaxSpeedMultiplier = async (value: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        adaptive_max_speed_multiplier: value,
      });
    }
    requestAdaptiveMaxSpeedMultiplier({
      machine_identification_unique,
      data: { SpoolSetAdaptiveMaxSpeedMultiplier: value },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  const adaptiveAccelerationFactorSchema = z.object({
    SpoolSetAdaptiveAccelerationFactor: z.number(),
  });
  const { request: requestAdaptiveAccelerationFactor } = useMachineMutation(
    adaptiveAccelerationFactorSchema,
  );
  const setAdaptiveAccelerationFactor = async (value: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        adaptive_acceleration_factor: value,
      });
    }
    requestAdaptiveAccelerationFactor({
      machine_identification_unique,
      data: { SpoolSetAdaptiveAccelerationFactor: value },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  const adaptiveDeaccelerationUrgencyMultiplierSchema = z.object({
    SpoolSetAdaptiveDeaccelerationUrgencyMultiplier: z.number(),
  });
  const { request: requestAdaptiveDeaccelerationUrgencyMultiplier } =
    useMachineMutation(adaptiveDeaccelerationUrgencyMultiplierSchema);
  const setAdaptiveDeaccelerationUrgencyMultiplier = async (value: number) => {
    if (spoolStateOptimistic.value) {
      spoolStateOptimistic.setOptimistic({
        ...spoolStateOptimistic.value,
        adaptive_deacceleration_urgency_multiplier: value,
      });
    }
    requestAdaptiveDeaccelerationUrgencyMultiplier({
      machine_identification_unique,
      data: { SpoolSetAdaptiveDeaccelerationUrgencyMultiplier: value },
    })
      .then((response) => {
        if (!response.success) spoolStateOptimistic.resetToReal();
      })
      .catch(() => spoolStateOptimistic.resetToReal());
  };

  // Read path
  const { spoolSpeedControllerState } = useWinder2Namespace(
    machine_identification_unique,
  );

  // Update real values from server
  useEffect(() => {
    if (spoolSpeedControllerState?.data) {
      spoolStateOptimistic.setReal(spoolSpeedControllerState.data);
    }
  }, [spoolSpeedControllerState]);

  return {
    spoolSpeedControllerState,
    setRegulationMode,
    setMinMaxMinSpeed,
    setMinMaxMaxSpeed,
    setAdaptiveTensionTarget,
    setAdaptiveRadiusLearningRate,
    setAdaptiveMaxSpeedMultiplier,
    setAdaptiveAccelerationFactor,
    setAdaptiveDeaccelerationUrgencyMultiplier,
    spoolControllerIsLoading:
      spoolStateOptimistic.isOptimistic || !spoolStateOptimistic.isInitialized,
    spoolControllerIsDisabled:
      spoolStateOptimistic.isOptimistic || !spoolStateOptimistic.isInitialized,
  };
}

export function useWinder2() {
  const { serial: serialString } = winder2SerialRoute.useParams();

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
      machine_identification: winder2.machine_identification,
      serial,
    };
  }, [serialString]);

  // Get consolidated state and live values from namespace
  const {
    state,
    traversePosition,
    pullerSpeed,
    spoolRpm,
    spoolDiameter,
    tensionArmAngle,
  } = useWinder2Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // Request functions for all operations
  const { request: requestTraverseGotoLimitInner } = useMachineMutation(
    z.literal("GotoTraverseLimitInner"),
  );
  const { request: requestTraverseGotoLimitOuter } = useMachineMutation(
    z.literal("GotoTraverseLimitOuter"),
  );
  const { request: requestTraverseGotoHome } = useMachineMutation(
    z.literal("GotoTraverseHome"),
  );
  const { request: requestSetLaserpointer } = useMachineMutation(
    z.object({ EnableTraverseLaserpointer: z.boolean() }),
  );
  const { request: requestModeSet } = useMachineMutation(
    z.object({ SetMode: modeSchema }),
  );
  const { request: requestTensionArmZero } = useMachineMutation(
    z.literal("ZeroTensionArmAngle"),
  );
  const { request: requestTraverseSetLimitInner } = useMachineMutation(
    z.object({ SetTraverseLimitInner: z.number() }),
  );
  const { request: requestTraverseSetLimitOuter } = useMachineMutation(
    z.object({ SetTraverseLimitOuter: z.number() }),
  );
  const { request: requestTraverseSetStepSize } = useMachineMutation(
    z.object({ SetTraverseStepSize: z.number() }),
  );
  const { request: requestTraverseSetPadding } = useMachineMutation(
    z.object({ SetTraversePadding: z.number() }),
  );
  const { request: requestPullerSetTargetSpeed } = useMachineMutation(
    z.object({ SetPullerTargetSpeed: z.number() }),
  );
  const { request: requestPullerSetTargetDiameter } = useMachineMutation(
    z.object({ SetPullerTargetDiameter: z.number() }),
  );
  const { request: requestPullerSetRegulationMode } = useMachineMutation(
    z.object({
      SetPullerRegulationMode: pullerRegulationSchema,
    }),
  );
  const { request: requestPullerSetForward } = useMachineMutation(
    z.object({ SetPullerForward: z.boolean() }),
  );
  const { request: requestSpoolSetRegulationMode } = useMachineMutation(
    z.object({ SetSpoolRegulationMode: spoolRegulationModeSchema }),
  );
  const { request: requestSpoolSetMinMaxMinSpeed } = useMachineMutation(
    z.object({ SetSpoolMinMaxMinSpeed: z.number() }),
  );
  const { request: requestSpoolSetMinMaxMaxSpeed } = useMachineMutation(
    z.object({ SetSpoolMinMaxMaxSpeed: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveTensionTarget } = useMachineMutation(
    z.object({ SetSpoolAdaptiveTensionTarget: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveRadiusLearningRate } =
    useMachineMutation(
      z.object({ SetSpoolAdaptiveRadiusLearningRate: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveMaxSpeedMultiplier } =
    useMachineMutation(
      z.object({ SetSpoolAdaptiveMaxSpeedMultiplier: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveAccelerationFactor } =
    useMachineMutation(
      z.object({ SetSpoolAdaptiveAccelerationFactor: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier } =
    useMachineMutation(
      z.object({ SetSpoolAdaptiveDeaccelerationUrgencyMultiplier: z.number() }),
    );

  // Helper function for optimistic updates using produce
  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  // Action functions
  const zeroTensionArmAngle = () => {
    updateStateOptimistically(
      (current) => {
        current.data.tension_arm_state.zeroed = true;
      },
      () =>
        requestTensionArmZero({
          machine_identification_unique: machineIdentification,
          data: "ZeroTensionArmAngle",
        }),
    );
  };

  const setTraverseLimitInner = (limitInner: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_inner = limitInner;
      },
      () =>
        requestTraverseSetLimitInner({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitInner: limitInner },
        }),
    );
  };

  const setTraverseLimitOuter = (limitOuter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_outer = limitOuter;
      },
      () =>
        requestTraverseSetLimitOuter({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitOuter: limitOuter },
        }),
    );
  };

  const gotoTraverseLimitInner = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_in = true;
      },
      () =>
        requestTraverseGotoLimitInner({
          machine_identification_unique: machineIdentification,
          data: "GotoTraverseLimitInner",
        }),
    );
  };

  const gotoTraverseLimitOuter = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_out = true;
      },
      () =>
        requestTraverseGotoLimitOuter({
          machine_identification_unique: machineIdentification,
          data: "GotoTraverseLimitOuter",
        }),
    );
  };

  const gotoTraverseHome = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_home = true;
      },
      () =>
        requestTraverseGotoHome({
          machine_identification_unique: machineIdentification,
          data: "GotoTraverseHome",
        }),
    );
  };

  const enableTraverseLaserpointer = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.laserpointer = enabled;
      },
      () =>
        requestSetLaserpointer({
          machine_identification_unique: machineIdentification,
          data: { EnableTraverseLaserpointer: enabled },
        }),
    );
  };

  const setMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestModeSet({
          machine_identification_unique: machineIdentification,
          data: { SetMode: mode },
        }),
    );
  };

  const setTraverseStepSize = (stepSize: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.step_size = stepSize;
      },
      () =>
        requestTraverseSetStepSize({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseStepSize: stepSize },
        }),
    );
  };

  const setTraversePadding = (padding: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.padding = padding;
      },
      () =>
        requestTraverseSetPadding({
          machine_identification_unique: machineIdentification,
          data: { SetTraversePadding: padding },
        }),
    );
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_speed = targetSpeed;
      },
      () =>
        requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetSpeed: targetSpeed },
        }),
    );
  };

  const setPullerTargetDiameter = (targetDiameter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_diameter = targetDiameter;
      },
      () =>
        requestPullerSetTargetDiameter({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetDiameter: targetDiameter },
        }),
    );
  };

  const setPullerRegulationMode = (regulationMode: PullerRegulation) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.regulation = regulationMode;
      },
      () =>
        requestPullerSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SetPullerRegulationMode: regulationMode },
        }),
    );
  };

  const setPullerForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.forward = forward;
      },
      () =>
        requestPullerSetForward({
          machine_identification_unique: machineIdentification,
          data: { SetPullerForward: forward },
        }),
    );
  };

  const setSpoolRegulationMode = (mode: SpoolRegulationMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.regulation_mode = mode;
      },
      () =>
        requestSpoolSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolRegulationMode: mode },
        }),
    );
  };

  const setSpoolMinMaxMinSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.minmax_min_speed = speed;
      },
      () =>
        requestSpoolSetMinMaxMinSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolMinMaxMinSpeed: speed },
        }),
    );
  };

  const setSpoolMinMaxMaxSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.minmax_max_speed = speed;
      },
      () =>
        requestSpoolSetMinMaxMaxSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolMinMaxMaxSpeed: speed },
        }),
    );
  };

  const setSpoolAdaptiveTensionTarget = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_tension_target =
          value;
      },
      () =>
        requestSpoolSetAdaptiveTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveTensionTarget: value },
        }),
    );
  };

  const setSpoolAdaptiveRadiusLearningRate = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_radius_learning_rate =
          value;
      },
      () =>
        requestSpoolSetAdaptiveRadiusLearningRate({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveRadiusLearningRate: value },
        }),
    );
  };

  const setSpoolAdaptiveMaxSpeedMultiplier = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_max_speed_multiplier =
          value;
      },
      () =>
        requestSpoolSetAdaptiveMaxSpeedMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveMaxSpeedMultiplier: value },
        }),
    );
  };

  const setSpoolAdaptiveAccelerationFactor = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_acceleration_factor =
          value;
      },
      () =>
        requestSpoolSetAdaptiveAccelerationFactor({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveAccelerationFactor: value },
        }),
    );
  };

  const setSpoolAdaptiveDeaccelerationUrgencyMultiplier = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_deacceleration_urgency_multiplier =
          value;
      },
      () =>
        requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveDeaccelerationUrgencyMultiplier: value },
        }),
    );
  };

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    // Individual live values (TimeSeries)
    traversePosition,
    pullerSpeed,
    spoolRpm,
    spoolDiameter,
    tensionArmAngle,

    // Loading states
    isLoading,
    isDisabled,

    // Action functions
    enableTraverseLaserpointer,
    setMode,
    zeroTensionArmAngle,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseHome,
    setTraverseStepSize,
    setTraversePadding,
    setPullerTargetSpeed,
    setPullerTargetDiameter,
    setPullerRegulationMode,
    setPullerForward,
    setSpoolRegulationMode,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,
  };
}
