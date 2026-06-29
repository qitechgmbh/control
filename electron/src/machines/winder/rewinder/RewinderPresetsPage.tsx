import React from "react";
import { useRewinder } from "./useRewinder";
import { rewinder } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import {
  pullerStateSchema,
  takeupSpoolStateSchema,
  sourceSpoolStateSchema,
  traverseStateSchema,
  tensionArmControlStateSchema,
  prepareControlStateSchema,
  rewindAutomaticActionStateSchema,
} from "./rewinderNamespace";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";

const rewinderPresetDataSchema = z
  .object({
    traverse_state: traverseStateSchema
      .pick({
        limit_inner: true,
        limit_outer: true,
        step_size: true,
        padding: true,
        laserpointer: true,
      })
      .partial(),
    puller_state: pullerStateSchema.partial(),
    takeup_spool_state: takeupSpoolStateSchema.partial(),
    source_spool_state: sourceSpoolStateSchema.partial(),
    takeup_tension_arm_control_state:
      tensionArmControlStateSchema.partial(),
    source_tension_arm_control_state:
      tensionArmControlStateSchema.partial(),
    prepare_control_state: prepareControlStateSchema.partial(),
    rewind_automatic_action_state:
      rewindAutomaticActionStateSchema.partial(),
  })
  .partial();

type RewinderPresetData = z.infer<typeof rewinderPresetDataSchema>;

const schemas = new Map([[1, rewinderPresetDataSchema]]);

const previewEntries: PresetPreviewEntries<RewinderPresetData> = [
  // --- Traverse ---
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (data: RewinderPresetData) =>
      data.traverse_state?.limit_inner?.toFixed(1) ?? "N/A",
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (data: RewinderPresetData) =>
      data.traverse_state?.limit_outer?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Step Size",
    unit: "mm",
    renderValue: (data: RewinderPresetData) =>
      data.traverse_state?.step_size?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (data: RewinderPresetData) =>
      data.traverse_state?.padding?.toFixed(1) ?? "N/A",
  },
  {
    name: "Laserpointer",
    renderValue: (data: RewinderPresetData) =>
      data.traverse_state?.laserpointer !== undefined
        ? data.traverse_state.laserpointer
          ? "On"
          : "Off"
        : "N/A",
  },
  previewSeparator,
  // --- Puller ---
  {
    name: "Puller Target Speed",
    unit: "m/min",
    renderValue: (data: RewinderPresetData) =>
      data.puller_state?.target_speed?.toFixed(2) ?? "N/A",
  },
  previewSeparator,
  // --- Takeup Spool ---
  {
    name: "Takeup Regulation",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.regulation_mode ?? "N/A",
  },
  {
    name: "Takeup Min Speed",
    unit: "rpm",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.minmax_min_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Takeup Max Speed",
    unit: "rpm",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.minmax_max_speed?.toFixed(2) ?? "N/A",
  },
  {
    name: "Takeup Adaptive Tension Target",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.adaptive_tension_target?.toFixed(2) ?? "N/A",
  },
  {
    name: "Takeup Adaptive Learning Rate",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.adaptive_radius_learning_rate?.toFixed(2) ??
      "N/A",
  },
  {
    name: "Takeup Adaptive Max Speed Mult.",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.adaptive_max_speed_multiplier?.toFixed(2) ??
      "N/A",
  },
  {
    name: "Takeup Adaptive Accel. Factor",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.adaptive_acceleration_factor?.toFixed(2) ??
      "N/A",
  },
  {
    name: "Takeup Adaptive Deaccel. Urgency",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_spool_state?.adaptive_deacceleration_urgency_multiplier?.toFixed(
        2,
      ) ?? "N/A",
  },
  previewSeparator,
  // --- Source Spool ---
  {
    name: "Source Adaptive Tension Target",
    renderValue: (data: RewinderPresetData) =>
      data.source_spool_state?.adaptive_tension_target?.toFixed(2) ?? "N/A",
  },
  previewSeparator,
  // --- Tension Arm Control ---
  {
    name: "Takeup Tension Arm Target Angle",
    unit: "deg",
    renderValue: (data: RewinderPresetData) =>
      data.takeup_tension_arm_control_state?.target_angle?.toFixed(1) ?? "N/A",
  },
  {
    name: "Source Tension Arm Target Angle",
    unit: "deg",
    renderValue: (data: RewinderPresetData) =>
      data.source_tension_arm_control_state?.target_angle?.toFixed(1) ?? "N/A",
  },
  previewSeparator,
  // --- Prepare Control ---
  {
    name: "Prepare Tolerance Angle",
    unit: "deg",
    renderValue: (data: RewinderPresetData) =>
      data.prepare_control_state?.tolerance_angle?.toFixed(1) ?? "N/A",
  },
  {
    name: "Prepare Settle Rate",
    renderValue: (data: RewinderPresetData) =>
      data.prepare_control_state?.settle_rate?.toFixed(2) ?? "N/A",
  },
  previewSeparator,
  // --- Rewind Automatic Action ---
  {
    name: "Auto Action Required Meters",
    unit: "m",
    renderValue: (data: RewinderPresetData) =>
      data.rewind_automatic_action_state?.required_meters?.toFixed(1) ?? "N/A",
  },
  {
    name: "Auto Action Mode",
    renderValue: (data: RewinderPresetData) =>
      data.rewind_automatic_action_state?.mode ?? "N/A",
  },
];

