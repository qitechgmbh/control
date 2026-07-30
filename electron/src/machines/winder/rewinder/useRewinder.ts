import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { toastError } from "@/components/Toast";
import { MachineIdentificationUnique } from "@/machines/types";
import { rewinder } from "@/machines/properties";
import { rewinderSerialRoute } from "@/routes/routes";
import { useMemo } from "react";
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

  const withMachine = <T>(data: T) => ({
    machine_identification_unique: machineIdentification,
    data,
  });

  const stateData = state?.data;
  const currentMode = stateData?.mode_state.mode;
  const motionStopped = stateData?.mode_state.motion_stopped !== false;
  const settingsEditPermitted =
    motionStopped && (currentMode === "Standby" || currentMode === "Hold");
  const prepareSettingsEditPermitted =
    motionStopped && (currentMode === "Standby" || currentMode === "Hold");
  const progressResetPermitted =
    currentMode === "Standby" ||
    currentMode === "Hold" ||
    currentMode === "Rewind";
  const manualTraversePermitted =
    currentMode === "Hold" &&
    motionStopped &&
    stateData?.traverse_state.is_homed === true;

  const setMode = (mode: Mode) => {
    if (mode === currentMode) {
      return;
    }

    if (mode === "Rewind" && stateData?.mode_state.can_rewind !== true) {
      return;
    }

    if (
      mode === "Prepare" &&
      (stateData?.takeup_tension_arm_state.zeroed !== true ||
        stateData?.source_tension_arm_state.zeroed !== true)
    ) {
      return;
    }

    if (currentMode === "Rewind" && mode !== "Hold" && mode !== "Standby") {
      return;
    }

    void requestModeSet(withMachine({ SetMode: mode }));
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    void requestPullerSetTargetSpeed(
      withMachine({ SetPullerTargetSpeed: targetSpeed }),
    );
  };

  const setTakeupSpoolRegulationMode = (mode: SpoolRegulationMode) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetRegulationMode(
      withMachine({ SetTakeupSpoolRegulationMode: mode }),
    );
  };

  const setTakeupSpoolMinMaxMinSpeed = (speed: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetMinMaxMinSpeed(
      withMachine({ SetTakeupSpoolMinMaxMinSpeed: speed }),
    );
  };

  const setTakeupSpoolMinMaxMaxSpeed = (speed: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetMinMaxMaxSpeed(
      withMachine({ SetTakeupSpoolMinMaxMaxSpeed: speed }),
    );
  };

  const setTakeupTensionTarget = (target: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupTensionTarget(
      withMachine({ SetTakeupTensionTarget: target }),
    );
  };

  const setTakeupSpoolAdaptiveRadiusLearningRate = (value: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetAdaptiveRadiusLearningRate(
      withMachine({ SetTakeupSpoolAdaptiveRadiusLearningRate: value }),
    );
  };

  const setTakeupSpoolAdaptiveMaxSpeedMultiplier = (value: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetAdaptiveMaxSpeedMultiplier(
      withMachine({ SetTakeupSpoolAdaptiveMaxSpeedMultiplier: value }),
    );
  };

  const setTakeupSpoolAdaptiveAccelerationFactor = (value: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetAdaptiveAccelerationFactor(
      withMachine({ SetTakeupSpoolAdaptiveAccelerationFactor: value }),
    );
  };

  const setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier = (
    value: number,
  ) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetAdaptiveDeaccelerationUrgencyMultiplier(
      withMachine({
        SetTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier: value,
      }),
    );
  };

  const setTakeupSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTakeupSpoolSetDiameter(
      withMachine({ SetTakeupSpoolDiameter: diameterMm }),
    );
  };

  const setSourceSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestSourceSpoolSetDiameter(
      withMachine({ SetSourceSpoolDiameter: diameterMm }),
    );
  };

  const setSourceTensionTarget = (target: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestSourceTensionTarget(
      withMachine({ SetSourceTensionTarget: target }),
    );
  };

  const setTakeupTensionArmControl = (
    field: keyof StateEvent["data"]["takeup_tension_arm_control_state"],
    value: number,
  ) => {
    if (!settingsEditPermitted) {
      return;
    }

    const currentConfig = stateData?.takeup_tension_arm_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };

    void requestSetTakeupTensionArmControl(
      withMachine({ SetTakeupTensionArmControl: next }),
    );
  };

  const setSourceTensionArmControl = (
    field: keyof StateEvent["data"]["source_tension_arm_control_state"],
    value: number,
  ) => {
    if (!settingsEditPermitted) {
      return;
    }

    const currentConfig = stateData?.source_tension_arm_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };

    void requestSetSourceTensionArmControl(
      withMachine({ SetSourceTensionArmControl: next }),
    );
  };

  const setPrepareControl = (
    field: keyof StateEvent["data"]["prepare_control_state"],
    value: number,
  ) => {
    if (!prepareSettingsEditPermitted) {
      return;
    }

    const currentConfig = stateData?.prepare_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };

    void requestSetPrepareControl(withMachine({ SetPrepareControl: next }));
  };

  const setRewindAutomaticRequiredMeters = (meters: number) => {
    void requestSetRewindAutomaticRequiredMeters(
      withMachine({ SetRewindAutomaticRequiredMeters: meters }),
    );
  };

  const setRewindAutomaticAction = (mode: RewindAutomaticActionMode) => {
    if (mode === stateData?.rewind_automatic_action_state.mode) {
      return;
    }

    void requestSetRewindAutomaticAction(
      withMachine({ SetRewindAutomaticAction: mode }),
    );
  };

  const resetRewindProgress = () => {
    if (!progressResetPermitted) {
      return;
    }

    requestResetRewindProgress(withMachine("ResetRewindProgress"));
  };

  const hardStop = () => {
    if (currentMode !== "Rewind") {
      return;
    }

    requestHardStop(withMachine("HardStop"));
  };

  const zeroTakeupTensionArm = () => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestZeroTakeupTensionArm(withMachine("ZeroTakeupTensionArm"));
  };

  const zeroSourceTensionArm = () => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestZeroSourceTensionArm(withMachine("ZeroSourceTensionArm"));
  };

  const setTraverseLimitInner = (limit: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTraverseSetLimitInner(
      withMachine({ SetTraverseLimitInner: limit }),
    );
  };

  const setTraverseLimitOuter = (limit: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTraverseSetLimitOuter(
      withMachine({ SetTraverseLimitOuter: limit }),
    );
  };

  const setTraverseStartPosition = (position: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTraverseSetStartPosition(
      withMachine({ SetTraverseStartPosition: position }),
    );
  };

  const setTraverseStepSize = (stepSize: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTraverseSetStepSize(
      withMachine({ SetTraverseStepSize: stepSize }),
    );
  };

  const setTraversePadding = (padding: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestTraverseSetPadding(
      withMachine({ SetTraversePadding: padding }),
    );
  };

  const gotoTraverseHome = () => {
    if (currentMode !== "Hold" || !motionStopped) {
      return;
    }

    requestTraverseGotoHome(withMachine("GotoTraverseHome"));
  };
  const gotoTraverseLimitInner = () => {
    if (!manualTraversePermitted) {
      return;
    }

    requestTraverseGotoLimitInner(withMachine("GotoTraverseLimitInner"));
  };
  const gotoTraverseLimitOuter = () => {
    if (!manualTraversePermitted) {
      return;
    }

    requestTraverseGotoLimitOuter(withMachine("GotoTraverseLimitOuter"));
  };
  const gotoTraverseStartPosition = () => {
    if (!manualTraversePermitted) {
      return;
    }

    requestTraverseGotoStartPosition(withMachine("GotoTraverseStartPosition"));
  };

  const enableTraverseLaserpointer = (enabled: boolean) => {
    if (!settingsEditPermitted) {
      return;
    }

    void requestEnableTraverseLaserpointer(
      withMachine({ EnableTraverseLaserpointer: enabled }),
    );
  };

  return {
    state: stateData,
    defaultState: defaultState?.data,
    traversePosition,
    pullerSpeed,
    takeupSpoolRpm,
    sourceSpoolRpm,
    takeupTensionArmAngle,
    sourceTensionArmAngle,
    rewindProgress,
    isLoading: false,
    isDisabled: false,
    motionStopped,
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
