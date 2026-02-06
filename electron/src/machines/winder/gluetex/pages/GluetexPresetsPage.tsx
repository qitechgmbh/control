import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import { gluetex } from "@/machines/properties";
import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import {
  pullerStateSchema,
  spoolSpeedControllerStateSchema,
  heatingStatesSchema,
  heatingPidStatesSchema,
  addonMotorStateSchema,
  addonMotor5StateSchema,
  slavePullerStateSchema,
  tensionArmMonitorStateSchema,
} from "../state/gluetexNamespace";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";

// Define the preset data schema for Gluetex
// Including: winder (traverse, puller, spool), addons (motors 3,4,5, slave puller, tension arm monitor), temp (heating), and settings (quality control)
const gluetexPresetDataSchema = z
  .object({
    // Winder settings - Traverse
    traverse_state: z
      .object({
        limit_inner: z.number(),
        limit_outer: z.number(),
        step_size: z.number(),
        padding: z.number(),
        laserpointer: z.boolean(),
      })
      .partial(),

    // Winder settings - Puller
    puller_state: pullerStateSchema.partial(),

    // Winder settings - Spool speed controller
    spool_speed_controller_state: spoolSpeedControllerStateSchema.partial(),

    // Temperature/Heating settings
    heating_states: heatingStatesSchema.partial(),
    heating_pid_settings: heatingPidStatesSchema.partial(),

    // Addon settings - Motors
    addon_motor_3_state: addonMotor5StateSchema.partial(),
    addon_motor_4_state: addonMotorStateSchema.partial(),
    addon_motor_5_state: addonMotorStateSchema.partial(),

    // Addon settings - Slave puller
    slave_puller_state: slavePullerStateSchema.partial(),

    // Addon settings - Tension arm monitor
    tension_arm_monitor_state: tensionArmMonitorStateSchema.partial(),

    // Settings - Quality control (local state)
    quality_control_state: z
      .object({
        optris1: z
          .object({
            min_voltage: z.number(),
            max_voltage: z.number(),
          })
          .partial(),
        optris2: z
          .object({
            min_voltage: z.number(),
            max_voltage: z.number(),
          })
          .partial(),
      })
      .partial(),
  })
  .partial();

type GluetexPreset = z.infer<typeof gluetexPresetDataSchema>;

const schemas = new Map([[1, gluetexPresetDataSchema]]);

