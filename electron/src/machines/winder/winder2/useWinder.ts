import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import {
  machineIdentificationUnique,
  MachineIdentificationUnique,
} from "@/machines/types";
import { VENDOR_QITECH, winder2 } from "@/machines/properties";
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
  SpoolAutomaticActionMode,
  spoolAutomaticActionModeSchema,
  gearRatioSchema,
  GearRatio,
} from "./winder2Namespace";
import { useEffect, useMemo } from "react";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";

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

  const machine_identification_unique = machineIdentification;

  // Get consolidated state and live values from namespace
  const {
    state,
    defaultState,
    traversePosition,
    pullerSpeed,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
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
  const { request: requestPullerSetGearRatio } = useMachineMutation(
    z.object({ SetPullerGearRatio: gearRatioSchema }),
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
  const { request: requestSpoolSetForward } = useMachineMutation(
    z.object({ SetSpoolForward: z.boolean() }),
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
  const { request: requestSpoolAutomaticRequiredMeters } = useMachineMutation(
    z.object({ SetSpoolAutomaticRequiredMeters: z.number() }),
  );
  const { request: requestSpoolResetProgress } = useMachineMutation(
    z.literal("ResetSpoolProgress"),
  );

  const { request: requestSpoolAutomaticAction } = useMachineMutation(
    z.object({ SetSpoolAutomaticAction: spoolAutomaticActionModeSchema }),
  );

  const { request: requestConnectedMachine } = useMachineMutation(
    z.object({
      SetConnectedMachine: machineIdentificationUnique,
    }),
  );

  const { request: requestDisconnectedMachine } = useMachineMutation(
    z.object({
      DisconnectMachine: machineIdentificationUnique,
    }),
  );

  // Helper function for optimistic updates using produce
  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
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

  const setPullerGearRatio = (gearRatio: GearRatio) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.gear_ratio = gearRatio;
        // Reset target speed to 0 to prevent sudden speed changes
        current.data.puller_state.target_speed = 0;
      },
      async () => {
        // First set the target speed to 0
        await requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetSpeed: 0 },
        });
        // Then set the gear ratio
        await requestPullerSetGearRatio({
          machine_identification_unique: machineIdentification,
          data: { SetPullerGearRatio: gearRatio },
        });
      },
    );
  };

  const setSpoolAutomaticRequiredMeters = (meters: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_automatic_action_state.spool_required_meters =
          meters;
      },
      () =>
        requestSpoolAutomaticRequiredMeters({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAutomaticRequiredMeters: meters },
        }),
    );
  };

  const setSpoolAutomaticAction = (mode: SpoolAutomaticActionMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_automatic_action_state.spool_automatic_action_mode =
          mode;
      },
      () =>
        requestSpoolAutomaticAction({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAutomaticAction: mode },
        }),
    );
  };

  const resetSpoolProgress = () => {
    requestSpoolResetProgress({
      machine_identification_unique: machineIdentification,
      data: "ResetSpoolProgress",
    });
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

  const setSpoolForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.forward = forward;
      },
      () =>
        requestSpoolSetForward({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolForward: forward },
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

  const setConnectedMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          machineIdentificationUnique;
      },
      () =>
        requestConnectedMachine({
          machine_identification_unique,
          data: { SetConnectedMachine: machineIdentificationUnique },
        }),
    );
  };

  const disconnectMachine = (machineIdentificationUnique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  }) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          null;
      },
      () =>
        requestDisconnectedMachine({
          machine_identification_unique,
          data: { DisconnectMachine: machineIdentificationUnique },
        }),
    );
  };

  // Calculate loading states
  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  // General Helper functions
  const machines = useMachines();
  // Filter machines for the correct type
  const filteredMachines = useMemo(
    () =>
      machines.filter(
        (m) =>
          m.machine_identification_unique.machine_identification.vendor ===
            VENDOR_QITECH &&
          m.machine_identification_unique.machine_identification.machine ===
            0x0008,
      ),
    [machines, machineIdentification],
  );

  // Get selected machine by serial
  const selectedMachine = useMemo(() => {
    const serial =
      state?.data.connected_machine_state?.machine_identification_unique
        ?.serial;

    return (
      filteredMachines.find(
        (m) => m.machine_identification_unique.serial === serial,
      ) ?? null
    );
  }, [filteredMachines, state]);

  return {
    // Consolidated state
    state: stateOptimistic.value?.data,

    filteredMachines,
    selectedMachine,

    // Default state for initial values
    defaultState: defaultState?.data,

    // Individual live values (TimeSeries)
    traversePosition,
    pullerSpeed,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,

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
    resetSpoolProgress,
    setTraverseStepSize,
    setTraversePadding,
    setPullerTargetSpeed,
    setPullerTargetDiameter,
    setPullerRegulationMode,
    setPullerForward,
    setPullerGearRatio,
    setSpoolAutomaticRequiredMeters,
    setSpoolAutomaticAction,
    setSpoolRegulationMode,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolForward,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setConnectedMachine,
    disconnectMachine,
  };
}
