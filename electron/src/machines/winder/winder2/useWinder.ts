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
    z.literal("TraverseGotoLimitInner"),
  );
  const { request: requestTraverseGotoLimitOuter } = useMachineMutation(
    z.literal("TraverseGotoLimitOuter"),
  );
  const { request: requestTraverseGotoHome } = useMachineMutation(
    z.literal("TraverseGotoHome"),
  );
  const { request: requestSetLaserpointer } = useMachineMutation(
    z.object({ TraverseEnableLaserpointer: z.boolean() }),
  );
  const { request: requestModeSet } = useMachineMutation(
    z.object({ ModeSet: modeSchema }),
  );
  const { request: requestTensionArmZero } = useMachineMutation(
    z.literal("TensionArmAngleZero"),
  );
  const { request: requestTraverseSetLimitInner } = useMachineMutation(
    z.object({ TraverseSetLimitInner: z.number() }),
  );
  const { request: requestTraverseSetLimitOuter } = useMachineMutation(
    z.object({ TraverseSetLimitOuter: z.number() }),
  );
  const { request: requestTraverseSetStepSize } = useMachineMutation(
    z.object({ TraverseSetStepSize: z.number() }),
  );
  const { request: requestTraverseSetPadding } = useMachineMutation(
    z.object({ TraverseSetPadding: z.number() }),
  );
  const { request: requestPullerSetTargetSpeed } = useMachineMutation(
    z.object({ PullerSetTargetSpeed: z.number() }),
  );
  const { request: requestPullerSetTargetDiameter } = useMachineMutation(
    z.object({ PullerSetTargetDiameter: z.number() }),
  );
  const { request: requestPullerSetRegulationMode } = useMachineMutation(
    z.object({
      PullerSetRegulationMode: pullerRegulationSchema,
    }),
  );
  const { request: requestPullerSetForward } = useMachineMutation(
    z.object({ PullerSetForward: z.boolean() }),
  );
  const { request: requestSpoolSetRegulationMode } = useMachineMutation(
    z.object({ SpoolSetRegulationMode: spoolRegulationModeSchema }),
  );
  const { request: requestSpoolSetMinMaxMinSpeed } = useMachineMutation(
    z.object({ SpoolSetMinMaxMinSpeed: z.number() }),
  );
  const { request: requestSpoolSetMinMaxMaxSpeed } = useMachineMutation(
    z.object({ SpoolSetMinMaxMaxSpeed: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveTensionTarget } = useMachineMutation(
    z.object({ SpoolSetAdaptiveTensionTarget: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveRadiusLearningRate } =
    useMachineMutation(
      z.object({ SpoolSetAdaptiveRadiusLearningRate: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveMaxSpeedMultiplier } =
    useMachineMutation(
      z.object({ SpoolSetAdaptiveMaxSpeedMultiplier: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveAccelerationFactor } =
    useMachineMutation(
      z.object({ SpoolSetAdaptiveAccelerationFactor: z.number() }),
    );
  const { request: requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier } =
    useMachineMutation(
      z.object({ SpoolSetAdaptiveDeaccelerationUrgencyMultiplier: z.number() }),
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
  const tensionArmAngleZero = () => {
    updateStateOptimistically(
      (current) => {
        current.data.tension_arm_state.zeroed = true;
      },
      () =>
        requestTensionArmZero({
          machine_identification_unique: machineIdentification,
          data: "TensionArmAngleZero",
        }),
    );
  };

  const traverseSetLimitInner = (limitInner: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_inner = limitInner;
      },
      () =>
        requestTraverseSetLimitInner({
          machine_identification_unique: machineIdentification,
          data: { TraverseSetLimitInner: limitInner },
        }),
    );
  };

  const traverseSetLimitOuter = (limitOuter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_outer = limitOuter;
      },
      () =>
        requestTraverseSetLimitOuter({
          machine_identification_unique: machineIdentification,
          data: { TraverseSetLimitOuter: limitOuter },
        }),
    );
  };

  const traverseGotoLimitInner = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_in = true;
      },
      () =>
        requestTraverseGotoLimitInner({
          machine_identification_unique: machineIdentification,
          data: "TraverseGotoLimitInner",
        }),
    );
  };

  const traverseGotoLimitOuter = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_out = true;
      },
      () =>
        requestTraverseGotoLimitOuter({
          machine_identification_unique: machineIdentification,
          data: "TraverseGotoLimitOuter",
        }),
    );
  };

  const traverseGotoHome = () => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.is_going_home = true;
      },
      () =>
        requestTraverseGotoHome({
          machine_identification_unique: machineIdentification,
          data: "TraverseGotoHome",
        }),
    );
  };

  const setLaserpointer = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.laserpointer = enabled;
      },
      () =>
        requestSetLaserpointer({
          machine_identification_unique: machineIdentification,
          data: { TraverseEnableLaserpointer: enabled },
        }),
    );
  };

  const modeSet = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () =>
        requestModeSet({
          machine_identification_unique: machineIdentification,
          data: { ModeSet: mode },
        }),
    );
  };

  const traverseSetStepSize = (stepSize: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.step_size = stepSize;
      },
      () =>
        requestTraverseSetStepSize({
          machine_identification_unique: machineIdentification,
          data: { TraverseSetStepSize: stepSize },
        }),
    );
  };

  const traverseSetPadding = (padding: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.padding = padding;
      },
      () =>
        requestTraverseSetPadding({
          machine_identification_unique: machineIdentification,
          data: { TraverseSetPadding: padding },
        }),
    );
  };

  const pullerSetTargetSpeed = (targetSpeed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_speed = targetSpeed;
      },
      () =>
        requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { PullerSetTargetSpeed: targetSpeed },
        }),
    );
  };

  const pullerSetTargetDiameter = (targetDiameter: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_diameter = targetDiameter;
      },
      () =>
        requestPullerSetTargetDiameter({
          machine_identification_unique: machineIdentification,
          data: { PullerSetTargetDiameter: targetDiameter },
        }),
    );
  };

  const pullerSetRegulationMode = (regulationMode: PullerRegulation) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.regulation = regulationMode;
      },
      () =>
        requestPullerSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { PullerSetRegulationMode: regulationMode },
        }),
    );
  };

  const pullerSetForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.forward = forward;
      },
      () =>
        requestPullerSetForward({
          machine_identification_unique: machineIdentification,
          data: { PullerSetForward: forward },
        }),
    );
  };

  const spoolSetRegulationMode = (mode: SpoolRegulationMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.regulation_mode = mode;
      },
      () =>
        requestSpoolSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetRegulationMode: mode },
        }),
    );
  };

  const spoolSetMinMaxMinSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.minmax_min_speed = speed;
      },
      () =>
        requestSpoolSetMinMaxMinSpeed({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetMinMaxMinSpeed: speed },
        }),
    );
  };

  const spoolSetMinMaxMaxSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.minmax_max_speed = speed;
      },
      () =>
        requestSpoolSetMinMaxMaxSpeed({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetMinMaxMaxSpeed: speed },
        }),
    );
  };

  const spoolSetAdaptiveTensionTarget = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_tension_target =
          value;
      },
      () =>
        requestSpoolSetAdaptiveTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetAdaptiveTensionTarget: value },
        }),
    );
  };

  const spoolSetAdaptiveRadiusLearningRate = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_radius_learning_rate =
          value;
      },
      () =>
        requestSpoolSetAdaptiveRadiusLearningRate({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetAdaptiveRadiusLearningRate: value },
        }),
    );
  };

  const spoolSetAdaptiveMaxSpeedMultiplier = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_max_speed_multiplier =
          value;
      },
      () =>
        requestSpoolSetAdaptiveMaxSpeedMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetAdaptiveMaxSpeedMultiplier: value },
        }),
    );
  };

  const spoolSetAdaptiveAccelerationFactor = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_acceleration_factor =
          value;
      },
      () =>
        requestSpoolSetAdaptiveAccelerationFactor({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetAdaptiveAccelerationFactor: value },
        }),
    );
  };

  const spoolSetAdaptiveDeaccelerationUrgencyMultiplier = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_speed_controller_state.adaptive_deacceleration_urgency_multiplier =
          value;
      },
      () =>
        requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SpoolSetAdaptiveDeaccelerationUrgencyMultiplier: value },
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
    setLaserpointer,
    ExtruderSetMode: modeSet,
    tensionArmAngleZero,
    traverseSetLimitInner,
    traverseSetLimitOuter,
    traverseGotoLimitInner,
    traverseGotoLimitOuter,
    traverseGotoHome,
    traverseSetStepSize,
    traverseSetPadding,
    pullerSetTargetSpeed,
    pullerSetTargetDiameter,
    pullerSetRegulationMode,
    pullerSetForward,
    spoolSetRegulationMode,
    spoolSetMinMaxMinSpeed,
    spoolSetMinMaxMaxSpeed,
    spoolSetAdaptiveTensionTarget,
    spoolSetAdaptiveRadiusLearningRate,
    spoolSetAdaptiveMaxSpeedMultiplier,
    spoolSetAdaptiveAccelerationFactor,
    spoolSetAdaptiveDeaccelerationUrgencyMultiplier,
  };
}
