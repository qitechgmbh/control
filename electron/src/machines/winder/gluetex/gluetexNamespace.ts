/**
 * @file gluetexNamespace.ts
 * @description TypeScript implementation of Gluetex namespace with hardcoded test data (no backend).
 */

import { useMemo, useState, useEffect } from "react";
import {
  createTimeSeries,
  TimeSeries,
} from "@/lib/timeseries";
import { MachineIdentificationUnique } from "@/machines/types";

// ========== Type Definitions ==========

export type PullerRegulation = "Speed" | "Diameter";
export type GearRatio = "OneToOne" | "OneToFive" | "OneToTen";
export type Mode = "Standby" | "Hold" | "Pull" | "Wind";
export type SpoolAutomaticActionMode = "NoAction" | "Pull" | "Hold";
export type SpoolRegulationMode = "Adaptive" | "MinMax";
export type StepperMode = "Standby" | "Run";
export type HeatingMode = "Standby" | "Heating";

export function getGearRatioMultiplier(
  gearRatio: GearRatio | undefined,
): number {
  switch (gearRatio) {
    case "OneToOne":
      return 1.0;
    case "OneToFive":
      return 5.0;
    case "OneToTen":
      return 10.0;
    default:
      return 1.0;
  }
}

export type TraverseState = {
  limit_inner: number;
  limit_outer: number;
  position_in: number;
  position_out: number;
  is_going_in: boolean;
  is_going_out: boolean;
  is_homed: boolean;
  is_going_home: boolean;
  is_traversing: boolean;
  laserpointer: boolean;
  step_size: number;
  padding: number;
  can_go_in: boolean;
  can_go_out: boolean;
  can_go_home: boolean;
};

export type PullerState = {
  regulation: PullerRegulation;
  target_speed: number;
  target_diameter: number;
  forward: boolean;
  gear_ratio: GearRatio;
};

export type ModeState = {
  mode: Mode;
  can_wind: boolean;
};

export type TensionArmState = {
  zeroed: boolean;
};

export type SpoolSpeedControllerState = {
  regulation_mode: SpoolRegulationMode;
  minmax_min_speed: number;
  minmax_max_speed: number;
  adaptive_tension_target: number;
  adaptive_radius_learning_rate: number;
  adaptive_max_speed_multiplier: number;
  adaptive_acceleration_factor: number;
  adaptive_deacceleration_urgency_multiplier: number;
  forward: boolean;
};

export type SpoolAutomaticActionState = {
  spool_required_meters: number;
  spool_automatic_action_mode: SpoolAutomaticActionMode;
};

export type ConnectedMachineState = {
  machine_identification_unique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  } | null;
  is_available: boolean;
};

export type StepperState = {
  stepper2_mode: StepperMode;
  stepper34_mode: StepperMode;
  cutting_unit_mode: StepperMode;
};

export type HeatingState = {
  heating_mode: HeatingMode;
};

export type TemperatureState = {
  current_temperature: number;
  min_temperature: number;
  max_temperature: number;
};

export type QualityControlState = {
  temperature1: TemperatureState;
  temperature2: TemperatureState;
};

export type StateEvent = {
  is_default_state: boolean;
  traverse_state: TraverseState;
  puller_state: PullerState;
  mode_state: ModeState;
  tension_arm_state: TensionArmState;
  spool_speed_controller_state: SpoolSpeedControllerState;
  spool_automatic_action_state: SpoolAutomaticActionState;
  connected_machine_state: ConnectedMachineState;
  stepper_state: StepperState;
  heating_state: HeatingState;
  quality_control_state: QualityControlState;
};

export type GluetexNamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
  traversePosition: TimeSeries;
  pullerSpeed: TimeSeries;
  spoolRpm: TimeSeries;
  tensionArmAngle: TimeSeries;
  spoolProgress: TimeSeries;
  temperature1: TimeSeries;
  temperature2: TimeSeries;
};

// ========== Hardcoded Test Data ==========

const HARDCODED_STATE: StateEvent = {
  is_default_state: false,
  traverse_state: {
    limit_inner: 20,
    limit_outer: 160,
    position_in: 0,
    position_out: 180,
    is_going_in: false,
    is_going_out: false,
    is_homed: true,
    is_going_home: false,
    is_traversing: false,
    laserpointer: false,
    step_size: 1.5,
    padding: 5,
    can_go_in: true,
    can_go_out: true,
    can_go_home: true,
  },
  puller_state: {
    regulation: "Speed",
    target_speed: 15.0,
    target_diameter: 50.0,
    forward: true,
    gear_ratio: "OneToOne",
  },
  mode_state: {
    mode: "Standby",
    can_wind: true,
  },
  tension_arm_state: {
    zeroed: true,
  },
  spool_speed_controller_state: {
    regulation_mode: "Adaptive",
    minmax_min_speed: 5.0,
    minmax_max_speed: 50.0,
    adaptive_tension_target: 35.0,
    adaptive_radius_learning_rate: 0.1,
    adaptive_max_speed_multiplier: 2.0,
    adaptive_acceleration_factor: 0.5,
    adaptive_deacceleration_urgency_multiplier: 1.5,
    forward: true,
  },
  spool_automatic_action_state: {
    spool_required_meters: 100.0,
    spool_automatic_action_mode: "NoAction",
  },
  connected_machine_state: {
    machine_identification_unique: null,
    is_available: false,
  },
  stepper_state: {
    stepper2_mode: "Standby",
    stepper34_mode: "Standby",
    cutting_unit_mode: "Standby",
  },
  heating_state: {
    heating_mode: "Standby",
  },
  quality_control_state: {
    temperature1: {
      current_temperature: 85.0,
      min_temperature: 80.0,
      max_temperature: 90.0,
    },
    temperature2: {
      current_temperature: 125.0,
      min_temperature: 120.0,
      max_temperature: 130.0,
    },
  },
};

