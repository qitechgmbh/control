import React from "react";
import { useWinder2 } from "./useWinder";
import { winder2 } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import {
  pullerStateSchema,
  spoolSpeedControllerStateSchema,
} from "./winder2Namespace";
import { z } from "zod";
import { PresetPreviewEntry, previewSeparator } from "@/components/preset/PresetPreviewTable";

const winder2PresetDataSchema = z
  .object({
    traverse_state: z.object({
      limit_inner: z.number(),
      limit_outer: z.number(),
      step_size: z.number(),
      padding: z.number(),
      laserpointer: z.boolean(),
    }),
    puller_state: pullerStateSchema,
    spool_speed_controller_state: spoolSpeedControllerStateSchema,
  })
  .deepPartial();

type Winder2 = typeof winder2PresetDataSchema;

type Winder2PresetData = z.infer<Winder2>;

const previewEntries: PresetPreviewEntry<Winder2>[] = [
  {
    name: "Inner Traverse Limit",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data?.traverse_state?.limit_inner?.toFixed(1) ?? "N/A",
  },
  {
    name: "Outer Traverse Limit",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data?.traverse_state?.limit_outer?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Step Size",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data?.traverse_state?.step_size?.toFixed(1) ?? "N/A",
  },
  {
    name: "Traverse Padding",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data?.traverse_state?.padding?.toFixed(1),
  },
  previewSeparator,
  {
    name: "Puller Regulation",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data?.puller_state?.regulation,
  },
  {
    name: "Puller Direction",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.puller_state?.forward ? "Forward" : "Backward",
  },
  {
    name: "Puller Target Speed",
    unit: "m/min",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.puller_state?.target_speed?.toFixed(2),
  },
  {
    name: "Puller Target Diameter",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.puller_state?.target_diameter?.toFixed(1),
  },
  {
    name: "Puller Target Diameter",
    unit: "mm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.puller_state?.target_diameter?.toFixed(1),
  },
  previewSeparator,
  {
    name: "Spool Regulation",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.spool_speed_controller_state?.regulation_mode,
  },
  {
    name: "Spool Min Speed",
    unit: "rpm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.spool_speed_controller_state?.minmax_min_speed?.toFixed(2),
  },
  {
    name: "Spool Max Speed",
    unit: "rpm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.spool_speed_controller_state?.minmax_max_speed?.toFixed(2),
  },
  previewSeparator,
  {
    name: "Spool Max Speed",
    unit: "rpm",
    renderValue: (preset: Preset<Winder2>) =>
      preset.data.spool_speed_controller_state?.minmax_max_speed?.toFixed(2),
  },
];

export function Winder2PresetsPage() {
  const {
    state,
    defaultState,

    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,

    setPullerRegulationMode,
    setPullerTargetDiameter,
    setPullerForward,
    setPullerTargetSpeed,

    setSpoolRegulationMode,

    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,

    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,

    enableTraverseLaserpointer,
  } = useWinder2();

  const applyPreset = (preset: Preset<Winder2>) => {
    setTraverseLimitInner(preset.data?.traverse_state?.limit_inner ?? 22);
    setTraverseLimitOuter(preset.data?.traverse_state?.limit_outer ?? 92);
    setTraverseStepSize(preset.data?.traverse_state?.step_size ?? 1.75);
    setTraversePadding(preset.data?.traverse_state?.padding ?? 0.88);

    setPullerRegulationMode(preset.data?.puller_state?.regulation ?? "Speed");
    setPullerForward(preset.data?.puller_state?.forward ?? true);
    setPullerTargetSpeed(preset.data?.puller_state?.target_speed ?? 1.0);
    setPullerTargetDiameter(preset.data?.puller_state?.target_diameter ?? 1.75);

    setSpoolRegulationMode(
      preset.data?.spool_speed_controller_state?.regulation_mode ?? "MinMax",
    );
    setSpoolMinMaxMinSpeed(
      preset.data?.spool_speed_controller_state?.minmax_min_speed ?? 0,
    );
    setSpoolMinMaxMaxSpeed(
      preset.data?.spool_speed_controller_state?.minmax_max_speed ?? 150.0,
    );

    setSpoolAdaptiveTensionTarget(
      preset.data?.spool_speed_controller_state?.adaptive_tension_target ?? 0.7,
    );
    setSpoolAdaptiveRadiusLearningRate(
      preset.data?.spool_speed_controller_state?.adaptive_radius_learning_rate ??
        0.5,
    );
    setSpoolAdaptiveMaxSpeedMultiplier(
      preset.data?.spool_speed_controller_state?.adaptive_max_speed_multiplier ??
        4,
    );
    setSpoolAdaptiveAccelerationFactor(
      preset.data?.spool_speed_controller_state?.adaptive_acceleration_factor ??
        0.2,
    );
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
      preset.data?.spool_speed_controller_state
        ?.adaptive_deacceleration_urgency_multiplier ?? 15.0,
    );

    enableTraverseLaserpointer(
      preset.data.traverse_state?.laserpointer ?? false,
    );
  };

  const toPresetData = (s: typeof state): Winder2PresetData => ({
    traverse_state: {
      limit_inner: s?.traverse_state?.limit_inner,
      limit_outer: s?.traverse_state?.limit_outer,
      step_size: s?.traverse_state?.step_size,
      padding: s?.traverse_state?.padding,
      laserpointer: s?.traverse_state?.laserpointer,
    },
    puller_state: s?.puller_state ?? {},
    spool_speed_controller_state: s?.spool_speed_controller_state ?? {},
  });

  return (
    <PresetsPage
      machine_identification={winder2.machine_identification}
      currentState={toPresetData(state)}
      schemaVersion={1}
      defaultState={toPresetData(defaultState)}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
    />
  );
}