// Preview entries for the preset cards
const previewEntries: PresetPreviewEntries<GluetexPreset> = [
  // Winder - Traverse
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.limit_inner?.toFixed(1) ?? "N/A",
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.limit_outer?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Step Size",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.step_size?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.traverse_state?.padding?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Winder - Puller
  {
    name: "Puller Regulation",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.regulation ?? "N/A",
  },
  {
    name: "Puller Direction",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.forward !== undefined
        ? data.puller_state.forward
          ? "Forward"
          : "Backward"
        : "N/A",
  },
  {
    name: "Puller Gear Ratio",
    renderValue: (data: GluetexPreset) => {
      const ratio = data.puller_state?.gear_ratio;
      if (ratio === "OneToOne") return "1:1";
      if (ratio === "OneToFive") return "1:5";
      if (ratio === "OneToTen") return "1:10";
      return "N/A";
    },
  },
  {
    name: "Puller Target Speed",
    unit: "m/min",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.target_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Puller Target Diameter",
    unit: "mm",
    renderValue: (data: GluetexPreset) =>
      data.puller_state?.target_diameter?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Winder - Spool
  {
    name: "Spool Regulation",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.regulation_mode ?? "N/A",
  },
  {
    name: "Spool Direction",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.forward !== undefined
        ? data.spool_speed_controller_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Spool Min Speed",
    unit: "rpm",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.minmax_min_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Spool Max Speed",
    unit: "rpm",
    renderValue: (data: GluetexPreset) =>
      data.spool_speed_controller_state?.minmax_max_speed?.toFixed(2) ?? "N/A",
  },
  previewSeparator,
  // Temperature - Heating zones
  {
    name: "Heating Enabled",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.enabled !== undefined
        ? data.heating_states.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Zone 1 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_1?.target_temperature?.toFixed(1) ?? "N/A",
  },
  {
    name: "Zone 2 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_2?.target_temperature?.toFixed(1) ?? "N/A",
  },
  {
    name: "Zone 3 Target Temp",
    unit: "C",
    renderValue: (data: GluetexPreset) =>
      data.heating_states?.zone_3?.target_temperature?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // Addons - Motors
  {
    name: "Motor 3 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_3_state?.enabled !== undefined
        ? data.addon_motor_3_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 3 Direction",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_3_state?.forward !== undefined
        ? data.addon_motor_3_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Motor 3 Master Ratio",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_3_state?.master_ratio?.toFixed(2) ?? "N/A",
  },
  {
    name: "Motor 4 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_4_state?.enabled !== undefined
        ? data.addon_motor_4_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 4 Direction",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_4_state?.forward !== undefined
        ? data.addon_motor_4_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  {
    name: "Motor 5 Enabled",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_5_state?.enabled !== undefined
        ? data.addon_motor_5_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Motor 5 Direction",
    renderValue: (data: GluetexPreset) =>
      data.addon_motor_5_state?.forward !== undefined
        ? data.addon_motor_5_state.forward
          ? "Forward"
          : "Reverse"
        : "N/A",
  },
  previewSeparator,
  // Addons - Slave puller
  {
    name: "Slave Puller Enabled",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.enabled !== undefined
        ? data.slave_puller_state.enabled
          ? "Yes"
          : "No"
        : "N/A",
  },
  {
    name: "Slave Puller Target Angle",
    unit: "deg",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.target_angle?.toFixed(1) ?? "N/A",
  },
  {
    name: "Slave Puller Sensitivity",
    unit: "deg",
    renderValue: (data: GluetexPreset) =>
      data.slave_puller_state?.sensitivity?.toFixed(1) ?? "N/A",
  },
];

export function GluetexPresetsPage() {
  const {
    state,
    defaultState,

    // Traverse
    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    enableTraverseLaserpointer,

    // Puller
    setPullerRegulationMode,
    setPullerTargetDiameter,
    setPullerForward,
    setPullerTargetSpeed,
    setPullerGearRatio,

    // Spool
    setSpoolRegulationMode,
    setSpoolForward,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,

    // Heating/Temperature
    setHeatingMode,
    setHeatingZone1Temperature,
    setHeatingZone2Temperature,
    setHeatingZone3Temperature,
    setHeatingZone4Temperature,
    setHeatingZone5Temperature,
    setHeatingZone6Temperature,
    setHeatingPid,

    // Addons - Motors
    setStepper3Mode,
    setStepper3Master,
    setStepper3Slave,
    setStepper3Forward,
    setStepper4Mode,
    setStepper4Master,
    setStepper4Slave,
    setStepper4Forward,
    setStepper5Mode,
    setStepper5Master,
    setStepper5Slave,
    setStepper5Forward,
    setStepper3Konturlaenge,
    setStepper3Pause,

    // Addons - Slave puller
    setSlavePullerEnabled,
    setSlavePullerForward,
    setSlavePullerTargetAngle,
    setSlavePullerSensitivity,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,

    // Addons - Tension arm monitor
    setTensionArmMonitorEnabled,
    setTensionArmMonitorMinAngle,
    setTensionArmMonitorMaxAngle,

    // Settings - Quality control
    setOptris1Min,
    setOptris1Max,
    setOptris2Min,
    setOptris2Max,
  } = useGluetex();

  const applyPreset = (preset: Preset<GluetexPreset>) => {
    // Apply traverse settings
    if (preset.data?.traverse_state?.limit_inner !== undefined) {
      setTraverseLimitInner(preset.data.traverse_state.limit_inner);
    }
    if (preset.data?.traverse_state?.limit_outer !== undefined) {
      setTraverseLimitOuter(preset.data.traverse_state.limit_outer);
    }
    if (preset.data?.traverse_state?.step_size !== undefined) {
      setTraverseStepSize(preset.data.traverse_state.step_size);
    }
    if (preset.data?.traverse_state?.padding !== undefined) {
      setTraversePadding(preset.data.traverse_state.padding);
    }
    if (preset.data?.traverse_state?.laserpointer !== undefined) {
      enableTraverseLaserpointer(preset.data.traverse_state.laserpointer);
    }

    // Apply puller settings
    if (preset.data?.puller_state?.regulation !== undefined) {
      setPullerRegulationMode(preset.data.puller_state.regulation);
    }
    if (preset.data?.puller_state?.forward !== undefined) {
      setPullerForward(preset.data.puller_state.forward);
    }
    if (preset.data?.puller_state?.target_speed !== undefined) {
      setPullerTargetSpeed(preset.data.puller_state.target_speed);
    }
    if (preset.data?.puller_state?.gear_ratio !== undefined) {
      setPullerGearRatio(preset.data.puller_state.gear_ratio);
    }
    if (preset.data?.puller_state?.target_diameter !== undefined) {
      setPullerTargetDiameter(preset.data.puller_state.target_diameter);
    }

    // Apply spool settings
    if (
      preset.data?.spool_speed_controller_state?.regulation_mode !== undefined
    ) {
      setSpoolRegulationMode(
        preset.data.spool_speed_controller_state.regulation_mode,
      );
    }
    if (preset.data?.spool_speed_controller_state?.forward !== undefined) {
      setSpoolForward(preset.data.spool_speed_controller_state.forward);
    }
    if (
      preset.data?.spool_speed_controller_state?.minmax_min_speed !== undefined
    ) {
      setSpoolMinMaxMinSpeed(
        preset.data.spool_speed_controller_state.minmax_min_speed,
      );
    }
    if (
      preset.data?.spool_speed_controller_state?.minmax_max_speed !== undefined
    ) {
      setSpoolMinMaxMaxSpeed(
        preset.data.spool_speed_controller_state.minmax_max_speed,
      );
    }
    if (
      preset.data?.spool_speed_controller_state?.adaptive_tension_target !==
      undefined
    ) {
      setSpoolAdaptiveTensionTarget(
        preset.data.spool_speed_controller_state.adaptive_tension_target,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_radius_learning_rate !== undefined
    ) {
      setSpoolAdaptiveRadiusLearningRate(
        preset.data.spool_speed_controller_state.adaptive_radius_learning_rate,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_max_speed_multiplier !== undefined
    ) {
      setSpoolAdaptiveMaxSpeedMultiplier(
        preset.data.spool_speed_controller_state.adaptive_max_speed_multiplier,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_acceleration_factor !== undefined
    ) {
      setSpoolAdaptiveAccelerationFactor(
        preset.data.spool_speed_controller_state.adaptive_acceleration_factor,
      );
    }
    if (
      preset.data?.spool_speed_controller_state
        ?.adaptive_deacceleration_urgency_multiplier !== undefined
    ) {
      setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
        preset.data.spool_speed_controller_state
          .adaptive_deacceleration_urgency_multiplier,
      );
    }

    // Apply heating settings
    if (preset.data?.heating_states?.enabled !== undefined) {
      setHeatingMode(
        preset.data.heating_states.enabled ? "Heating" : "Standby",
      );
    }
    if (preset.data?.heating_states?.zone_1?.target_temperature !== undefined) {
      setHeatingZone1Temperature(
        preset.data.heating_states.zone_1.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_2?.target_temperature !== undefined) {
      setHeatingZone2Temperature(
        preset.data.heating_states.zone_2.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_3?.target_temperature !== undefined) {
      setHeatingZone3Temperature(
        preset.data.heating_states.zone_3.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_4?.target_temperature !== undefined) {
      setHeatingZone4Temperature(
        preset.data.heating_states.zone_4.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_5?.target_temperature !== undefined) {
      setHeatingZone5Temperature(
        preset.data.heating_states.zone_5.target_temperature,
      );
    }
    if (preset.data?.heating_states?.zone_6?.target_temperature !== undefined) {
      setHeatingZone6Temperature(
        preset.data.heating_states.zone_6.target_temperature,
      );
    }

    // Apply heating PID settings
    if (preset.data?.heating_pid_settings?.zone_1) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_1;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_1", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_2) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_2;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_2", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_3) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_3;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_3", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_4) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_4;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_4", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_5) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_5;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_5", kp, ki, kd);
      }
    }
    if (preset.data?.heating_pid_settings?.zone_6) {
      const { kp, ki, kd } = preset.data.heating_pid_settings.zone_6;
      if (kp !== undefined && ki !== undefined && kd !== undefined) {
        setHeatingPid("zone_6", kp, ki, kd);
      }
    }

    // Apply addon motor 3 settings
    if (preset.data?.addon_motor_3_state?.enabled !== undefined) {
      setStepper3Mode(
        preset.data.addon_motor_3_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_3_state?.forward !== undefined) {
      setStepper3Forward(preset.data.addon_motor_3_state.forward);
    }
    if (preset.data?.addon_motor_3_state?.master_ratio !== undefined) {
      setStepper3Master(preset.data.addon_motor_3_state.master_ratio);
    }
    if (preset.data?.addon_motor_3_state?.slave_ratio !== undefined) {
      setStepper3Slave(preset.data.addon_motor_3_state.slave_ratio);
    }
    if (preset.data?.addon_motor_3_state?.konturlaenge_mm !== undefined) {
      setStepper3Konturlaenge(preset.data.addon_motor_3_state.konturlaenge_mm);
    }
    if (preset.data?.addon_motor_3_state?.pause_mm !== undefined) {
      setStepper3Pause(preset.data.addon_motor_3_state.pause_mm);
    }

    // Apply addon motor 4 settings
    if (preset.data?.addon_motor_4_state?.enabled !== undefined) {
      setStepper4Mode(
        preset.data.addon_motor_4_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_4_state?.forward !== undefined) {
      setStepper4Forward(preset.data.addon_motor_4_state.forward);
    }
    if (preset.data?.addon_motor_4_state?.master_ratio !== undefined) {
      setStepper4Master(preset.data.addon_motor_4_state.master_ratio);
    }
    if (preset.data?.addon_motor_4_state?.slave_ratio !== undefined) {
      setStepper4Slave(preset.data.addon_motor_4_state.slave_ratio);
    }

    // Apply addon motor 5 settings
    if (preset.data?.addon_motor_5_state?.enabled !== undefined) {
      setStepper5Mode(
        preset.data.addon_motor_5_state.enabled ? "Run" : "Standby",
      );
    }
    if (preset.data?.addon_motor_5_state?.forward !== undefined) {
      setStepper5Forward(preset.data.addon_motor_5_state.forward);
    }
    if (preset.data?.addon_motor_5_state?.master_ratio !== undefined) {
      setStepper5Master(preset.data.addon_motor_5_state.master_ratio);
    }
    if (preset.data?.addon_motor_5_state?.slave_ratio !== undefined) {
      setStepper5Slave(preset.data.addon_motor_5_state.slave_ratio);
    }

    // Apply slave puller settings
    if (preset.data?.slave_puller_state?.enabled !== undefined) {
      setSlavePullerEnabled(preset.data.slave_puller_state.enabled);
    }
    if (preset.data?.slave_puller_state?.forward !== undefined) {
      setSlavePullerForward(preset.data.slave_puller_state.forward);
    }
    if (preset.data?.slave_puller_state?.target_angle !== undefined) {
      setSlavePullerTargetAngle(preset.data.slave_puller_state.target_angle);
    }
    if (preset.data?.slave_puller_state?.sensitivity !== undefined) {
      setSlavePullerSensitivity(preset.data.slave_puller_state.sensitivity);
    }
    if (preset.data?.slave_puller_state?.min_speed_factor !== undefined) {
      setSlavePullerMinSpeedFactor(
        preset.data.slave_puller_state.min_speed_factor,
      );
    }
    if (preset.data?.slave_puller_state?.max_speed_factor !== undefined) {
      setSlavePullerMaxSpeedFactor(
        preset.data.slave_puller_state.max_speed_factor,
      );
    }

    // Apply tension arm monitor settings
    if (preset.data?.tension_arm_monitor_state?.enabled !== undefined) {
      setTensionArmMonitorEnabled(
        preset.data.tension_arm_monitor_state.enabled,
      );
    }
    if (preset.data?.tension_arm_monitor_state?.min_angle !== undefined) {
      setTensionArmMonitorMinAngle(
        preset.data.tension_arm_monitor_state.min_angle,
      );
    }
    if (preset.data?.tension_arm_monitor_state?.max_angle !== undefined) {
      setTensionArmMonitorMaxAngle(
        preset.data.tension_arm_monitor_state.max_angle,
      );
    }

    // Apply quality control settings
    if (
      preset.data?.quality_control_state?.optris1?.min_voltage !== undefined
    ) {
      setOptris1Min(preset.data.quality_control_state.optris1.min_voltage);
    }
    if (
      preset.data?.quality_control_state?.optris1?.max_voltage !== undefined
    ) {
      setOptris1Max(preset.data.quality_control_state.optris1.max_voltage);
    }
    if (
      preset.data?.quality_control_state?.optris2?.min_voltage !== undefined
    ) {
      setOptris2Min(preset.data.quality_control_state.optris2.min_voltage);
    }
    if (
      preset.data?.quality_control_state?.optris2?.max_voltage !== undefined
    ) {
      setOptris2Max(preset.data.quality_control_state.optris2.max_voltage);
    }
  };

  const toPresetData = (s?: typeof state): GluetexPreset => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state
      ? {
          regulation: s.puller_state.regulation,
          target_speed: s.puller_state.target_speed,
          target_diameter: s.puller_state.target_diameter,
          forward: s.puller_state.forward,
          gear_ratio: s.puller_state.gear_ratio,
        }
      : undefined,
    spool_speed_controller_state: s?.spool_speed_controller_state
      ? {
          regulation_mode: s.spool_speed_controller_state.regulation_mode,
          minmax_min_speed: s.spool_speed_controller_state.minmax_min_speed,
          minmax_max_speed: s.spool_speed_controller_state.minmax_max_speed,
          adaptive_tension_target:
            s.spool_speed_controller_state.adaptive_tension_target,
          adaptive_radius_learning_rate:
            s.spool_speed_controller_state.adaptive_radius_learning_rate,
          adaptive_max_speed_multiplier:
            s.spool_speed_controller_state.adaptive_max_speed_multiplier,
          adaptive_acceleration_factor:
            s.spool_speed_controller_state.adaptive_acceleration_factor,
          adaptive_deacceleration_urgency_multiplier:
            s.spool_speed_controller_state
              .adaptive_deacceleration_urgency_multiplier,
          forward: s.spool_speed_controller_state.forward,
        }
      : undefined,
    heating_states: s?.heating_states
      ? {
          enabled: s.heating_states.enabled,
          zone_1: s.heating_states.zone_1
            ? {
                target_temperature: s.heating_states.zone_1.target_temperature,
                wiring_error: s.heating_states.zone_1.wiring_error,
                autotuning_active: s.heating_states.zone_1.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_1.autotuning_progress,
              }
            : undefined,
          zone_2: s.heating_states.zone_2
            ? {
                target_temperature: s.heating_states.zone_2.target_temperature,
                wiring_error: s.heating_states.zone_2.wiring_error,
                autotuning_active: s.heating_states.zone_2.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_2.autotuning_progress,
              }
            : undefined,
          zone_3: s.heating_states.zone_3
            ? {
                target_temperature: s.heating_states.zone_3.target_temperature,
                wiring_error: s.heating_states.zone_3.wiring_error,
                autotuning_active: s.heating_states.zone_3.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_3.autotuning_progress,
              }
            : undefined,
          zone_4: s.heating_states.zone_4
            ? {
                target_temperature: s.heating_states.zone_4.target_temperature,
                wiring_error: s.heating_states.zone_4.wiring_error,
                autotuning_active: s.heating_states.zone_4.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_4.autotuning_progress,
              }
            : undefined,
          zone_5: s.heating_states.zone_5
            ? {
                target_temperature: s.heating_states.zone_5.target_temperature,
                wiring_error: s.heating_states.zone_5.wiring_error,
                autotuning_active: s.heating_states.zone_5.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_5.autotuning_progress,
              }
            : undefined,
          zone_6: s.heating_states.zone_6
            ? {
                target_temperature: s.heating_states.zone_6.target_temperature,
                wiring_error: s.heating_states.zone_6.wiring_error,
                autotuning_active: s.heating_states.zone_6.autotuning_active,
                autotuning_progress:
                  s.heating_states.zone_6.autotuning_progress,
              }
            : undefined,
        }
      : undefined,
    heating_pid_settings: s?.heating_pid_settings
      ? {
          zone_1: s.heating_pid_settings.zone_1
            ? {
                kp: s.heating_pid_settings.zone_1.kp,
                ki: s.heating_pid_settings.zone_1.ki,
                kd: s.heating_pid_settings.zone_1.kd,
                zone: s.heating_pid_settings.zone_1.zone,
              }
            : undefined,
          zone_2: s.heating_pid_settings.zone_2
            ? {
                kp: s.heating_pid_settings.zone_2.kp,
                ki: s.heating_pid_settings.zone_2.ki,
                kd: s.heating_pid_settings.zone_2.kd,
                zone: s.heating_pid_settings.zone_2.zone,
              }
            : undefined,
          zone_3: s.heating_pid_settings.zone_3
            ? {
                kp: s.heating_pid_settings.zone_3.kp,
                ki: s.heating_pid_settings.zone_3.ki,
                kd: s.heating_pid_settings.zone_3.kd,
                zone: s.heating_pid_settings.zone_3.zone,
              }
            : undefined,
          zone_4: s.heating_pid_settings.zone_4
            ? {
                kp: s.heating_pid_settings.zone_4.kp,
                ki: s.heating_pid_settings.zone_4.ki,
                kd: s.heating_pid_settings.zone_4.kd,
                zone: s.heating_pid_settings.zone_4.zone,
              }
            : undefined,
          zone_5: s.heating_pid_settings.zone_5
            ? {
                kp: s.heating_pid_settings.zone_5.kp,
                ki: s.heating_pid_settings.zone_5.ki,
                kd: s.heating_pid_settings.zone_5.kd,
                zone: s.heating_pid_settings.zone_5.zone,
              }
            : undefined,
          zone_6: s.heating_pid_settings.zone_6
            ? {
                kp: s.heating_pid_settings.zone_6.kp,
                ki: s.heating_pid_settings.zone_6.ki,
                kd: s.heating_pid_settings.zone_6.kd,
                zone: s.heating_pid_settings.zone_6.zone,
              }
            : undefined,
        }
      : undefined,
    addon_motor_3_state: s?.addon_motor_3_state
      ? {
          enabled: s.addon_motor_3_state.enabled,
          forward: s.addon_motor_3_state.forward,
          master_ratio: s.addon_motor_3_state.master_ratio,
          slave_ratio: s.addon_motor_3_state.slave_ratio,
          konturlaenge_mm: s.addon_motor_3_state.konturlaenge_mm,
          pause_mm: s.addon_motor_3_state.pause_mm,
        }
      : undefined,
    addon_motor_4_state: s?.addon_motor_4_state
      ? {
          enabled: s.addon_motor_4_state.enabled,
          forward: s.addon_motor_4_state.forward,
          master_ratio: s.addon_motor_4_state.master_ratio,
          slave_ratio: s.addon_motor_4_state.slave_ratio,
        }
      : undefined,
    addon_motor_5_state: s?.addon_motor_5_state
      ? {
          enabled: s.addon_motor_5_state.enabled,
          forward: s.addon_motor_5_state.forward,
          master_ratio: s.addon_motor_5_state.master_ratio,
          slave_ratio: s.addon_motor_5_state.slave_ratio,
        }
      : undefined,
    slave_puller_state: s?.slave_puller_state
      ? {
          enabled: s.slave_puller_state.enabled,
          forward: s.slave_puller_state.forward,
          target_angle: s.slave_puller_state.target_angle,
          sensitivity: s.slave_puller_state.sensitivity,
          min_speed_factor: s.slave_puller_state.min_speed_factor,
          max_speed_factor: s.slave_puller_state.max_speed_factor,
        }
      : undefined,
    tension_arm_monitor_state: s?.tension_arm_monitor_state
      ? {
          enabled: s.tension_arm_monitor_state.enabled,
          min_angle: s.tension_arm_monitor_state.min_angle,
          max_angle: s.tension_arm_monitor_state.max_angle,
        }
      : undefined,
    quality_control_state: s?.quality_control_state
      ? {
          optris1: s.quality_control_state.optris1
            ? {
                min_voltage: s.quality_control_state.optris1.min_voltage,
                max_voltage: s.quality_control_state.optris1.max_voltage,
              }
            : undefined,
          optris2: s.quality_control_state.optris2
            ? {
                min_voltage: s.quality_control_state.optris2.min_voltage,
                max_voltage: s.quality_control_state.optris2.max_voltage,
              }
            : undefined,
        }
      : undefined,
  });

  const defaults = toPresetData(defaultState);

  return (
    <PresetsPage
      machine_identification={gluetex.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      defaultState={defaults}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
    />
  );
}
