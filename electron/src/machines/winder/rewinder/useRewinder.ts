import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { rewinder } from "@/machines/properties";
import { rewinderSerialRoute } from "@/routes/routes";
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
  const { request: requestTakeupSpoolSetDiameter } = useMachineMutation(
    z.object({ SetTakeupSpoolDiameter: z.number() }),
  );
  const { request: requestSourceSpoolSetDiameter } = useMachineMutation(
    z.object({ SetSourceSpoolDiameter: z.number() }),
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
  const { request: requestHardStop } = useMachineMutation(
    z.literal("HardStop"),
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
  const { request: requestTraverseSetStartPosition } = useMachineMutation(
    z.object({ SetTraverseStartPosition: z.number() }),
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
  const { request: requestTraverseGotoStartPosition } = useMachineMutation(
    z.literal("GotoTraverseStartPosition"),
  );
  const { request: requestEnableTraverseLaserpointer } = useMachineMutation(
    z.object({ EnableTraverseLaserpointer: z.boolean() }),
  );

  const updateStateOptimistically = (
    _producer: (current: StateEvent) => void,
    serverRequest: () => void,
  ) => {
    serverRequest();
  };

  const currentMode = stateOptimistic.value?.data.mode_state.mode;
  const isDecelerating =
    stateOptimistic.value?.data.mode_state.is_decelerating === true;
  const settingsEditPermitted =
    !isDecelerating && (currentMode === "Standby" || currentMode === "Hold");
  const prepareSettingsEditPermitted =
    !isDecelerating && (currentMode === "Standby" || currentMode === "Hold");
  const progressResetPermitted =
    !isDecelerating &&
    (currentMode === "Standby" ||
      currentMode === "Hold" ||
      currentMode === "Rewind");
  const manualTraversePermitted =
    !isDecelerating &&
    currentMode === "Hold" &&
    stateOptimistic.value?.data.traverse_state.is_homed === true;

  const setMode = (mode: Mode) => {
    if (isDecelerating) {
      if (mode === "Hold" || mode === "Standby") {
        void requestModeSet({
          machine_identification_unique: machineIdentification,
          data: { SetMode: mode },
        });
      }
      return;
    }

    if (mode === currentMode) {
      return;
    }

    if (
      mode === "Rewind" &&
      stateOptimistic.value?.data.mode_state.can_rewind !== true
    ) {
      return;
    }

    if (
      mode === "Prepare" &&
      (stateOptimistic.value?.data.takeup_tension_arm_state.zeroed !== true ||
        stateOptimistic.value?.data.source_tension_arm_state.zeroed !== true)
    ) {
      return;
    }

    void requestModeSet({
      machine_identification_unique: machineIdentification,
      data: { SetMode: mode },
    });
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    if (isDecelerating || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_speed = targetSpeed;
      },
      () =>
        void requestPullerSetTargetSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetPullerTargetSpeed: targetSpeed },
        }),
    );
  };

  const setTakeupSpoolRegulationMode = (mode: SpoolRegulationMode) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.regulation_mode = mode;
      },
      () =>
        void requestTakeupSpoolSetRegulationMode({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolRegulationMode: mode },
        }),
    );
  };

  const setTakeupSpoolMinMaxMinSpeed = (speed: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.minmax_min_speed = speed;
      },
      () =>
        void requestTakeupSpoolSetMinMaxMinSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolMinMaxMinSpeed: speed },
        }),
    );
  };

  const setTakeupSpoolMinMaxMaxSpeed = (speed: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.minmax_max_speed = speed;
      },
      () =>
        void requestTakeupSpoolSetMinMaxMaxSpeed({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolMinMaxMaxSpeed: speed },
        }),
    );
  };

  const setTakeupTensionTarget = (target: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_tension_target = target;
      },
      () =>
        void requestTakeupTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupTensionTarget: target },
        }),
    );
  };

  const setTakeupSpoolAdaptiveRadiusLearningRate = (value: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_radius_learning_rate = value;
      },
      () =>
        void requestTakeupSpoolSetAdaptiveRadiusLearningRate({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveRadiusLearningRate: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveMaxSpeedMultiplier = (value: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_max_speed_multiplier = value;
      },
      () =>
        void requestTakeupSpoolSetAdaptiveMaxSpeedMultiplier({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveMaxSpeedMultiplier: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveAccelerationFactor = (value: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_acceleration_factor = value;
      },
      () =>
        void requestTakeupSpoolSetAdaptiveAccelerationFactor({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolAdaptiveAccelerationFactor: value },
        }),
    );
  };

  const setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier = (
    value: number,
  ) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.adaptive_deacceleration_urgency_multiplier =
          value;
      },
      () =>
        void requestTakeupSpoolSetAdaptiveDeaccelerationUrgencyMultiplier({
          machine_identification_unique: machineIdentification,
          data: {
            SetTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier: value,
          },
        }),
    );
  };

  const setTakeupSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.diameter_mm = diameterMm;
      },
      () =>
        void requestTakeupSpoolSetDiameter({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupSpoolDiameter: diameterMm },
        }),
    );
  };

  const setSourceSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.source_spool_state.diameter_mm = diameterMm;
      },
      () =>
        void requestSourceSpoolSetDiameter({
          machine_identification_unique: machineIdentification,
          data: { SetSourceSpoolDiameter: diameterMm },
        }),
    );
  };

  const setSourceTensionTarget = (target: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.source_spool_state.adaptive_tension_target = target;
      },
      () =>
        void requestSourceTensionTarget({
          machine_identification_unique: machineIdentification,
          data: { SetSourceTensionTarget: target },
        }),
    );
  };

  const setTakeupTensionArmControl = (
    field: keyof StateEvent["data"]["takeup_tension_arm_control_state"],
    value: number,
  ) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

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
        void requestSetTakeupTensionArmControl({
          machine_identification_unique: machineIdentification,
          data: { SetTakeupTensionArmControl: next },
        }),
    );
  };

  const setSourceTensionArmControl = (
    field: keyof StateEvent["data"]["source_tension_arm_control_state"],
    value: number,
  ) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

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
        void requestSetSourceTensionArmControl({
          machine_identification_unique: machineIdentification,
          data: { SetSourceTensionArmControl: next },
        }),
    );
  };

  const setPrepareControl = (
    field: keyof StateEvent["data"]["prepare_control_state"],
    value: number,
  ) => {
    if (!prepareSettingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

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
        void requestSetPrepareControl({
          machine_identification_unique: machineIdentification,
          data: { SetPrepareControl: next },
        }),
    );
  };

  const setRewindAutomaticRequiredMeters = (meters: number) => {
    if (isDecelerating || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.rewind_automatic_action_state.required_meters = meters;
      },
      () =>
        void requestSetRewindAutomaticRequiredMeters({
          machine_identification_unique: machineIdentification,
          data: { SetRewindAutomaticRequiredMeters: meters },
        }),
    );
  };

  const setRewindAutomaticAction = (mode: RewindAutomaticActionMode) => {
    if (isDecelerating) {
      return;
    }

    if (
      mode === stateOptimistic.value?.data.rewind_automatic_action_state.mode
    ) {
      return;
    }

    void requestSetRewindAutomaticAction({
      machine_identification_unique: machineIdentification,
      data: { SetRewindAutomaticAction: mode },
    });
  };

  const resetRewindProgress = () => {
    if (!progressResetPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    requestResetRewindProgress({
      machine_identification_unique: machineIdentification,
      data: "ResetRewindProgress",
    });
  };

  const hardStop = () => {
    if (
      (!isDecelerating && currentMode !== "Rewind") ||
      stateOptimistic.isOptimistic
    ) {
      return;
    }

    requestHardStop({
      machine_identification_unique: machineIdentification,
      data: "HardStop",
    });
  };

  const zeroTakeupTensionArm = () => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_tension_arm_state.zeroed = true;
      },
      () =>
        void requestZeroTakeupTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroTakeupTensionArm",
        }),
    );
  };

  const zeroSourceTensionArm = () => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.source_tension_arm_state.zeroed = true;
      },
      () =>
        void requestZeroSourceTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroSourceTensionArm",
        }),
    );
  };

  const setTraverseLimitInner = (limit: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_inner = limit;
      },
      () =>
        void requestTraverseSetLimitInner({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitInner: limit },
        }),
    );
  };

  const setTraverseLimitOuter = (limit: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_outer = limit;
      },
      () =>
        void requestTraverseSetLimitOuter({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseLimitOuter: limit },
        }),
    );
  };

  const setTraverseStartPosition = (position: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.start_position = position;
      },
      () =>
        void requestTraverseSetStartPosition({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseStartPosition: position },
        }),
    );
  };

  const setTraverseStepSize = (stepSize: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.step_size = stepSize;
      },
      () =>
        void requestTraverseSetStepSize({
          machine_identification_unique: machineIdentification,
          data: { SetTraverseStepSize: stepSize },
        }),
    );
  };

  const setTraversePadding = (padding: number) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.padding = padding;
      },
      () =>
        void requestTraverseSetPadding({
          machine_identification_unique: machineIdentification,
          data: { SetTraversePadding: padding },
        }),
    );
  };

  const gotoTraverseHome = () => {
    if (
      isDecelerating ||
      currentMode !== "Hold" ||
      stateOptimistic.isOptimistic
    ) {
      return;
    }

    requestTraverseGotoHome({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseHome",
    });
  };
  const gotoTraverseLimitInner = () => {
    if (!manualTraversePermitted || stateOptimistic.isOptimistic) {
      return;
    }

    requestTraverseGotoLimitInner({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseLimitInner",
    });
  };
  const gotoTraverseLimitOuter = () => {
    if (!manualTraversePermitted || stateOptimistic.isOptimistic) {
      return;
    }

    requestTraverseGotoLimitOuter({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseLimitOuter",
    });
  };
  const gotoTraverseStartPosition = () => {
    if (!manualTraversePermitted || stateOptimistic.isOptimistic) {
      return;
    }

    requestTraverseGotoStartPosition({
      machine_identification_unique: machineIdentification,
      data: "GotoTraverseStartPosition",
    });
  };

  const enableTraverseLaserpointer = (enabled: boolean) => {
    if (!settingsEditPermitted || stateOptimistic.isOptimistic) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.laserpointer = enabled;
      },
      () =>
        void requestEnableTraverseLaserpointer({
          machine_identification_unique: machineIdentification,
          data: { EnableTraverseLaserpointer: enabled },
        }),
    );
  };

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
    isDecelerating,
    settingsEditPermitted,
    prepareSettingsEditPermitted,
    progressResetPermitted,
    manualTraversePermitted,
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
    setTakeupSpoolDiameter,
    setSourceSpoolDiameter,
    setSourceTensionTarget,
    setTakeupTensionArmControl,
    setSourceTensionArmControl,
    setPrepareControl,
    hardStop,
    setRewindAutomaticRequiredMeters,
    setRewindAutomaticAction,
    resetRewindProgress,
    zeroTakeupTensionArm,
    zeroSourceTensionArm,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    setTraverseStartPosition,
    setTraverseStepSize,
    setTraversePadding,
    gotoTraverseHome,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseStartPosition,
    enableTraverseLaserpointer,
  };
}
