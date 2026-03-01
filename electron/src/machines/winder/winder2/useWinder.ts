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
  SpoolSpeedControlMode,
  StateEvent,
  useWinder2Namespace,
  modeSchema,
  Mode,
  spoolSpeedControlModeSchema,
  OnSpoolLengthTaskCompletedAction,
  spoolLengthTaskCompletedAction,
  gearRatioSchema,
  GearRatio,
  pullerSpeedControlModeSchema,
  directionSchema,
  Direction,
  PullerSpeedControlMode
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
    spoolLengthTaskProgress,
  } = useWinder2Namespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<StateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // boilerplate junk ends here..

  // machine request functions
  const { request: requestModeSet } = useMachineMutation(
    z.object({ SetMode: modeSchema }),
  );

  // spool request functions
  const { request: requestSpoolSetDirection } = useMachineMutation(
    z.object({ SetSpoolDirection: directionSchema }),
  );
  const { request: requestSpoolSetSpeedControlMode } = useMachineMutation(
    z.object({ SetSpoolSpeedControlMode: spoolSpeedControlModeSchema }),
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
  const { request: requestSpoolSetAdaptiveRadiusLearningRate } = useMachineMutation(
    z.object({ SetSpoolAdaptiveRadiusLearningRate: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveMaxSpeedMultiplier } = useMachineMutation(
    z.object({ SetSpoolAdaptiveMaxSpeedMultiplier: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveAccelerationFactor } = useMachineMutation(
    z.object({ SetSpoolAdaptiveAccelerationFactor: z.number() }),
  );
  const { request: requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier } = useMachineMutation(
    z.object({ SetSpoolAdaptiveDeaccelerationUrgencyMultiplier: z.number() }),
  );

  // puller request functions
  const { request: requestPullerSetDirection } = useMachineMutation(
    z.object({ SetPullerDirection: directionSchema }),
  );
  const { request: requestPullerSetGearRatio } = useMachineMutation(
    z.object({ SetPullerGearRatio: gearRatioSchema }),
  );
  const { request: requestPullerSetSpeedControlMode } = useMachineMutation(
    z.object({
      SetPullerSpeedControlMode: pullerSpeedControlModeSchema,
    }),
  );
  const { request: requestPullerSetFixedTargetSpeed } = useMachineMutation(
    z.object({
      SetPullerFixedTargetSpeed: z.number(),
    }),
  );
  const { request: requestPullerSetAdaptiveBaseSpeed } = useMachineMutation(
    z.object({
      SetPullerAdaptiveBaseSpeed: z.number(),
    }),
  );
  const { request: requestPullerSetAdaptiveDeviationMax } = useMachineMutation(
    z.object({
      SetPullerAdaptiveDeviationMax: z.number(),
    }),
  );
  const { request: requestPullerSetAdaptiveReferenceMachine } = useMachineMutation(
    z.object({
      SetPullerAdaptiveReferenceMachine: machineIdentificationUnique.nullable(),
    }),
  );

  // traverse
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
  const { request: requestTraverseGotoLimitInner } = useMachineMutation(
    z.literal("GotoTraverseLimitInner"),
  );
  const { request: requestTraverseGotoLimitOuter } = useMachineMutation(
    z.literal("GotoTraverseLimitOuter"),
  );
  const { request: requestTraverseGotoHome } = useMachineMutation(
    z.literal("GotoTraverseHome"),
  );
  const { request: requestSetLaserpointerEnabled } = useMachineMutation(
    z.object({ SetTraverseLaserpointerEnabled: z.boolean() }),
  );

  // tension arm
  const { request: requestTensionArmCalibrate } = useMachineMutation(
    z.literal("CalibrateTensionArmAngle"),
  );

  // spool length task
  const { request: requestSpoolLengthTaskSetTargetLength } = useMachineMutation(
    z.object({ SetSpoolLengthTaskTargetLength: z.number() }),
  );
  const { request: requestSpoolLengthTaskResetProgress } = useMachineMutation(
    z.literal("ResetSetSpoolLengthTaskProgress"),
  );
  const { request: requestSpoolLengthTaskSetOnCompletedAction } = useMachineMutation(
    z.object({ SetOnSpoolLengthTaskCompletedAction: spoolLengthTaskCompletedAction }),
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
  const setMode = (mode: Mode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode = mode;
      },
      () =>
        requestModeSet({
          machine_identification_unique: machineIdentification,
          data: { SetMode: mode },
        }),
    );
  };

  // spool
  const setSpoolDirection = (direction: Direction) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_state.direction = direction;
      },
      () =>
        requestSpoolSetDirection({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolDirection: direction },
        }),
    );
  };

  const setSpoolSpeedControlMode = (mode: SpoolSpeedControlMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_state.speed_control_mode = mode;
      },
      () =>
        requestSpoolSetSpeedControlMode({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolSpeedControlMode: mode },
        }),
    );
  };

  const setSpoolMinMaxMinSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_state.minmax_min_speed = speed;
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
        current.data.spool_state.minmax_max_speed = speed;
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
        current.data.spool_state.adaptive_tension_target =
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
        current.data.spool_state.adaptive_radius_learning_rate =
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
        current.data.spool_state.adaptive_max_speed_multiplier =
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
        current.data.spool_state.adaptive_acceleration_factor =
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
        current.data.spool_state.adaptive_deacceleration_urgency_multiplier =
          value;
      },
      () =>
        requestSpoolSetAdaptiveDeaccelerationUrgencyMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolAdaptiveDeaccelerationUrgencyMultiplier: value },
        }),
    );
  };

  // puller
  const setPullerDirection = (direction: Direction) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.direction = direction;
      },
      () =>
        requestPullerSetDirection({
          machine_identification_unique: machineIdentification,
          data: { SetPullerDirection: direction },
        }),
    );
  };

  const setPullerGearRatio = (gearRatio: GearRatio) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.gear_ratio = gearRatio;
      },
      () => {
        requestPullerSetGearRatio({
          machine_identification_unique: machineIdentification,
          data: { SetPullerGearRatio: gearRatio },
        });
      },
    );
  };

  const setPullerSpeedControlMode = (mode: PullerSpeedControlMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.speed_control_mode = mode;
      },
      () =>
        requestPullerSetSpeedControlMode({
          machine_identification_unique: machineIdentification,
          data: { SetPullerSpeedControlMode: mode },
        }),
    );
  };

  const setPullerFixedTargetSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.fixed_target_speed = speed;
      },
      () =>
        requestPullerSetFixedTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerFixedTargetSpeed: speed },
        }),
    );
  };

  const setPullerAdaptiveBaseSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.fixed_target_speed = speed;
      },
      () =>
        requestPullerSetAdaptiveBaseSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerAdaptiveBaseSpeed: speed },
        }),
    );
  };

  const setPullerAdaptiveDeviationMax = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.fixed_target_speed = speed;
      },
      () =>
        requestPullerSetAdaptiveDeviationMax({
          machine_identification_unique: machineIdentification,
          data: { SetPullerAdaptiveDeviationMax: speed },
        }),
    );
  };

  const setPullerAdaptiveReferenceMachine = (machineUid: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  } | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state
          .adaptive_reference_machine
          .machine_identification_unique 
            = machineUid;
      },
      () =>
        requestPullerSetAdaptiveReferenceMachine({
          machine_identification_unique: machineIdentification,
          data: { SetPullerAdaptiveReferenceMachine: machineUid },
        }),
    );
  };

  // traverse
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

  const setTraverseLaserpointerEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.laserpointer_enabled = enabled;
      },
      () =>
        requestSetLaserpointerEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLaserpointerEnabled: enabled },
        }),
    );
  };

  // tension arm
  const calibrateTensionArmAngle = () => {
    updateStateOptimistically(
      (current) => {
        current.data.tension_arm_state.is_calibrated = true;
      },
      () =>
        requestTensionArmCalibrate({
          machine_identification_unique: machineIdentification,
          data: "CalibrateTensionArmAngle",
        }),
    );
  };

  // spool length task
  const setSpoolLengthTaskTargetLength = (target_length: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_length_task_state.target_length = target_length; 
      },
      () =>
        requestSpoolLengthTaskSetTargetLength({
          machine_identification_unique: machineIdentification,
          data: { SetSpoolLengthTaskTargetLength: target_length },
        }),
    );
  };

  const resetSpoolLengthTaskProgress = () => {
    requestSpoolLengthTaskResetProgress({
      machine_identification_unique: machineIdentification,
      data: "ResetSetSpoolLengthTaskProgress",
    });
  };

  const setOnSpoolLengthTaskCompletedAction = (action: OnSpoolLengthTaskCompletedAction) => {
    updateStateOptimistically(
      (current) => {
        current.data.spool_length_task_state.on_completed_action =
          action;
      },
      () =>
        requestSpoolLengthTaskSetOnCompletedAction({
          machine_identification_unique: machineIdentification,
          data: { SetOnSpoolLengthTaskCompletedAction: action },
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
            0x0006, // id of laser machine
      ),
    [machines, machineIdentification],
  );

  // Get selected machine by serial
  const selectedMachine = useMemo(() => {
    const serial =
      state?.data.puller_state.adaptive_reference_machine?.machine_identification_unique
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
    spoolProgress: spoolLengthTaskProgress,

    // Loading states
    isLoading,
    isDisabled,


    // Action functions
    //---------------------------------------------------------

    // machine
    setMode,

    // spool
    setSpoolDirection,
    setSpoolSpeedControlMode,

    // spool speed minmax strategy
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,

    // spool speed adaptive strategy
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,

    // puller
    setPullerDirection,
    setPullerGearRatio,
    setPullerSpeedControlMode,

    // puller speed fixed strategy
    setPullerFixedTargetSpeed,

    // puller speed adaptive strategy
    setPullerAdaptiveBaseSpeed,
    setPullerAdaptiveDeviationMax,
    setPullerAdaptiveReferenceMachine,

    // traverse
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseHome,
    setTraverseLaserpointerEnabled,

    // tension arm
    calibrateTensionArmAngle,

    // spool length task
    setSpoolLengthTaskTargetLength,
    resetSpoolLengthTaskProgress,
    setOnSpoolLengthTaskCompletedAction,
  };
}
