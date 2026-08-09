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
  StateEvent,
  TraverseStart,
  modeSchema,
  prepareControlStateSchema,
  rewindAutomaticActionModeSchema,
  tensionArmControlStateSchema,
  traverseStartSchema,
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
  const { request: requestTakeupSpoolSetDiameter } = useMachineMutation(
    z.object({ SetTakeupSpoolDiameter: z.number() }),
  );
  const { request: requestSourceSpoolSetDiameter } = useMachineMutation(
    z.object({ SetSourceSpoolDiameter: z.number() }),
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
  const { request: requestTraverseSetStart } = useMachineMutation(
    z.object({ SetTraverseStart: traverseStartSchema }),
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

  const updateStateOptimistically = (
    producer: (current: StateEvent) => void,
    serverRequest: () => unknown | Promise<unknown>,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    void serverRequest();
  };

  const stateData = stateOptimistic.value?.data;
  const currentMode = stateData?.mode_state.mode;
  const motionStopped = stateData?.mode_state.motion_stopped !== false;
  const settingsEditPermitted = stateData != null;
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

    updateStateOptimistically(
      (current) => {
        current.data.mode_state.mode = mode;
      },
      () => requestModeSet(withMachine({ SetMode: mode })),
    );
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.puller_state.target_speed = targetSpeed;
      },
      () =>
        requestPullerSetTargetSpeed(
          withMachine({ SetPullerTargetSpeed: targetSpeed }),
        ),
    );
  };

  const setTakeupSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_spool_state.diameter_mm = diameterMm;
      },
      () =>
        requestTakeupSpoolSetDiameter(
          withMachine({ SetTakeupSpoolDiameter: diameterMm }),
        ),
    );
  };

  const setSourceSpoolDiameter = (diameterMm: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.source_spool_state.diameter_mm = diameterMm;
      },
      () =>
        requestSourceSpoolSetDiameter(
          withMachine({ SetSourceSpoolDiameter: diameterMm }),
        ),
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

    updateStateOptimistically(
      (current) => {
        current.data.takeup_tension_arm_control_state = next;
      },
      () =>
        requestSetTakeupTensionArmControl(
          withMachine({ SetTakeupTensionArmControl: next }),
        ),
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

    updateStateOptimistically(
      (current) => {
        current.data.source_tension_arm_control_state = next;
      },
      () =>
        requestSetSourceTensionArmControl(
          withMachine({ SetSourceTensionArmControl: next }),
        ),
    );
  };

  const setPrepareControl = (
    field: keyof StateEvent["data"]["prepare_control_state"],
    value: number,
  ) => {
    if (!settingsEditPermitted) {
      return;
    }

    const currentConfig = stateData?.prepare_control_state;
    if (!currentConfig) return;
    const next = {
      ...currentConfig,
      [field]: value,
    };

    updateStateOptimistically(
      (current) => {
        current.data.prepare_control_state = next;
      },
      () => requestSetPrepareControl(withMachine({ SetPrepareControl: next })),
    );
  };

  const setRewindAutomaticRequiredMeters = (meters: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.rewind_automatic_action_state.required_meters = meters;
      },
      () =>
        requestSetRewindAutomaticRequiredMeters(
          withMachine({ SetRewindAutomaticRequiredMeters: meters }),
        ),
    );
  };

  const setRewindAutomaticAction = (mode: RewindAutomaticActionMode) => {
    if (mode === stateData?.rewind_automatic_action_state.mode) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.rewind_automatic_action_state.mode = mode;
      },
      () =>
        requestSetRewindAutomaticAction(
          withMachine({ SetRewindAutomaticAction: mode }),
        ),
    );
  };

  const resetRewindProgress = () => {
    if (!stateData) {
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
    if (
      !motionStopped ||
      (currentMode !== "Standby" && currentMode !== "Hold")
    ) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.takeup_tension_arm_state.zeroed = true;
      },
      () => requestZeroTakeupTensionArm(withMachine("ZeroTakeupTensionArm")),
    );
  };

  const zeroSourceTensionArm = () => {
    if (
      !motionStopped ||
      (currentMode !== "Standby" && currentMode !== "Hold")
    ) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.source_tension_arm_state.zeroed = true;
      },
      () => requestZeroSourceTensionArm(withMachine("ZeroSourceTensionArm")),
    );
  };

  const setTraverseLimitInner = (limit: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_inner = limit;
      },
      () =>
        requestTraverseSetLimitInner(
          withMachine({ SetTraverseLimitInner: limit }),
        ),
    );
  };

  const setTraverseLimitOuter = (limit: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.limit_outer = limit;
      },
      () =>
        requestTraverseSetLimitOuter(
          withMachine({ SetTraverseLimitOuter: limit }),
        ),
    );
  };

  const setTraverseStart = (start: TraverseStart) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.start = start;
        current.data.traverse_state.start_position =
          start === "Left"
            ? current.data.traverse_state.limit_outer
            : start === "Right"
              ? current.data.traverse_state.limit_inner
              : current.data.traverse_state.custom_start_position;
      },
      () => requestTraverseSetStart(withMachine({ SetTraverseStart: start })),
    );
  };

  const setTraverseStartPosition = (position: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.start = "Custom";
        current.data.traverse_state.start_position = position;
        current.data.traverse_state.custom_start_position = position;
      },
      () =>
        requestTraverseSetStartPosition(
          withMachine({ SetTraverseStartPosition: position }),
        ),
    );
  };

  const setTraverseStepSize = (stepSize: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.step_size = stepSize;
      },
      () =>
        requestTraverseSetStepSize(
          withMachine({ SetTraverseStepSize: stepSize }),
        ),
    );
  };

  const setTraversePadding = (padding: number) => {
    if (!settingsEditPermitted) {
      return;
    }

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.padding = padding;
      },
      () =>
        requestTraverseSetPadding(withMachine({ SetTraversePadding: padding })),
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

    updateStateOptimistically(
      (current) => {
        current.data.traverse_state.laserpointer = enabled;
      },
      () =>
        requestEnableTraverseLaserpointer(
          withMachine({ EnableTraverseLaserpointer: enabled }),
        ),
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
    isLoading: stateOptimistic.isOptimistic,
    isDisabled: !stateOptimistic.isInitialized,
    motionStopped,
    settingsEditPermitted,
    manualTraversePermitted,
    setMode,
    setPullerTargetSpeed,
    setTakeupSpoolDiameter,
    setSourceSpoolDiameter,
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
    setTraverseStart,
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
