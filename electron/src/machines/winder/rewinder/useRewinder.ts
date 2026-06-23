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
  Mode,
  RewindAutomaticActionMode,
  SpoolRegulationMode,
  StateEvent,
  modeSchema,
  prepareControlStateSchema,
  rewindAutomaticActionModeSchema,
  spoolRegulationModeSchema,
  tensionArmControlStateSchema,
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
    rewindProgress,
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
  const { request: requestSetTakeupTensionArmControl } = useMachineMutation(
    z.object({ SetTakeupTensionArmControl: tensionArmControlStateSchema }),
  );
  const { request: requestSetSourceTensionArmControl } = useMachineMutation(
    z.object({ SetSourceTensionArmControl: tensionArmControlStateSchema }),
  );
  const { request: requestSetPrepareControl } = useMachineMutation(
    z.object({ SetPrepareControl: prepareControlStateSchema }),
  );
  const { request: requestSetRewindAutomaticRequiredMeters } =
    useMachineMutation(
      z.object({ SetRewindAutomaticRequiredMeters: z.number() }),
    );
  const { request: requestSetRewindAutomaticAction } = useMachineMutation(
    z.object({ SetRewindAutomaticAction: rewindAutomaticActionModeSchema }),
  );
  const { request: requestResetRewindProgress } = useMachineMutation(
    z.literal("ResetRewindProgress"),
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

  const setTakeupTensionArmControl = (
    field: keyof StateEvent["data"]["takeup_tension_arm_control_state"],
    value: number,
  ) => {
    const currentConfig =
      stateOptimistic.value?.data.takeup_tension_arm_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };
    updateStateOptimistically(
      (current) => {
        current.data.takeup_tension_arm_control_state = next;
      },
      () =>
        requestSetTakeupTensionArmControl({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupTensionArmControl: next },
        }),
    );
  };

  const setSourceTensionArmControl = (
    field: keyof StateEvent["data"]["source_tension_arm_control_state"],
    value: number,
  ) => {
    const currentConfig =
      stateOptimistic.value?.data.source_tension_arm_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };
    updateStateOptimistically(
      (current) => {
        current.data.source_tension_arm_control_state = next;
      },
      () =>
        requestSetSourceTensionArmControl({
          machine_identification_unique: machineIdentification,
          data: { SetSourceTensionArmControl: next },
        }),
    );
  };

  const setPrepareControl = (
    field: keyof StateEvent["data"]["prepare_control_state"],
    value: number,
  ) => {
    const currentConfig = stateOptimistic.value?.data.prepare_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };
    updateStateOptimistically(
      (current) => {
        current.data.prepare_control_state = next;
      },
      () =>
        requestSetPrepareControl({
          machine_identification_unique: machineIdentification,
          data: { SetPrepareControl: next },
        }),
    );
  };

  const setRewindAutomaticRequiredMeters = (meters: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.rewind_automatic_action_state.required_meters = meters;
      },
      () =>
        requestSetRewindAutomaticRequiredMeters({
          machine_identification_unique: machineIdentification,
          data: { SetRewindAutomaticRequiredMeters: meters },
        }),
    );
  };

  const setRewindAutomaticAction = (mode: RewindAutomaticActionMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.rewind_automatic_action_state.mode = mode;
      },
      () =>
        requestSetRewindAutomaticAction({
          machine_identification_unique: machineIdentification,
          data: { SetRewindAutomaticAction: mode },
        }),
    );
  };

  const resetRewindProgress = () =>
    requestResetRewindProgress({
      machine_identification_unique: machineIdentification,
      data: "ResetRewindProgress",
    });

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
    rewindProgress,
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: false,
    setMode,
    setPullerTargetSpeed,
    setTakeupSpoolRegulationMode,
    setTakeupSpoolMinMaxMinSpeed,
    setTakeupSpoolMinMaxMaxSpeed,
    setTakeupTensionTarget,
    setTakeupSpoolAdaptiveRadiusLearningRate,
    setTakeupSpoolAdaptiveMaxSpeedMultiplier,
    setTakeupSpoolAdaptiveAccelerationFactor,
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    setSourceTensionTarget,
    setTakeupTensionArmControl,
    setSourceTensionArmControl,
    setPrepareControl,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    resetRewindProgress,
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
