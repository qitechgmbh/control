/**
 * @file useGluetex.ts
 * @description Hook for Gluetex machine with hardcoded test data (no backend integration).
 */

import { useMemo, useState } from "react";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  SpoolRegulationMode,
  StateEvent,
  useGluetexNamespace,
  Mode,
  PullerRegulation,
  SpoolAutomaticActionMode,
  GearRatio,
  StepperMode,
  HeatingMode,
} from "./gluetexNamespace";

export function useGluetex() {
  // For testing, we use a dummy machine identification
  const machineIdentification: MachineIdentificationUnique = useMemo(
    () => ({
      machine_identification: {
        vendor: 1,
        machine: 3,
      },
      serial: 999,
    }),
    [],
  );

  // Get state and live values from namespace (hardcoded)
  const {
    state: namespaceState,
    defaultState: namespaceDefaultState,
    traversePosition,
    pullerSpeed,
    slavePullerSpeed,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
    temperature1,
    temperature2,
  } = useGluetexNamespace(machineIdentification);

  // Use local state for testing changes
  const [localState, setLocalState] = useState<StateEvent | null>(
    namespaceState,
  );

  // Mock action functions - these update local state instead of calling backend
  const zeroTensionArmAngle = () => {
    console.log("Mock: Zero tension arm angle");
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        tension_arm_state: { ...prev.tension_arm_state, zeroed: true },
      };
    });
  };

  const setTraverseLimitInner = (limitInner: number) => {
    console.log("Mock: Set traverse limit inner:", limitInner);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, limit_inner: limitInner },
      };
    });
  };

  const setTraverseLimitOuter = (limitOuter: number) => {
    console.log("Mock: Set traverse limit outer:", limitOuter);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, limit_outer: limitOuter },
      };
    });
  };

  const gotoTraverseLimitInner = () => {
    console.log("Mock: Go to traverse limit inner");
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, is_going_in: true },
      };
    });
    setTimeout(() => {
      setLocalState((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          traverse_state: { ...prev.traverse_state, is_going_in: false },
        };
      });
    }, 2000);
  };

  const gotoTraverseLimitOuter = () => {
    console.log("Mock: Go to traverse limit outer");
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, is_going_out: true },
      };
    });
    setTimeout(() => {
      setLocalState((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          traverse_state: { ...prev.traverse_state, is_going_out: false },
        };
      });
    }, 2000);
  };

  const gotoTraverseHome = () => {
    console.log("Mock: Go to traverse home");
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, is_going_home: true },
      };
    });
    setTimeout(() => {
      setLocalState((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          traverse_state: {
            ...prev.traverse_state,
            is_going_home: false,
            is_homed: true,
          },
        };
      });
    }, 2000);
  };

  const enableTraverseLaserpointer = (enabled: boolean) => {
    console.log("Mock: Enable traverse laserpointer:", enabled);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, laserpointer: enabled },
      };
    });
  };

  const setMode = (mode: Mode) => {
    console.log("Mock: Set mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        mode_state: { ...prev.mode_state, mode },
      };
    });
  };

  const setTraverseStepSize = (stepSize: number) => {
    console.log("Mock: Set traverse step size:", stepSize);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, step_size: stepSize },
      };
    });
  };

  const setTraversePadding = (padding: number) => {
    console.log("Mock: Set traverse padding:", padding);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        traverse_state: { ...prev.traverse_state, padding },
      };
    });
  };

  const setPullerTargetSpeed = (targetSpeed: number) => {
    console.log("Mock: Set puller target speed:", targetSpeed);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        puller_state: { ...prev.puller_state, target_speed: targetSpeed },
      };
    });
  };

  const setPullerTargetDiameter = (targetDiameter: number) => {
    console.log("Mock: Set puller target diameter:", targetDiameter);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        puller_state: { ...prev.puller_state, target_diameter: targetDiameter },
      };
    });
  };

  const setPullerRegulationMode = (regulationMode: PullerRegulation) => {
    console.log("Mock: Set puller regulation mode:", regulationMode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        puller_state: { ...prev.puller_state, regulation: regulationMode },
      };
    });
  };

  const setPullerForward = (forward: boolean) => {
    console.log("Mock: Set puller forward:", forward);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        puller_state: { ...prev.puller_state, forward },
      };
    });
  };

  const setPullerGearRatio = (gearRatio: GearRatio) => {
    console.log("Mock: Set puller gear ratio:", gearRatio);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        puller_state: {
          ...prev.puller_state,
          gear_ratio: gearRatio,
          target_speed: 0,
        },
      };
    });
  };

  const setSpoolAutomaticRequiredMeters = (meters: number) => {
    console.log("Mock: Set spool automatic required meters:", meters);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_automatic_action_state: {
          ...prev.spool_automatic_action_state,
          spool_required_meters: meters,
        },
      };
    });
  };

  const setSpoolAutomaticAction = (mode: SpoolAutomaticActionMode) => {
    console.log("Mock: Set spool automatic action:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_automatic_action_state: {
          ...prev.spool_automatic_action_state,
          spool_automatic_action_mode: mode,
        },
      };
    });
  };

  const resetSpoolProgress = () => {
    console.log("Mock: Reset spool progress");
  };

  const setSpoolRegulationMode = (mode: SpoolRegulationMode) => {
    console.log("Mock: Set spool regulation mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          regulation_mode: mode,
        },
      };
    });
  };

  const setSpoolMinMaxMinSpeed = (speed: number) => {
    console.log("Mock: Set spool min max min speed:", speed);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          minmax_min_speed: speed,
        },
      };
    });
  };

  const setSpoolMinMaxMaxSpeed = (speed: number) => {
    console.log("Mock: Set spool min max max speed:", speed);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          minmax_max_speed: speed,
        },
      };
    });
  };

  const setSpoolForward = (forward: boolean) => {
    console.log("Mock: Set spool forward:", forward);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          forward,
        },
      };
    });
  };

  const setSpoolAdaptiveTensionTarget = (value: number) => {
    console.log("Mock: Set spool adaptive tension target:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          adaptive_tension_target: value,
        },
      };
    });
  };

  const setSpoolAdaptiveRadiusLearningRate = (value: number) => {
    console.log("Mock: Set spool adaptive radius learning rate:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          adaptive_radius_learning_rate: value,
        },
      };
    });
  };

  const setSpoolAdaptiveMaxSpeedMultiplier = (value: number) => {
    console.log("Mock: Set spool adaptive max speed multiplier:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          adaptive_max_speed_multiplier: value,
        },
      };
    });
  };

  const setSpoolAdaptiveAccelerationFactor = (value: number) => {
    console.log("Mock: Set spool adaptive acceleration factor:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          adaptive_acceleration_factor: value,
        },
      };
    });
  };

  const setSpoolAdaptiveDeaccelerationUrgencyMultiplier = (value: number) => {
    console.log(
      "Mock: Set spool adaptive deacceleration urgency multiplier:",
      value,
    );
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        spool_speed_controller_state: {
          ...prev.spool_speed_controller_state,
          adaptive_deacceleration_urgency_multiplier: value,
        },
      };
    });
  };

  const setConnectedMachine = (machineIdentificationUnique: any) => {
    console.log("Mock: Set connected machine:", machineIdentificationUnique);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        connected_machine_state: {
          ...prev.connected_machine_state,
          machine_identification_unique: machineIdentificationUnique,
        },
      };
    });
  };

  const disconnectMachine = (machineIdentificationUnique: any) => {
    console.log("Mock: Disconnect machine:", machineIdentificationUnique);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        connected_machine_state: {
          ...prev.connected_machine_state,
          machine_identification_unique: null,
        },
      };
    });
  };

  const setStepper2Mode = (mode: StepperMode) => {
    console.log("Mock: Set stepper 2 mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        stepper_state: { ...prev.stepper_state, stepper2_mode: mode },
      };
    });
  };

  const setStepper34Mode = (mode: StepperMode) => {
    console.log("Mock: Set stepper 3&4 mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        stepper_state: { ...prev.stepper_state, stepper34_mode: mode },
      };
    });
  };

  const setCuttingUnitMode = (mode: StepperMode) => {
    console.log("Mock: Set cutting unit mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        stepper_state: { ...prev.stepper_state, cutting_unit_mode: mode },
      };
    });
  };

  const setHeatingMode = (mode: HeatingMode) => {
    console.log("Mock: Set heating mode:", mode);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        heating_state: { heating_mode: mode },
      };
    });
  };

  const setTemperature1Min = (min: number) => {
    console.log("Mock: Set temperature 1 min:", min);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        quality_control_state: {
          ...prev.quality_control_state,
          temperature1: {
            ...prev.quality_control_state.temperature1,
            min_temperature: min,
          },
        },
      };
    });
  };

  const setTemperature1Max = (max: number) => {
    console.log("Mock: Set temperature 1 max:", max);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        quality_control_state: {
          ...prev.quality_control_state,
          temperature1: {
            ...prev.quality_control_state.temperature1,
            max_temperature: max,
          },
        },
      };
    });
  };

  const setTemperature2Min = (min: number) => {
    console.log("Mock: Set temperature 2 min:", min);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        quality_control_state: {
          ...prev.quality_control_state,
          temperature2: {
            ...prev.quality_control_state.temperature2,
            min_temperature: min,
          },
        },
      };
    });
  };

  const setTemperature2Max = (max: number) => {
    console.log("Mock: Set temperature 2 max:", max);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        quality_control_state: {
          ...prev.quality_control_state,
          temperature2: {
            ...prev.quality_control_state.temperature2,
            max_temperature: max,
          },
        },
      };
    });
  };

  const setStepper3Master = (value: number) => {
    console.log("Mock: Set stepper 3 master:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        motor_ratios_state: {
          ...prev.motor_ratios_state,
          stepper3_master: value,
        },
      };
    });
  };

  const setStepper3Slave = (value: number) => {
    console.log("Mock: Set stepper 3 slave:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        motor_ratios_state: {
          ...prev.motor_ratios_state,
          stepper3_slave: value,
        },
      };
    });
  };

  const setStepper4Master = (value: number) => {
    console.log("Mock: Set stepper 4 master:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        motor_ratios_state: {
          ...prev.motor_ratios_state,
          stepper4_master: value,
        },
      };
    });
  };

  const setStepper4Slave = (value: number) => {
    console.log("Mock: Set stepper 4 slave:", value);
    setLocalState((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        motor_ratios_state: {
          ...prev.motor_ratios_state,
          stepper4_slave: value,
        },
      };
    });
  };

  // Mock loading states
  const isLoading = false;
  const isDisabled = false;

  // Mock filtered machines
  const filteredMachines: any[] = [];
  const selectedMachine = null;

  return {
    // State (use local state if available, fallback to namespace state)
    state: localState || namespaceState,
    filteredMachines,
    selectedMachine,
    defaultState: namespaceDefaultState,

    // Live values (TimeSeries)
    traversePosition,
    pullerSpeed,
    slavePullerSpeed,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
    temperature1,
    temperature2,

    // Loading states
    isLoading,
    isDisabled,

    // Action functions (all mocked)
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
    setStepper2Mode,
    setStepper34Mode,
    setCuttingUnitMode,
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