export function RewinderPresetsPage() {
  const {
    state,
    defaultState,

    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    enableTraverseLaserpointer,

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
  } = useRewinder();

  const applyPreset = (preset: Preset<RewinderPresetData>) => {
    // Traverse
    setTraverseLimitInner(
      preset.data?.traverse_state?.limit_inner ?? 20,
    );
    setTraverseLimitOuter(
      preset.data?.traverse_state?.limit_outer ?? 100,
    );
    setTraverseStepSize(
      preset.data?.traverse_state?.step_size ?? 5,
    );
    setTraversePadding(
      preset.data?.traverse_state?.padding ?? 10,
    );
    enableTraverseLaserpointer(
      preset.data?.traverse_state?.laserpointer ?? false,
    );

    // Puller
    setPullerTargetSpeed(
      preset.data?.puller_state?.target_speed ?? 10.0,
    );

    // Takeup Spool
    setTakeupSpoolRegulationMode(
      preset.data?.takeup_spool_state?.regulation_mode ?? "Adaptive",
    );
    setTakeupSpoolMinMaxMinSpeed(
      preset.data?.takeup_spool_state?.minmax_min_speed ?? 5.0,
    );
    setTakeupSpoolMinMaxMaxSpeed(
      preset.data?.takeup_spool_state?.minmax_max_speed ?? 50.0,
    );
    setTakeupTensionTarget(
      preset.data?.takeup_spool_state?.adaptive_tension_target ?? 2.5,
    );
    setTakeupSpoolAdaptiveRadiusLearningRate(
      preset.data?.takeup_spool_state?.adaptive_radius_learning_rate ?? 0.1,
    );
    setTakeupSpoolAdaptiveMaxSpeedMultiplier(
      preset.data?.takeup_spool_state?.adaptive_max_speed_multiplier ?? 2.0,
    );
    setTakeupSpoolAdaptiveAccelerationFactor(
      preset.data?.takeup_spool_state?.adaptive_acceleration_factor ?? 1.5,
    );
    setTakeupSpoolAdaptiveDeaccelerationUrgencyMultiplier(
      preset.data?.takeup_spool_state
        ?.adaptive_deacceleration_urgency_multiplier ?? 2.0,
    );

    // Source Spool
    setSourceTensionTarget(
      preset.data?.source_spool_state?.adaptive_tension_target ?? 2.0,
    );

    // Tension Arm Control
    if (preset.data?.takeup_tension_arm_control_state) {
      const tc = preset.data.takeup_tension_arm_control_state;
      if (tc.target_angle !== undefined)
        setTakeupTensionArmControl("target_angle", tc.target_angle);
      if (tc.hard_min_angle !== undefined)
        setTakeupTensionArmControl("hard_min_angle", tc.hard_min_angle);
      if (tc.hard_max_angle !== undefined)
        setTakeupTensionArmControl("hard_max_angle", tc.hard_max_angle);
      if (tc.start_min_angle !== undefined)
        setTakeupTensionArmControl("start_min_angle", tc.start_min_angle);
      if (tc.start_max_angle !== undefined)
        setTakeupTensionArmControl("start_max_angle", tc.start_max_angle);
    }
    if (preset.data?.source_tension_arm_control_state) {
      const sc = preset.data.source_tension_arm_control_state;
      if (sc.target_angle !== undefined)
        setSourceTensionArmControl("target_angle", sc.target_angle);
      if (sc.hard_min_angle !== undefined)
        setSourceTensionArmControl("hard_min_angle", sc.hard_min_angle);
      if (sc.hard_max_angle !== undefined)
        setSourceTensionArmControl("hard_max_angle", sc.hard_max_angle);
      if (sc.start_min_angle !== undefined)
        setSourceTensionArmControl("start_min_angle", sc.start_min_angle);
      if (sc.start_max_angle !== undefined)
        setSourceTensionArmControl("start_max_angle", sc.start_max_angle);
    }

    // Prepare Control
    if (preset.data?.prepare_control_state) {
      const pc = preset.data.prepare_control_state;
      if (pc.tolerance_angle !== undefined)
        setPrepareControl("tolerance_angle", pc.tolerance_angle);
      if (pc.settle_rate !== undefined)
        setPrepareControl("settle_rate", pc.settle_rate);
    }

    // Rewind Automatic Action
    setRewindAutomaticRequiredMeters(
      preset.data?.rewind_automatic_action_state?.required_meters ?? 100.0,
    );
    setRewindAutomaticAction(
      preset.data?.rewind_automatic_action_state?.mode ?? "NoAction",
    );
  };

  const toPresetData = (
    s: typeof state,
  ): RewinderPresetData => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state ?? {},
    takeup_spool_state: s?.takeup_spool_state ?? {},
    source_spool_state: s?.source_spool_state ?? {},
    takeup_tension_arm_control_state:
      s?.takeup_tension_arm_control_state ?? {},
    source_tension_arm_control_state:
      s?.source_tension_arm_control_state ?? {},
    prepare_control_state: s?.prepare_control_state ?? {},
    rewind_automatic_action_state:
      s?.rewind_automatic_action_state ?? {},
  });

  return (
    <PresetsPage
      machine_identification={rewinder.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      defaultState={toPresetData(defaultState)}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
    />
  );
}
