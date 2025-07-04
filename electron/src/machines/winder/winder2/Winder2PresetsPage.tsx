import React from "react";
import { useWinder2 } from "./useWinder";
import { winder2 } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import { StateEvent } from "./winder2Namespace";

type Winder2PresetData = {
  traverse_state: {
    limit_inner: number | undefined;
    limit_outer: number | undefined;
    step_size: number | undefined;
    padding: number | undefined;
    laserpointer: boolean | undefined;
  };
  puller_state: Partial<StateEvent["data"]["puller_state"]>;
  spool_speed_controller_state: Partial<
    StateEvent["data"]["spool_speed_controller_state"]
  >;
};

function renderPreview(preset: Preset<Winder2PresetData>) {
  return (
    <>
      Inner Traverse Limit = {preset.data.traverse_state?.limit_inner ?? "N/A"}{" "}
      mm <br />
      Outer Traverse Limit = {preset.data.traverse_state?.limit_outer ??
        "N/A"}{" "}
      mm <br />
      Traverse Step Size = {preset.data.traverse_state?.step_size ??
        "N/A"} mm <br />
      Traverse Padding = {preset.data.traverse_state?.padding ?? "N/A"} mm{" "}
      <br />
      <br />
      Puller Regulation = {preset.data.puller_state?.regulation ?? "N/A"} <br />
      Puller Direction ={" "}
      {preset.data.puller_state?.forward ? "Forward" : "Backward"} <br />
      Puller Target Speed = {preset.data.puller_state?.target_speed ??
        "N/A"}{" "}
      m/min <br />
      Puller Target Diameter ={" "}
      {preset.data.puller_state?.target_diameter ?? "N/A"} mm <br />
      <br />
      Spool Regulation ={" "}
      {preset.data.spool_speed_controller_state?.regulation_mode ?? "N/A"}{" "}
      <br />
      Spool Min Speed ={" "}
      {preset.data.spool_speed_controller_state?.minmax_min_speed ?? "N/A"}{" "}
      m/min <br />
      Spool max Speed ={" "}
      {preset.data.spool_speed_controller_state?.minmax_max_speed ?? "N/A"}{" "}
      m/min <br />
    </>
  );
}

export function Winder2PresetsPage() {
  const {
    state,

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

  // TODO: Commented out code needs to be implemented in the backend first
  const applyPreset = (preset: Preset<Winder2PresetData>) => {
    setTraverseLimitInner(preset.data?.traverse_state?.limit_inner ?? 22);
    setTraverseLimitOuter(preset.data?.traverse_state?.limit_outer ?? 92);
    setTraverseStepSize(preset.data?.traverse_state?.step_size ?? 1.75);
    setTraversePadding(preset.data?.traverse_state?.padding ?? 0.88);

    setPullerRegulationMode(preset.data?.puller_state?.regulation ?? "Speed");
    setPullerForward(preset.data?.puller_state?.forward ?? true);
    setPullerTargetSpeed(preset.data?.puller_state?.target_speed ?? 1.0);
    // setPullerTargetDiameter(preset.data?.puller_state?.target_diameter ?? 1.75);

    setSpoolRegulationMode(
      preset.data?.spool_speed_controller_state?.regulation_mode ?? "MinMax",
    );
    setSpoolMinMaxMinSpeed(
      preset.data?.spool_speed_controller_state?.minmax_min_speed ?? 0,
    );
    setSpoolMinMaxMaxSpeed(
      preset.data?.spool_speed_controller_state?.minmax_max_speed ?? 150.0,
    );

    // setSpoolAdaptiveTensionTarget(
    //   preset.data?.spool_speed_controller_state?.adaptive_tension_target ?? 0.7,
    // );
    // setSpoolAdaptiveRadiusLearningRate(
    //   preset.data?.spool_speed_controller_state?.adaptive_radius_learning_rate ??
    //     0.5,
    // );
    // setSpoolAdaptiveMaxSpeedMultiplier(
    //   preset.data?.spool_speed_controller_state?.adaptive_max_speed_multiplier ??
    //     4,
    // );
    // setSpoolAdaptiveAccelerationFactor(
    //   preset.data?.spool_speed_controller_state?.adaptive_acceleration_factor ??
    //     0.2,
    // );
    // setSpoolAdaptiveDeaccelerationUrgencyMultiplier(
    //   preset.data?.spool_speed_controller_state
    //     ?.adaptive_deacceleration_urgency_multiplier ?? 15.0,
    // );

    enableTraverseLaserpointer(
      preset.data.traverse_state?.laserpointer ?? false,
    );
  };

  const readCurrentState = (): Winder2PresetData => ({
    traverse_state: {
      limit_inner: state?.traverse_state?.limit_inner,
      limit_outer: state?.traverse_state?.limit_outer,
      step_size: state?.traverse_state?.step_size,
      padding: state?.traverse_state?.padding,
      laserpointer: state?.traverse_state?.laserpointer,
    },
    puller_state: state?.puller_state ?? {},
    spool_speed_controller_state: state?.spool_speed_controller_state ?? {},
  });

  return (
    <PresetsPage
      machine_identification={winder2.machine_identification}
      readCurrentState={readCurrentState}
      schemaVersion={1}
      applyPreset={applyPreset}
      renderPreview={renderPreview}
    />
  );
}
