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
  operationModeSchema,
  OperationMode,
  spoolRegulationModeSchema,
  pullerRegulationSchema,
  PullerRegulation,
  SpoolAutomaticActionMode,
  spoolAutomaticActionModeSchema,
  gearRatioSchema,
  GearRatio,
  StepperMode,
  HeatingMode,
} from "../state/gluetexNamespace";
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
    optris1Voltage,
    optris2Voltage,
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
  const { request: requestOperationModeSet } = useMachineMutation(
    z.object({ SetOperationMode: operationModeSchema }),
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
  const { request: requestAddonMotor3SetForward } = useMachineMutation(
    z.object({ SetAddonMotor3Forward: z.boolean() }),
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
  const { request: requestAddonMotor4SetForward } = useMachineMutation(
    z.object({ SetAddonMotor4Forward: z.boolean() }),
  );
  const { request: requestAddonMotor4SetMasterRatio } = useMachineMutation(
    z.object({ SetAddonMotor4MasterRatio: z.number() }),
  );
  const { request: requestAddonMotor4SetSlaveRatio } = useMachineMutation(
    z.object({ SetAddonMotor4SlaveRatio: z.number() }),
  );

  // Addon Motor 5 mutations
  const { request: requestAddonMotor5SetEnabled } = useMachineMutation(
    z.object({ SetAddonMotor5Enabled: z.boolean() }),
  );
  const { request: requestAddonMotor5SetForward } = useMachineMutation(
    z.object({ SetAddonMotor5Forward: z.boolean() }),
  );
  const { request: requestAddonMotor5SetMasterRatio } = useMachineMutation(
    z.object({ SetAddonMotor5MasterRatio: z.number() }),
  );
  const { request: requestAddonMotor5SetSlaveRatio } = useMachineMutation(
    z.object({ SetAddonMotor5SlaveRatio: z.number() }),
  );
  const { request: requestAddonMotor5TensionSetEnabled } = useMachineMutation(
    z.object({ SetAddonMotor5TensionEnabled: z.boolean() }),
  );
  const { request: requestAddonMotor5TensionSetTargetAngle } =
    useMachineMutation(
      z.object({ SetAddonMotor5TensionTargetAngle: z.number() }),
    );
  const { request: requestAddonMotor5TensionSetSensitivity } =
    useMachineMutation(
      z.object({ SetAddonMotor5TensionSensitivity: z.number() }),
    );
  const { request: requestAddonMotor5TensionSetMinSpeedFactor } =
    useMachineMutation(
      z.object({ SetAddonMotor5TensionMinSpeedFactor: z.number() }),
    );
  const { request: requestAddonMotor5TensionSetMaxSpeedFactor } =
    useMachineMutation(
      z.object({ SetAddonMotor5TensionMaxSpeedFactor: z.number() }),
    );
  const { request: requestAddonMotor3SetKonturlaenge } = useMachineMutation(
    z.object({ SetAddonMotor3Konturlaenge: z.number() }),
  );
  const { request: requestAddonMotor3SetPause } = useMachineMutation(
    z.object({ SetAddonMotor3Pause: z.number() }),
  );
  const { request: requestHomeAddonMotor3 } = useMachineMutation(
    z.literal("HomeAddonMotor3"),
  );

  // Slave Puller mutations
  const { request: requestSlavePullerSetEnabled } = useMachineMutation(
    z.object({ SetSlavePullerEnabled: z.boolean() }),
  );
  const { request: requestSlavePullerSetForward } = useMachineMutation(
    z.object({ SetSlavePullerForward: z.boolean() }),
  );
  const { request: requestSlavePullerSetTargetAngle } = useMachineMutation(
    z.object({ SetSlavePullerTargetAngle: z.number() }),
  );
  const { request: requestSlavePullerSetSensitivity } = useMachineMutation(
    z.object({ SetSlavePullerSensitivity: z.number() }),
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

  // Winder Tension Arm Monitor mutations
  const { request: requestSetWinderTensionArmMonitorEnabled } =
    useMachineMutation(
      z.object({ SetWinderTensionArmMonitorEnabled: z.boolean() }),
    );
  const { request: requestSetWinderTensionArmMonitorMinAngle } =
    useMachineMutation(
      z.object({ SetWinderTensionArmMonitorMinAngle: z.number() }),
    );
  const { request: requestSetWinderTensionArmMonitorMaxAngle } =
    useMachineMutation(
      z.object({ SetWinderTensionArmMonitorMaxAngle: z.number() }),
    );

  // Addon Tension Arm Monitor mutations
  const { request: requestSetAddonTensionArmMonitorEnabled } =
    useMachineMutation(
      z.object({ SetAddonTensionArmMonitorEnabled: z.boolean() }),
    );
  const { request: requestSetAddonTensionArmMonitorMinAngle } =
    useMachineMutation(
      z.object({ SetAddonTensionArmMonitorMinAngle: z.number() }),
    );
  const { request: requestSetAddonTensionArmMonitorMaxAngle } =
    useMachineMutation(
      z.object({ SetAddonTensionArmMonitorMaxAngle: z.number() }),
    );

  // Slave Tension Arm Monitor mutations
  const { request: requestSetSlaveTensionArmMonitorEnabled } =
    useMachineMutation(
      z.object({ SetSlaveTensionArmMonitorEnabled: z.boolean() }),
    );
  const { request: requestSetSlaveTensionArmMonitorMinAngle } =
    useMachineMutation(
      z.object({ SetSlaveTensionArmMonitorMinAngle: z.number() }),
    );
  const { request: requestSetSlaveTensionArmMonitorMaxAngle } =
    useMachineMutation(
      z.object({ SetSlaveTensionArmMonitorMaxAngle: z.number() }),
    );

  // Voltage Monitor mutations
  const { request: requestSetOptris1MonitorEnabled } = useMachineMutation(
    z.object({ SetOptris1MonitorEnabled: z.boolean() }),
  );
  const { request: requestSetOptris1MonitorMinVoltage } = useMachineMutation(
    z.object({ SetOptris1MonitorMinVoltage: z.number() }),
  );
  const { request: requestSetOptris1MonitorMaxVoltage } = useMachineMutation(
    z.object({ SetOptris1MonitorMaxVoltage: z.number() }),
  );
  const { request: requestSetOptris1MonitorDelay } = useMachineMutation(
    z.object({ SetOptris1MonitorDelay: z.number() }),
  );
  const { request: requestSetOptris2MonitorEnabled } = useMachineMutation(
    z.object({ SetOptris2MonitorEnabled: z.boolean() }),
  );
  const { request: requestSetOptris2MonitorMinVoltage } = useMachineMutation(
    z.object({ SetOptris2MonitorMinVoltage: z.number() }),
  );
  const { request: requestSetOptris2MonitorMaxVoltage } = useMachineMutation(
    z.object({ SetOptris2MonitorMaxVoltage: z.number() }),
  );
  const { request: requestSetOptris2MonitorDelay } = useMachineMutation(
    z.object({ SetOptris2MonitorDelay: z.number() }),
  );

  // Sleep Timer mutations
  const { request: requestSetSleepTimerEnabled } = useMachineMutation(
    z.object({ SetSleepTimerEnabled: z.boolean() }),
  );
  const { request: requestSetSleepTimerTimeout } = useMachineMutation(
    z.object({ SetSleepTimerTimeout: z.number() }),
  );
  const { request: requestResetSleepTimer } = useMachineMutation(
    z.literal("ResetSleepTimer"),
  );

  // Order Information mutations
  const { request: requestSetOrderNumber } = useMachineMutation(
    z.object({ SetOrderNumber: z.number() }),
  );
  const { request: requestSetSerialNumber } = useMachineMutation(
    z.object({ SetSerialNumber: z.number() }),
  );
  const { request: requestSetProductDescription } = useMachineMutation(
    z.object({ SetProductDescription: z.string() }),
  );

  // Valve Control mutations
  const { request: requestSetValveEnabled } = useMachineMutation(
    z.object({ SetValveEnabled: z.boolean() }),
  );
  const { request: requestSetValveManualOverride } = useMachineMutation(
    z.object({ SetValveManualOverride: z.boolean().nullable() }),
  );
  const { request: requestSetValveOnDistanceMm } = useMachineMutation(
    z.object({ SetValveOnDistanceMm: z.number() }),
  );
  const { request: requestSetValveOffDistanceMm } = useMachineMutation(
    z.object({ SetValveOffDistanceMm: z.number() }),
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

  const { request: requestStartHeatingAutoTune } = useMachineMutation(
    z.object({
      StartHeatingAutoTune: z.tuple([
        z.enum(["Zone1", "Zone2", "Zone3", "Zone4", "Zone5", "Zone6"]),
        z.number(),
      ]),
    }),
  );

  const { request: requestStopHeatingAutoTune } = useMachineMutation(
    z.object({
      StopHeatingAutoTune: z.enum([
        "Zone1",
        "Zone2",
        "Zone3",
        "Zone4",
        "Zone5",
        "Zone6",
      ]),
    }),
  );

  const { request: requestSetHeatingEnabled } = useMachineMutation(
    z.object({ SetHeatingEnabled: z.boolean() }),
  );

  const { request: requestSetHeatingTargetTemperature } = useMachineMutation(
    z.object({
      SetHeatingTargetTemperature: z.tuple([
        z.enum(["Zone1", "Zone2", "Zone3", "Zone4", "Zone5", "Zone6"]),
        z.number(),
      ]),
    }),
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

  const setOperationMode = (mode: OperationMode) => {
    updateStateOptimistically(
      (current) => {
        current.data.mode_state.operation_mode = mode;
      },
      () =>
        requestOperationModeSet({
          machine_identification_unique: machineIdentification,
          data: { SetOperationMode: mode },
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

  const setOptris1Min = (min: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.optris1.min_voltage = min;
    });
  };

  const setOptris1Max = (max: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.optris1.max_voltage = max;
    });
  };

  const setOptris2Min = (min: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.optris2.min_voltage = min;
    });
  };

  const setOptris2Max = (max: number) => {
    updateStateLocally((current) => {
      current.data.quality_control_state.optris2.max_voltage = max;
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

  const setStepper5Mode = (mode: StepperMode) => {
    const enabled = mode === "Run";
    updateStateOptimistically(
      (current) => {
        current.data.stepper_state.stepper5_mode = mode;
      },
      () => {
        requestAddonMotor5SetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5Enabled: enabled },
        });
      },
    );
  };

  const setStepper5Master = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper5_master = value;
      },
      () =>
        requestAddonMotor5SetMasterRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5MasterRatio: value },
        }),
    );
  };

  const setStepper5Slave = (value: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.motor_ratios_state.stepper5_slave = value;
      },
      () =>
        requestAddonMotor5SetSlaveRatio({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5SlaveRatio: value },
        }),
    );
  };

  const setStepper3Konturlaenge = (value: number) => {
    // Update backend only, no optimistic update needed for now
    requestAddonMotor3SetKonturlaenge({
      machine_identification_unique: machineIdentification,
      data: { SetAddonMotor3Konturlaenge: value },
    });
  };

  const setStepper3Pause = (value: number) => {
    // Update backend only, no optimistic update needed for now
    requestAddonMotor3SetPause({
      machine_identification_unique: machineIdentification,
      data: { SetAddonMotor3Pause: value },
    });
  };

  const homeAddonMotor3 = () => {
    // Trigger manual homing for addon motor 3
    requestHomeAddonMotor3({
      machine_identification_unique: machineIdentification,
      data: "HomeAddonMotor3",
    });
  };

  // Addon Motor Forward direction setters
  const setStepper3Forward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_3_state.forward = forward;
      },
      () =>
        requestAddonMotor3SetForward({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor3Forward: forward },
        }),
    );
  };

  const setStepper4Forward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_4_state.forward = forward;
      },
      () =>
        requestAddonMotor4SetForward({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor4Forward: forward },
        }),
    );
  };

  const setStepper5Forward = (forward: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_state.forward = forward;
      },
      () =>
        requestAddonMotor5SetForward({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5Forward: forward },
        }),
    );
  };

  const setAddonMotor5TensionEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_tension_control_state.enabled = enabled;
      },
      () =>
        requestAddonMotor5TensionSetEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5TensionEnabled: enabled },
        }),
    );
  };

  const setAddonMotor5TensionTargetAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_tension_control_state.target_angle = angle;
      },
      () =>
        requestAddonMotor5TensionSetTargetAngle({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5TensionTargetAngle: angle },
        }),
    );
  };

  const setAddonMotor5TensionSensitivity = (sensitivity: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_tension_control_state.sensitivity =
          sensitivity;
      },
      () =>
        requestAddonMotor5TensionSetSensitivity({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5TensionSensitivity: sensitivity },
        }),
    );
  };

  const setAddonMotor5TensionMinSpeedFactor = (factor: number | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_tension_control_state.min_speed_factor =
          factor;
      },
      () =>
        requestAddonMotor5TensionSetMinSpeedFactor({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5TensionMinSpeedFactor: factor ?? 0 },
        }),
    );
  };

  const setAddonMotor5TensionMaxSpeedFactor = (factor: number | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_motor_5_tension_control_state.max_speed_factor =
          factor;
      },
      () =>
        requestAddonMotor5TensionSetMaxSpeedFactor({
          machine_identification_unique: machineIdentification,
          data: { SetAddonMotor5TensionMaxSpeedFactor: factor ?? 0 },
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

  const setSlavePullerTargetAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.target_angle = angle;
      },
      () =>
        requestSlavePullerSetTargetAngle({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerTargetAngle: angle },
        }),
    );
  };

  const setSlavePullerSensitivity = (sensitivity: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_puller_state.sensitivity = sensitivity;
      },
      () =>
        requestSlavePullerSetSensitivity({
          machine_identification_unique: machineIdentification,
          data: { SetSlavePullerSensitivity: sensitivity },
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

  // Winder Tension Arm Monitor action functions
  const setWinderTensionArmMonitorEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.winder_tension_arm_monitor_state.enabled = enabled;
        // Clear triggered flag when disabling
        if (!enabled) {
          current.data.winder_tension_arm_monitor_state.triggered = false;
        }
      },
      () =>
        requestSetWinderTensionArmMonitorEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetWinderTensionArmMonitorEnabled: enabled },
        }),
    );
  };

  const setWinderTensionArmMonitorMinAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.winder_tension_arm_monitor_state.min_angle = angle;
      },
      () =>
        requestSetWinderTensionArmMonitorMinAngle({
          machine_identification_unique: machineIdentification,
          data: { SetWinderTensionArmMonitorMinAngle: angle },
        }),
    );
  };

  const setWinderTensionArmMonitorMaxAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.winder_tension_arm_monitor_state.max_angle = angle;
      },
      () =>
        requestSetWinderTensionArmMonitorMaxAngle({
          machine_identification_unique: machineIdentification,
          data: { SetWinderTensionArmMonitorMaxAngle: angle },
        }),
    );
  };

  // Addon Tension Arm Monitor action functions
  const setAddonTensionArmMonitorEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_tension_arm_monitor_state.enabled = enabled;
        // Clear triggered flag when disabling
        if (!enabled) {
          current.data.addon_tension_arm_monitor_state.triggered = false;
        }
      },
      () =>
        requestSetAddonTensionArmMonitorEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetAddonTensionArmMonitorEnabled: enabled },
        }),
    );
  };

  const setAddonTensionArmMonitorMinAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_tension_arm_monitor_state.min_angle = angle;
      },
      () =>
        requestSetAddonTensionArmMonitorMinAngle({
          machine_identification_unique: machineIdentification,
          data: { SetAddonTensionArmMonitorMinAngle: angle },
        }),
    );
  };

  const setAddonTensionArmMonitorMaxAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.addon_tension_arm_monitor_state.max_angle = angle;
      },
      () =>
        requestSetAddonTensionArmMonitorMaxAngle({
          machine_identification_unique: machineIdentification,
          data: { SetAddonTensionArmMonitorMaxAngle: angle },
        }),
    );
  };

  // Slave Tension Arm Monitor action functions
  const setSlaveTensionArmMonitorEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_tension_arm_monitor_state.enabled = enabled;
        // Clear triggered flag when disabling
        if (!enabled) {
          current.data.slave_tension_arm_monitor_state.triggered = false;
        }
      },
      () =>
        requestSetSlaveTensionArmMonitorEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetSlaveTensionArmMonitorEnabled: enabled },
        }),
    );
  };

  const setSlaveTensionArmMonitorMinAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_tension_arm_monitor_state.min_angle = angle;
      },
      () =>
        requestSetSlaveTensionArmMonitorMinAngle({
          machine_identification_unique: machineIdentification,
          data: { SetSlaveTensionArmMonitorMinAngle: angle },
        }),
    );
  };

  const setSlaveTensionArmMonitorMaxAngle = (angle: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.slave_tension_arm_monitor_state.max_angle = angle;
      },
      () =>
        requestSetSlaveTensionArmMonitorMaxAngle({
          machine_identification_unique: machineIdentification,
          data: { SetSlaveTensionArmMonitorMaxAngle: angle },
        }),
    );
  };

  // Voltage Monitor action functions
  const setOptris1MonitorEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_1_monitor_state.enabled = enabled;
        // Clear triggered flag when disabling
        if (!enabled) {
          current.data.optris_1_monitor_state.triggered = false;
        }
      },
      () =>
        requestSetOptris1MonitorEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetOptris1MonitorEnabled: enabled },
        }),
    );
  };

  const setOptris1MonitorMinVoltage = (voltage: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_1_monitor_state.min_voltage = voltage;
      },
      () =>
        requestSetOptris1MonitorMinVoltage({
          machine_identification_unique: machineIdentification,
          data: { SetOptris1MonitorMinVoltage: voltage },
        }),
    );
  };

  const setOptris1MonitorMaxVoltage = (voltage: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_1_monitor_state.max_voltage = voltage;
      },
      () =>
        requestSetOptris1MonitorMaxVoltage({
          machine_identification_unique: machineIdentification,
          data: { SetOptris1MonitorMaxVoltage: voltage },
        }),
    );
  };

  const setOptris2MonitorEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_2_monitor_state.enabled = enabled;
        // Clear triggered flag when disabling
        if (!enabled) {
          current.data.optris_2_monitor_state.triggered = false;
        }
      },
      () =>
        requestSetOptris2MonitorEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetOptris2MonitorEnabled: enabled },
        }),
    );
  };

  const setOptris2MonitorMinVoltage = (voltage: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_2_monitor_state.min_voltage = voltage;
      },
      () =>
        requestSetOptris2MonitorMinVoltage({
          machine_identification_unique: machineIdentification,
          data: { SetOptris2MonitorMinVoltage: voltage },
        }),
    );
  };

  const setOptris2MonitorMaxVoltage = (voltage: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_2_monitor_state.max_voltage = voltage;
      },
      () =>
        requestSetOptris2MonitorMaxVoltage({
          machine_identification_unique: machineIdentification,
          data: { SetOptris2MonitorMaxVoltage: voltage },
        }),
    );
  };

  const setOptris1MonitorDelay = (delay_mm: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_1_monitor_state.delay_mm = delay_mm;
      },
      () =>
        requestSetOptris1MonitorDelay({
          machine_identification_unique: machineIdentification,
          data: { SetOptris1MonitorDelay: delay_mm },
        }),
    );
  };

  const setOptris2MonitorDelay = (delay_mm: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.optris_2_monitor_state.delay_mm = delay_mm;
      },
      () =>
        requestSetOptris2MonitorDelay({
          machine_identification_unique: machineIdentification,
          data: { SetOptris2MonitorDelay: delay_mm },
        }),
    );
  };

  // Sleep Timer action functions
  const setSleepTimerEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.sleep_timer_state.enabled = enabled;
        // Reset remaining time when enabling
        if (enabled) {
          current.data.sleep_timer_state.remaining_seconds =
            current.data.sleep_timer_state.timeout_seconds;
        }
      },
      () =>
        requestSetSleepTimerEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetSleepTimerEnabled: enabled },
        }),
    );
  };

  const setSleepTimerTimeout = (timeoutSeconds: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.sleep_timer_state.timeout_seconds = timeoutSeconds;
      },
      () =>
        requestSetSleepTimerTimeout({
          machine_identification_unique: machineIdentification,
          data: { SetSleepTimerTimeout: timeoutSeconds },
        }),
    );
  };

  const resetSleepTimer = () => {
    updateStateOptimistically(
      (current) => {
        current.data.sleep_timer_state.remaining_seconds =
          current.data.sleep_timer_state.timeout_seconds;
      },
      () =>
        requestResetSleepTimer({
          machine_identification_unique: machineIdentification,
          data: "ResetSleepTimer",
        }),
    );
  };

  // Order Information action functions
  const setOrderNumber = (orderNumber: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.order_info_state.order_number = orderNumber;
      },
      () =>
        requestSetOrderNumber({
          machine_identification_unique: machineIdentification,
          data: { SetOrderNumber: orderNumber },
        }),
    );
  };

  const setSerialNumber = (serialNumber: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.order_info_state.serial_number = serialNumber;
      },
      () =>
        requestSetSerialNumber({
          machine_identification_unique: machineIdentification,
          data: { SetSerialNumber: serialNumber },
        }),
    );
  };

  const setProductDescription = (description: string) => {
    updateStateOptimistically(
      (current) => {
        current.data.order_info_state.product_description = description;
      },
      () =>
        requestSetProductDescription({
          machine_identification_unique: machineIdentification,
          data: { SetProductDescription: description },
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

  const startHeatingAutoTune = (
    zoneNumber: number,
    targetTemperature: number,
  ) => {
    const zoneMap = {
      1: "Zone1",
      2: "Zone2",
      3: "Zone3",
      4: "Zone4",
      5: "Zone5",
      6: "Zone6",
    } as const;
    const zone = zoneMap[zoneNumber as keyof typeof zoneMap];

    if (!zone) {
      toastError("Invalid Zone", `Zone number ${zoneNumber} is invalid`);
      return;
    }

    // Automatically switch to Heating mode for auto-tuning
    setHeatingMode("Heating");

    requestStartHeatingAutoTune({
      machine_identification_unique: machineIdentification,
      data: { StartHeatingAutoTune: [zone, targetTemperature] },
    });
  };

  const stopHeatingAutoTune = (zoneNumber: number) => {
    const zoneMap = {
      1: "Zone1",
      2: "Zone2",
      3: "Zone3",
      4: "Zone4",
      5: "Zone5",
      6: "Zone6",
    } as const;
    const zone = zoneMap[zoneNumber as keyof typeof zoneMap];

    if (!zone) {
      toastError("Invalid Zone", `Zone number ${zoneNumber} is invalid`);
      return;
    }

    requestStopHeatingAutoTune({
      machine_identification_unique: machineIdentification,
      data: { StopHeatingAutoTune: zone },
    });
  };

  const setHeatingZone1Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_1.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone1", temperature] },
        }),
    );
  };

  const setHeatingZone2Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_2.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone2", temperature] },
        }),
    );
  };

  const setHeatingZone3Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_3.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone3", temperature] },
        }),
    );
  };

  const setHeatingZone4Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_4.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone4", temperature] },
        }),
    );
  };

  const setHeatingZone5Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_5.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone5", temperature] },
        }),
    );
  };

  const setHeatingZone6Temperature = (temperature: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.heating_states.zone_6.target_temperature = temperature;
      },
      () =>
        requestSetHeatingTargetTemperature({
          machine_identification_unique: machineIdentification,
          data: { SetHeatingTargetTemperature: ["Zone6", temperature] },
        }),
    );
  };

  // ========== Valve Control Functions ==========

  const setValveEnabled = (enabled: boolean) => {
    updateStateOptimistically(
      (current) => {
        current.data.valve_state.enabled = enabled;
      },
      () =>
        requestSetValveEnabled({
          machine_identification_unique: machineIdentification,
          data: { SetValveEnabled: enabled },
        }),
    );
  };

  const setValveManualOverride = (manual: boolean | null) => {
    updateStateOptimistically(
      (current) => {
        current.data.valve_state.manual_override = manual;
      },
      () =>
        requestSetValveManualOverride({
          machine_identification_unique: machineIdentification,
          data: { SetValveManualOverride: manual },
        }),
    );
  };

  const setValveOnDistanceMm = (distance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.valve_state.on_distance_mm = distance;
      },
      () =>
        requestSetValveOnDistanceMm({
          machine_identification_unique: machineIdentification,
          data: { SetValveOnDistanceMm: distance },
        }),
    );
  };

  const setValveOffDistanceMm = (distance: number) => {
    updateStateOptimistically(
      (current) => {
        current.data.valve_state.off_distance_mm = distance;
      },
      () =>
        requestSetValveOffDistanceMm({
          machine_identification_unique: machineIdentification,
          data: { SetValveOffDistanceMm: distance },
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
            0x000b,
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
  }, [filteredMachines, state?.data.connected_machine_state]);

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
    optris1Voltage,
    optris2Voltage,

    // Loading states
    isLoading,
    isDisabled,

    // Standard winder action functions (backend connected)
    enableTraverseLaserpointer,
    setMode,
    setOperationMode,
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
    setSlavePullerTargetAngle,
    setSlavePullerSensitivity,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,
    zeroSlaveTensionArm,
    zeroAddonTensionArm,

    // Winder Tension Arm Monitor action functions
    setWinderTensionArmMonitorEnabled,
    setWinderTensionArmMonitorMinAngle,
    setWinderTensionArmMonitorMaxAngle,

    // Addon Tension Arm Monitor action functions
    setAddonTensionArmMonitorEnabled,
    setAddonTensionArmMonitorMinAngle,
    setAddonTensionArmMonitorMaxAngle,

    // Slave Tension Arm Monitor action functions
    setSlaveTensionArmMonitorEnabled,
    setSlaveTensionArmMonitorMinAngle,
    setSlaveTensionArmMonitorMaxAngle,

    // Voltage Monitor action functions
    setOptris1MonitorEnabled,
    setOptris1MonitorMinVoltage,
    setOptris1MonitorMaxVoltage,
    setOptris1MonitorDelay,
    setOptris2MonitorEnabled,
    setOptris2MonitorMinVoltage,
    setOptris2MonitorMaxVoltage,
    setOptris2MonitorDelay,

    // Sleep Timer action functions
    setSleepTimerEnabled,
    setSleepTimerTimeout,
    resetSleepTimer,

    // Order Information action functions
    setOrderNumber,
    setSerialNumber,
    setProductDescription,

    // Heating action functions
    setHeatingPid,
    startHeatingAutoTune,
    stopHeatingAutoTune,
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,

    // Valve control action functions
    setValveEnabled,
    setValveManualOverride,
    setValveOnDistanceMm,
    setValveOffDistanceMm,

    // Addon action functions (local only)
    setStepper3Mode,
    setStepper4Mode,
    setStepper5Mode,
    setHeatingMode,
    setOptris1Min,
    setOptris1Max,
    setOptris2Min,
    setOptris2Max,
    setStepper3Master,
    setStepper3Slave,
    setStepper3Forward,
    setStepper4Master,
    setStepper4Slave,
    setStepper4Forward,
    setStepper5Master,
    setStepper5Slave,
    setStepper5Forward,
    setStepper3Konturlaenge,
    setStepper3Pause,
    homeAddonMotor3,
    setAddonMotor5TensionEnabled,
    setAddonMotor5TensionTargetAngle,
    setAddonMotor5TensionSensitivity,
    setAddonMotor5TensionMinSpeedFactor,
    setAddonMotor5TensionMaxSpeedFactor,
  };
}
