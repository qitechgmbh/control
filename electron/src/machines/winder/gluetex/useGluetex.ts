/**
 * @file useGluetex.ts
 * @description Hook for Gluetex machine with real backend connection.
 * Standard winder features connect to backend, addon features remain local.
 */

import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { useStateOptimistic } from "@/lib/useStateOptimistic";
import { MachineIdentificationUnique } from "@/machines/types";
import { VENDOR_QITECH, gluetex } from "@/machines/properties";
import { gluetexRoute } from "@/routes/routes";
import { useEffect, useMemo } from "react";
import { produce } from "immer";
import { useMachines } from "@/client/useMachines";
import { z } from "zod";
import {
  SpoolRegulationMode,
  ExtendedStateEvent,
  useGluetexNamespace,
  modeSchema,
  Mode,
  spoolRegulationModeSchema,
  pullerRegulationSchema,
  PullerRegulation,
  SpoolAutomaticActionMode,
  spoolAutomaticActionModeSchema,
  gearRatioSchema,
  GearRatio,
  StepperMode,
  HeatingMode,
} from "./gluetexNamespace";
import { machineIdentificationUnique } from "@/machines/types";

export function useGluetex() {
  const { serial: serialString } = gluetexRoute.useParams();

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
      machine_identification: gluetex.machine_identification,
      serial,
    };
  }, [serialString]);

  // Get consolidated state and live values from namespace
  const {
    state,
    defaultState,
    traversePosition,
    pullerSpeed,
    slavePullerSpeed,
    slaveTensionArmAngle,
    addonTensionArmAngle,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
    heater1Power,
    heater2Power,
    heater3Power,
    heater4Power,
    heater5Power,
    heater6Power,
  } = useGluetexNamespace(machineIdentification);

  // Single optimistic state for all state management
  const stateOptimistic = useStateOptimistic<ExtendedStateEvent>();

  // Update optimistic state when real state changes
  useEffect(() => {
    if (state) {
      stateOptimistic.setReal(state);
    }
  }, [state]);

  // ========== Backend Mutation Requests (Standard Winder Features) ==========

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

  // Addon Motor 3 mutations
  const { request: requestAddonMotor3SetEnabled } = useMachineMutation(
    z.object({ SetAddonMotor3Enabled: z.boolean() }),
  );
  const { request: requestAddonMotor3SetMasterRatio } = useMachineMutation(
    z.object({ SetAddonMotor3MasterRatio: z.number() }),
  );
  const { request: requestAddonMotor3SetSlaveRatio } = useMachineMutation(
    z.object({ SetAddonMotor3SlaveRatio: z.number() }),
  );

  // Addon Motor 4 mutations
  const { request: requestAddonMotor4SetEnabled } = useMachineMutation(
    z.object({ SetAddonMotor4Enabled: z.boolean() }),
  );
  const { request: requestAddonMotor4SetMasterRatio } = useMachineMutation(
    z.object({ SetAddonMotor4MasterRatio: z.number() }),
  );
  const { request: requestAddonMotor4SetSlaveRatio } = useMachineMutation(
    z.object({ SetAddonMotor4SlaveRatio: z.number() }),
  );

  // Slave Puller mutations
  const { request: requestSlavePullerSetEnabled } = useMachineMutation(
    z.object({ SetSlavePullerEnabled: z.boolean() }),
  );
  const { request: requestSlavePullerSetForward } = useMachineMutation(
    z.object({ SetSlavePullerForward: z.boolean() }),
  );
  const { request: requestSlavePullerSetMinAngle } = useMachineMutation(
    z.object({ SetSlavePullerMinAngle: z.number() }),
  );
  const { request: requestSlavePullerSetMaxAngle } = useMachineMutation(
    z.object({ SetSlavePullerMaxAngle: z.number() }),
  );
  const { request: requestSlavePullerSetMinSpeedFactor } = useMachineMutation(
    z.object({ SetSlavePullerMinSpeedFactor: z.number() }),
  );
  const { request: requestSlavePullerSetMaxSpeedFactor } = useMachineMutation(
    z.object({ SetSlavePullerMaxSpeedFactor: z.number() }),
  );
  const { request: requestZeroSlaveTensionArm } = useMachineMutation(
    z.literal("ZeroSlaveTensionArm"),
  );
  const { request: requestZeroAddonTensionArm } = useMachineMutation(
    z.literal("ZeroAddonTensionArm"),
  );

  const { request: requestConfigureHeatingPid } = useMachineMutation(
    z.object({
      ConfigureHeatingPid: z.object({
        zone: z.string(),
        kp: z.number(),
        ki: z.number(),
        kd: z.number(),
      }),
    }),
  );

  const { request: requestSetHeatingEnabled } = useMachineMutation(
    z.object({ SetHeatingEnabled: z.boolean() }),
  );

  // ========== Helper Functions ==========

  // Helper function for optimistic updates using produce
  const updateStateOptimistically = (
    producer: (current: ExtendedStateEvent) => void,
    serverRequest: () => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState && !stateOptimistic.isOptimistic) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
    serverRequest();
  };

  // Helper for local-only updates (addon features)
  const updateStateLocally = (
    producer: (current: ExtendedStateEvent) => void,
  ) => {
    const currentState = stateOptimistic.value;
    if (currentState) {
      stateOptimistic.setOptimistic(produce(currentState, producer));
    }
  };

  // ========== Action Functions (Standard Winder - Backend Connected) ==========

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

  const setConnectedMachine = (
    machineIdentificationUnique: MachineIdentificationUnique,
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          machineIdentificationUnique;
      },
      () =>
        requestConnectedMachine({
          machine_identification_unique: machineIdentification,
          data: { SetConnectedMachine: machineIdentificationUnique },
        }),
    );
  };

  const disconnectMachine = (
    machineIdentificationUnique: MachineIdentificationUnique,
  ) => {
    updateStateOptimistically(
      (current) => {
        current.data.connected_machine_state.machine_identification_unique =
          null;
      },
      () =>
        requestDisconnectedMachine({
          machine_identification_unique: machineIdentification,
          data: { DisconnectMachine: machineIdentificationUnique },
        }),
    );
  };

  // ========== Action Functions (Addon Features - Local Only) ==========

  const setStepper3Mode = (mode: StepperMode) => {
    const enabled = mode === "Run";
    updateStateOptimistically(
      (current) => {
        current.data.stepper_state.stepper3_mode = mode;
      },
      () => {
        requestAddonMotor3SetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor3Enabled: enabled },
        });
      },
    );
  };

  const setStepper4Mode = (mode: StepperMode) => {
    const enabled = mode === "Run";
    updateStateOptimistically(
      (current) => {
        current.data.stepper_state.stepper4_mode = mode;
      },
      () => {
        requestAddonMotor4SetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor4Enabled: enabled },
        });
      },
    );
  };

  const setHeatingMode = (mode: HeatingMode) => {
    const enabled = mode === "Heating";
    updateStateOptimistically(
      (current) => {
        current.data.heating_state.heating_mode = mode;
      },
      () => {
        requestSetHeatingEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingEnabled: enabled },
        });
      },
    );
  };

  const setTemperature1Min = (min: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.temperature1.min_temperature = min;
    });
  };

  const setTemperature1Max = (max: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.temperature1.max_temperature = max;
    });
  };

  const setTemperature2Min = (min: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.temperature2.min_temperature = min;
    });
  };

  const setTemperature2Max = (max: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.temperature2.max_temperature = max;
    });
  };

  const setStepper3Master = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper3_master = value;
      },
      () =>
        requestAddonMotor3SetMasterRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor3MasterRatio: value },
        }),
    );
  };

  const setStepper3Slave = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper3_slave = value;
      },
      () =>
        requestAddonMotor3SetSlaveRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor3SlaveRatio: value },
        }),
    );
  };

  const setStepper4Master = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper4_master = value;
      },
      () =>
        requestAddonMotor4SetMasterRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor4MasterRatio: value },
        }),
    );
  };

  const setStepper4Slave = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper4_slave = value;
      },
      () =>
        requestAddonMotor4SetSlaveRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor4SlaveRatio: value },
        }),
    );
  };

  // ========== Slave Puller Functions ==========

  const setSlavePullerEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.enabled = enabled;
      },
      () =>
        requestSlavePullerSetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerEnabled: enabled },
        }),
    );
  };

  const setSlavePullerForward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.forward = forward;
      },
      () =>
        requestSlavePullerSetForward({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerForward: forward },
        }),
    );
  };

  const setSlavePullerMinAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.min_angle = angle;
      },
      () =>
        requestSlavePullerSetMinAngle({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerMinAngle: angle },
        }),
    );
  };

  const setSlavePullerMaxAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.max_angle = angle;
      },
      () =>
        requestSlavePullerSetMaxAngle({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerMaxAngle: angle },
        }),
    );
  };

  const setSlavePullerMinSpeedFactor = (factor: number | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.min_speed_factor = factor;
      },
      () =>
        requestSlavePullerSetMinSpeedFactor({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerMinSpeedFactor: factor ?? 0 },
        }),
    );
  };

  const setSlavePullerMaxSpeedFactor = (factor: number | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.max_speed_factor = factor;
      },
      () =>
        requestSlavePullerSetMaxSpeedFactor({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerMaxSpeedFactor: factor ?? 0 },
        }),
    );
  };

  const zeroSlaveTensionArm = () => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.tension_arm.zeroed = true;
      },
      () =>
        requestZeroSlaveTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroSlaveTensionArm",
        }),
    );
  };

  const zeroAddonTensionArm = () => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_tension_arm_state.zeroed = true;
      },
      () =>
        requestZeroAddonTensionArm({
          machine_identification_unique: machineIdentification,
          data: "ZeroAddonTensionArm",
        }),
    );
  };

  const setHeatingPid = (zone: string, kp: number, ki: number, kd: number) => {
    updateStateOptimistically(
      (current) => {
        const heatingPidSettings =
          current.data.heating_pid_settings[
            zone as keyof typeof current.data.heating_pid_settings
          ];
        if (heatingPidSettings) {
          heatingPidSettings.kp = kp;
          heatingPidSettings.ki = ki;
          heatingPidSettings.kd = kd;
        }
      },
      () =>
        requestConfigureHeatingPid({
          machine_identification_unique: machineIdentification,
          data: { ConfigureHeatingPid: { zone, kp, ki, kd } },
        }),
    );
  };

  // ========== Machine Filtering ==========

  const machines = useMachines();
  const filteredMachines = useMemo(
    () =>
      machines.filter(
        (m) =>
          m.machine_identification_unique.machine_identification.vendor ===
            VENDOR_QITECH &&
          m.machine_identification_unique.machine_identification.machine ===
            0x0008,
      ),
    [machines],
  );

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

  // ========== Loading States ==========

  const isLoading = stateOptimistic.isOptimistic;
  const isDisabled = !stateOptimistic.isInitialized;

  // ========== Return Hook Result ==========

  return {
    // State
    state: stateOptimistic.value?.data,
    defaultState: defaultState?.data,

    // Machine filtering
    filteredMachines,
    selectedMachine,

    // Live values (TimeSeries)
    traversePosition,
    pullerSpeed,
    slavePullerSpeed,
    slaveTensionArmAngle,
    addonTensionArmAngle,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
    heater1Power,
    heater2Power,
    heater3Power,
    heater4Power,
    heater5Power,
    heater6Power,

    // Loading states
    isLoading,
    isDisabled,

    // Standard winder action functions (backend connected)
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

    // Slave Puller action functions
    setSlavePullerEnabled,
    setSlavePullerForward,
    setSlavePullerMinAngle,
    setSlavePullerMaxAngle,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,
    zeroSlaveTensionArm,
    zeroAddonTensionArm,

    // Heating action functions
    setHeatingPid,

    // Addon action functions (local only)
    setStepper3Mode,
    setStepper4Mode,
    setHeatingMode,
    setTemperature1Min,
    setTemperature1Max,
    setTemperature2Min,
    setTemperature2Max,
    setStepper3Master,
    setStepper3Slave,
    setStepper4Master,
    setStepper4Slave,
  };
}