const DEFAULT_STATE: StateEvent = {
  is_default_state: true,
  traverse_state: {
    limit_inner: 10,
    limit_outer: 170,
    position_in: 0,
    position_out: 180,
    is_going_in: false,
    is_going_out: false,
    is_homed: false,
    is_going_home: false,
    is_traversing: false,
    laserpointer: false,
    step_size: 1.0,
    padding: 3,
    can_go_in: true,
    can_go_out: true,
    can_go_home: true,
  },
  puller_state: {
    regulation: "Speed",
    target_speed: 10.0,
    target_diameter: 40.0,
    forward: true,
    gear_ratio: "OneToOne",
  },
  mode_state: {
    mode: "Standby",
    can_wind: true,
  },
  tension_arm_state: {
    zeroed: false,
  },
  spool_speed_controller_state: {
    regulation_mode: "Adaptive",
    minmax_min_speed: 5.0,
    minmax_max_speed: 40.0,
    adaptive_tension_target: 30.0,
    adaptive_radius_learning_rate: 0.1,
    adaptive_max_speed_multiplier: 2.0,
    adaptive_acceleration_factor: 0.5,
    adaptive_deacceleration_urgency_multiplier: 1.5,
    forward: true,
  },
  spool_automatic_action_state: {
    spool_required_meters: 50.0,
    spool_automatic_action_mode: "NoAction",
  },
  connected_machine_state: {
    machine_identification_unique: null,
    is_available: false,
  },
  stepper_state: {
    stepper2_mode: "Standby",
    stepper34_mode: "Standby",
    cutting_unit_mode: "Standby",
  },
  heating_state: {
    heating_mode: "Standby",
  },
  quality_control_state: {
    temperature1: {
      current_temperature: 85.0,
      min_temperature: 75.0,
      max_temperature: 95.0,
    },
    temperature2: {
      current_temperature: 125.0,
      min_temperature: 115.0,
      max_temperature: 135.0,
    },
  },
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

/**
 * Hook for Gluetex namespace with hardcoded test data
 * This simulates the backend behavior with fake data for testing
 */
export function useGluetexNamespace(
  _machine_identification_unique: MachineIdentificationUnique,
): GluetexNamespaceStore {
  const [state, setState] = useState<StateEvent>(HARDCODED_STATE);

  // Create time series with simulated data
  const traversePosition = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    // Add some initial simulated values
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 90 + Math.sin(i / 5) * 30; // Oscillating between 60 and 120
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const pullerSpeed = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 14.5 + Math.random() * 1; // Around 15 m/min with variation
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const spoolRpm = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 25 + Math.random() * 5; // Around 25-30 rpm
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const tensionArmAngle = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 35 + Math.sin(i / 3) * 10; // Oscillating around 35 degrees
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const spoolProgress = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = i * 0.5; // Gradually increasing progress
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const temperature1 = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 85.0 + Math.sin(i / 10) * 3; // Oscillating around 85°C
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  const temperature2 = useMemo(() => {
    const { initialTimeSeries, insert } = createTimeSeries(
      TWENTY_MILLISECOND,
      ONE_SECOND,
      FIVE_SECOND,
      ONE_HOUR,
    );
    let series = initialTimeSeries;
    
    const now = Date.now();
    for (let i = 0; i < 50; i++) {
      const timestamp = now - (50 - i) * 100;
      const value = 125.0 + Math.sin(i / 8) * 2; // Oscillating around 125°C
      series = insert(series, { value, timestamp });
    }
    
    return series;
  }, []);

  // Simulate live data updates
  useEffect(() => {
    const interval = setInterval(() => {
      // Update traverse position to simulate movement
      const currentPos = state.traverse_state.position_in;
      setState((prev) => ({
        ...prev,
        traverse_state: {
          ...prev.traverse_state,
          position_in: (currentPos + 0.5) % 180, // Slowly moving
        },
      }));
    }, 1000);

    return () => clearInterval(interval);
  }, [state.traverse_state.position_in]);

  return {
    state,
    defaultState: DEFAULT_STATE,
    traversePosition,
    pullerSpeed,
    spoolRpm,
    tensionArmAngle,
    spoolProgress,
    temperature1,
    temperature2,
  };
}
