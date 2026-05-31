import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { rewinder } from "@/machines/properties";
import { rewinderSerialRoute } from "@/routes/routes";
import { produce } from "immer";
import { useEffect, useMemo } from "react";
import { z } from "zod";
import {
  GearRatio,
  Mode,
  SpoolRegulationMode,
  StateEvent,
  gearRatioSchema,
  modeSchema,
  spoolRegulationModeSchema,
  useRewinderNamespace,
} from "./rewinderNamespace";

export function useRewinder() {
  const { serial: serialString } = rewinderSerialRoute.useParams();

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
      machine_identification: rewinder.machine_identification,
      serial,
    };
  }, [serialString]);

  const {
    state,
    defaultState,
    traversePosition,
    pullerSpeed,
    takeupSpoolRpm,
    sourceSpoolRpm,
    takeupTensionArmAngle,
    sourceTensionArmAngle,
  } = useRewinderNamespace(machineIdentification);

  const stateOptimistic = useStateOptimistic<StateEvent>();

  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  const { request: requestModeSet } = useMachineMutation(
    z.object({ SetMode: modeSchema }),
  );
  const { request: requestPullerSetTargetSpeed } = useMachineMutation(
    z.object({ SetPullerTargetSpeed: z.number() }),
  );
  const { request: requestPullerSetGearRatio } = useMachineMutation(
    z.object({ SetPullerGearRatio: gearRatioSchema }),
  );
  const { request: requestTakeupSpoolSetRegulationMode } = useMachineMutation(
    z.object({ SetTakeupSpoolRegulationMode: spoolRegulationModeSchema }),
  );
  const { request: requestTakeupSpoolSetMinMaxMinSpeed } = useMachineMutation(
    z.object({ SetTakeupSpoolMinMaxMinSpeed: z.number() }),
  );
  const { request: requestTakeupSpoolSetMinMaxMaxSpeed } = useMachineMutation(
    z.object({ SetTakeupSpoolMinMaxMaxSpeed: z.number() }),
  );
  const { request: requestTakeupTensionTarget } = useMachineMutation(
    z.object({ SetTakeupTensionTarget: z.number() }),
  );
  const { request: requestTakeupSpoolSetAdaptiveRadiusLearningRate } =
    useMachineMutation(
      z.object({ SetTakeupSpoolAdaptiveRadiusLearningRate: z.number() }),
    );
  const { request: requestTakeupSpoolSetAdaptiveMaxSpeedMultiplier } =
    useMachineMutation(
      z.object({ SetTakeupSpoolAdaptiveMaxSpeedMultiplier: z.number() }),
    );
  const { request: requestTakeupSpoolSetAdaptiveAccelerationFactor } =
    useMachineMutation(
      z.object({ SetTakeupSpoolAdaptiveAccelerationFactor: z.number() }),
    );
  const {
    request: requestTakeupSpoolSetAdaptiveDeaccelerationUrgencyMultiplier,
  } = useMachineMutation(
    z.object({
      SetTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier: z.number(),
    }),
  );
  const { request: requestSourceTensionTarget } = useMachineMutation(
    z.object({ SetSourceTensionTarget: z.number() }),
  );
  const { request: requestZeroTakeupTensionArm } = useMachineMutation(
    z.literal("ZeroTakeupTensionArm"),
  );
  const { request: requestZeroSourceTensionArm } = useMachineMutation(
    z.literal("ZeroSourceTensionArm"),
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
  const { request: requestTraverseGotoHome } = useMachineMutation(
    z.literal("GotoTraverseHome"),
  );
  const { request: requestTraverseGotoLimitInner } = useMachineMutation(
    z.literal("GotoTraverseLimitInner"),
  );
  const { request: requestTraverseGotoLimitOuter } = useMachineMutation(
    z.literal("GotoTraverseLimitOuter"),
  );

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => void | Promise<void>,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  const setMode = (mode: Mode) => {
    if (
      mode === "Rewind" &&
      stateOptimistic.value?.data.mode_state.can_rewind !== true
    ) {
      return;
    }

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

  const setPullerGearRatio = (gearRatio: GearRatio) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.gear_ratio = gearRatio;
        current.data.puller_state.target_speed = 0;
      },
      async () => {
        await requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetSpeed: 0 },
        });
        await requestPullerSetGearRatio({
          machine_identification_unique: machineIdentification,
          data: { SetPullerGearRatio: gearRatio },
        });
      },
    );
  };

  const setTakeupSpoolRegulationMode = (mode: SpoolRegulationMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.regulation_mode = mode;
      },
      () =>
        requestTakeupSpoolSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolRegulationMode: mode },
        }),
    );
  };

  const setTakeupSpoolMinMaxMinSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.minmax_min_speed = speed;
      },
      () =>
        requestTakeupSpoolSetMinMaxMinSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolMinMaxMinSpeed: speed },
        }),
    );
  };

  const setTakeupSpoolMinMaxMaxSpeed = (speed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.minmax_max_speed = speed;
      },
      () =>
        requestTakeupSpoolSetMinMaxMaxSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolMinMaxMaxSpeed: speed },
        }),
    );
  };

  const setTakeupTensionTarget = (target: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_tension_target = target;
      },
      () =>
        requestTakeupTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupTensionTarget: target },
        }),
    );
  };

  const setTakeupSpoolAdaptiveRadiusLearningRate = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_radius_learning_rate = value;
      },
      () =>
        requestTakeupSpoolSetAdaptiveRadiusLearningRate({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveRadiusLearningRate: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveMaxSpeedMultiplier = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_max_speed_multiplier = value;
      },
      () =>
        requestTakeupSpoolSetAdaptiveMaxSpeedMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveMaxSpeedMultiplier: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveAccelerationFactor = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_acceleration_factor = value;
      },
      () =>
        requestTakeupSpoolSetAdaptiveAccelerationFactor({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveAccelerationFactor: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier = (
    value: number,
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_deacceleration_urgency_multiplier =
          value;
      },
      () =>
        requestTakeupSpoolSetAdaptiveDeaccelerationUrgencyMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier: value },
        }),
    );
  };

  const setSourceTensionTarget = (target: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.source_spool_state.adaptive_tension_target = target;
      },
      () =>
        requestSourceTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SetSourceTensionTarget: target },
        }),
    );
  };

  const zeroTakeupTensionArm = () => {
    updateStateOptimistically(
      (current) => {
        current.data.takeup_tension_arm_state.zeroed = true;
      },
      () =>
        requestZeroTakeupTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroTakeupTensionArm",
        }),
    );
  };

  const zeroSourceTensionArm = () => {
    updateStateOptimistically(
      (current) => {
        current.data.source_tension_arm_state.zeroed = true;
      },
      () =>
        requestZeroSourceTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroSourceTensionArm",
        }),
    );
  };

  const setTraverseLimitInner = (limit: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_inner = limit;
      },
      () =>
        requestTraverseSetLimitInner({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitInner: limit },
        }),
    );
  };

  const setTraverseLimitOuter = (limit: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_outer = limit;
      },
      () =>
        requestTraverseSetLimitOuter({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitOuter: limit },
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

  const gotoTraverseHome = () =>
    requestTraverseGotoHome({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseHome",
    });
  const gotoTraverseLimitInner = () =>
    requestTraverseGotoLimitInner({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseLimitInner",
    });
  const gotoTraverseLimitOuter = () =>
    requestTraverseGotoLimitOuter({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseLimitOuter",
    });

  return {
    state: stateOptimistic.value?.data,
    defaultState: defaultState?.data,
    traversePosition,
    pullerSpeed,
    takeupSpoolRpm,
    sourceSpoolRpm,
    takeupTensionArmAngle,
    sourceTensionArmAngle,
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: false,
    setMode,
    setPullerTargetSpeed,
    setPullerGearRatio,
    setTakeupSpoolRegulationMode,
    setTakeupSpoolMinMaxMinSpeed,
    setTakeupSpoolMinMaxMaxSpeed,
    setTakeupTensionTarget,
    setTakeupSpoolAdaptiveRadiusLearningRate,
    setTakeupSpoolAdaptiveMaxSpeedMultiplier,
    setTakeupSpoolAdaptiveAccelerationFactor,
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setSourceTensionTarget,
    zeroTakeupTensionArm,
    zeroSourceTensionArm,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
  };
}
