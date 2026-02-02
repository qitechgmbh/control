import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import React, { useState } from "react";
import { useGluetex } from "../hooks/useGluetex";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { SelectionGroup } from "@/control/SelectionGroup";
import { MachineSelector } from "@/components/MachineConnectionDropdown";
import {
  getGluetexXLMode,
  setGluetexXLMode,
  GLUETEX_TRAVERSE_MAX_STANDARD,
  GLUETEX_TRAVERSE_MAX_XL,
} from "../config/gluetexConfig";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Icon } from "@/components/Icon";

export function GluetexSettingPage() {
  const [xlMode, setXlMode] = useState(getGluetexXLMode());
  const [autoTuneTargetTemps, setAutoTuneTargetTemps] = useState({
    zone_1: 75,
    zone_2: 75,
    zone_3: 75,
    zone_4: 75,
    zone_5: 75,
    zone_6: 75,
  });
  
  // Collapsible section states - all collapsed by default
  const [winderOpen, setWinderOpen] = useState(false);
  const [heatingOpen, setHeatingOpen] = useState(false);
  const [addonsOpen, setAddonsOpen] = useState(false);

  const {
    state,
    defaultState,
    setTraverseStepSize,
    setTraversePadding,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseHome,
    setPullerForward,
    setPullerGearRatio,
    setSpoolRegulationMode,
    setSpoolMinMaxMinSpeed,
    setSpoolMinMaxMaxSpeed,
    setSpoolForward,
    setSpoolAdaptiveTensionTarget,
    setSpoolAdaptiveRadiusLearningRate,
    setSpoolAdaptiveMaxSpeedMultiplier,
    setSpoolAdaptiveAccelerationFactor,
    setSpoolAdaptiveDeaccelerationUrgencyMultiplier,
    isLoading,
    isDisabled,
    selectedMachine,
    filteredMachines,
    setConnectedMachine,
    disconnectMachine,
    setSlavePullerEnabled,
    setSlavePullerForward,
    setSlavePullerMinAngle,
    setSlavePullerMaxAngle,
    setSlavePullerMinSpeedFactor,
    setSlavePullerMaxSpeedFactor,
    zeroSlaveTensionArm,
    setHeatingPid,
    startHeatingAutoTune,
    stopHeatingAutoTune,
    setTensionArmMonitorEnabled,
    setTensionArmMonitorMinAngle,
    setTensionArmMonitorMaxAngle,
    setSleepTimerEnabled,
    setSleepTimerTimeout,
    setStepper3Forward,
    setStepper4Forward,
    setStepper5Forward,
  } = useGluetex();

  const handleXlModeChange = (enabled: boolean) => {
    setGluetexXLMode(enabled);
    setXlMode(enabled);

    // When switching from XL to normal mode, reset traverse limits to default values
    if (!enabled && defaultState) {
      // Only reset if current values exceed the standard max
      const currentOuter = state?.traverse_state?.limit_outer ?? 0;
      const currentInner = state?.traverse_state?.limit_inner ?? 0;
      const defaultOuter = defaultState.traverse_state?.limit_outer;
      const defaultInner = defaultState.traverse_state?.limit_inner;

      if (
        currentOuter > GLUETEX_TRAVERSE_MAX_STANDARD &&
        defaultOuter !== undefined
      ) {
        setTraverseLimitOuter(defaultOuter);
        setTraverseLimitInner(defaultInner);
      }
      if (
        currentInner > GLUETEX_TRAVERSE_MAX_STANDARD &&
        defaultInner !== undefined
      ) {
        setTraverseLimitOuter(defaultOuter);
        setTraverseLimitInner(defaultInner);
      }

      // Home the traverse when switching from XL to normal mode
      gotoTraverseHome();
    }
  };

  return (
    <Page>
      <div className="flex flex-col gap-6">
        {/* ========== WINDER SETTINGS ========== */}
        <Collapsible open={winderOpen} onOpenChange={setWinderOpen}>
          <CollapsibleTrigger asChild>
            <div className="cursor-pointer rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-750 transition-colors">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold flex items-center gap-2">
                  <Icon name="lu:Settings2" className="h-5 w-5" />
                  Winder Settings
                </h2>
                <Icon
                  name={winderOpen ? "lu:ChevronUp" : "lu:ChevronDown"}
                  className="h-5 w-5 transition-transform"
                />
              </div>
            </div>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-4">
              <ControlGrid>
        <ControlCard title="Traverse">
          <Label label="Traverse Size">
            <SelectionGroupBoolean
              value={xlMode}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: `Standard (${GLUETEX_TRAVERSE_MAX_STANDARD}mm)`,
                icon: "lu:Settings",
              }}
              optionTrue={{
                children: `XL (${GLUETEX_TRAVERSE_MAX_XL}mm)`,
                icon: "lu:Maximize2",
              }}
              onChange={handleXlModeChange}
            />
          </Label>
          <Label label="Step Size">
            <EditValue
              value={state?.traverse_state?.step_size}
              title={"Step Size"}
              unit="mm"
              step={0.05}
              min={0.1}
              max={75}
              defaultValue={defaultState?.traverse_state?.step_size}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => setTraverseStepSize(value)}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={state?.traverse_state?.padding}
              title={"Padding"}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              defaultValue={defaultState?.traverse_state?.padding}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => setTraversePadding(value)}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Spool">
          <Label label="Speed Algorithm">
            <SelectionGroup
              value={state?.spool_speed_controller_state?.regulation_mode}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                MinMax: {
                  children: "Min/Max",
                  icon: "lu:ArrowUpDown",
                },
                Adaptive: {
                  children: "Adaptive",
                  icon: "lu:Brain",
                },
              }}
              onChange={(value) =>
                setSpoolRegulationMode(value as "Adaptive" | "MinMax")
              }
            />
          </Label>

          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.spool_speed_controller_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setSpoolForward(value)}
            />
          </Label>

          {state?.spool_speed_controller_state?.regulation_mode ===
            "MinMax" && (
            <>
              <Label label="Minimum Speed">
                <EditValue
                  value={state?.spool_speed_controller_state?.minmax_min_speed}
                  title={"Minimum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_speed_controller_state?.minmax_min_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMinSpeed(value)}
                />
              </Label>
              <Label label="Maximum Speed">
                <EditValue
                  value={state?.spool_speed_controller_state?.minmax_max_speed}
                  title={"Maximum Speed"}
                  unit="rpm"
                  step={10}
                  min={0}
                  max={600}
                  defaultValue={
                    defaultState?.spool_speed_controller_state?.minmax_max_speed
                  }
                  renderValue={(value) => roundToDecimals(value, 0)}
                  onChange={(value) => setSpoolMinMaxMaxSpeed(value)}
                />
              </Label>
            </>
          )}

          {state?.spool_speed_controller_state?.regulation_mode ===
            "Adaptive" && (
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Tension Target">
                <EditValue
                  value={
                    state?.spool_speed_controller_state?.adaptive_tension_target
                  }
                  title={"Tension Target"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={1}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_tension_target
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) => setSpoolAdaptiveTensionTarget(value)}
                />
              </Label>
              <Label label="Learning Rate">
                <EditValue
                  value={
                    state?.spool_speed_controller_state
                      ?.adaptive_radius_learning_rate
                  }
                  title={"Radius Learning Rate"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_radius_learning_rate
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) =>
                    setSpoolAdaptiveRadiusLearningRate(value)
                  }
                />
              </Label>
              <Label label="Max Speed Multiplier">
                <EditValue
                  value={
                    state?.spool_speed_controller_state
                      ?.adaptive_max_speed_multiplier
                  }
                  title={"Max Speed Multiplier"}
                  unit={undefined}
                  step={0.1}
                  min={0.1}
                  max={10}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_max_speed_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setSpoolAdaptiveMaxSpeedMultiplier(value)
                  }
                />
              </Label>
              <Label label="Acceleration Factor">
                <EditValue
                  value={
                    state?.spool_speed_controller_state
                      ?.adaptive_acceleration_factor
                  }
                  title={"Acceleration Factor"}
                  unit={undefined}
                  step={0.01}
                  min={0.01}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_acceleration_factor
                  }
                  renderValue={(value) => roundToDecimals(value, 2)}
                  onChange={(value) =>
                    setSpoolAdaptiveAccelerationFactor(value)
                  }
                />
              </Label>
              <Label label="Deaccel. Urgency">
                <EditValue
                  value={
                    state?.spool_speed_controller_state
                      ?.adaptive_deacceleration_urgency_multiplier
                  }
                  title={"Deacceleration Urgency Multiplier"}
                  unit={undefined}
                  step={0.5}
                  min={1}
                  max={100}
                  defaultValue={
                    defaultState?.spool_speed_controller_state
                      ?.adaptive_deacceleration_urgency_multiplier
                  }
                  renderValue={(value) => roundToDecimals(value, 1)}
                  onChange={(value) =>
                    setSpoolAdaptiveDeaccelerationUrgencyMultiplier(value)
                  }
                />
              </Label>
            </div>
          )}
        </ControlCard>

        <ControlCard title="Puller">
          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.puller_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setPullerForward(value)}
            />
          </Label>
          <Label label="Gear Ratio">
            <SelectionGroup
              value={state?.puller_state?.gear_ratio}
              disabled={isDisabled}
              loading={isLoading}
              options={{
                OneToOne: {
                  children: "1:1",
                  icon: "lu:Circle",
                },
                OneToFive: {
                  children: "1:5",
                  icon: "lu:Cog",
                },
                OneToTen: {
                  children: "1:10",
                  icon: "lu:Cog",
                },
              }}
              onChange={(value) =>
                setPullerGearRatio(
                  value as "OneToOne" | "OneToFive" | "OneToTen",
                )
              }
            />
          </Label>
        </ControlCard>
              </ControlGrid>
            </div>
          </CollapsibleContent>
        </Collapsible>

        {/* ========== ADDON SETTINGS ========== */}
        <Collapsible open={addonsOpen} onOpenChange={setAddonsOpen}>
          <CollapsibleTrigger asChild>
            <div className="cursor-pointer rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-750 transition-colors">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold flex items-center gap-2">
                  <Icon name="lu:Puzzle" className="h-5 w-5" />
                  Addon Settings
                </h2>
                <Icon
                  name={addonsOpen ? "lu:ChevronUp" : "lu:ChevronDown"}
                  className="h-5 w-5 transition-transform"
                />
              </div>
            </div>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-4">
              <ControlGrid>
        <ControlCard title="Slave Puller">
          <Label label="Enable Slave Puller">
            <SelectionGroupBoolean
              value={state?.slave_puller_state?.enabled}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Disabled",
                icon: "lu:Circle",
              }}
              optionTrue={{
                children: "Enabled",
                icon: "lu:Circle",
              }}
              onChange={(value) => setSlavePullerEnabled(value)}
            />
          </Label>

          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={state?.slave_puller_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setSlavePullerForward(value)}
            />
          </Label>

          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Min Angle (Detection Zone)">
              <EditValue
                value={state?.slave_puller_state?.min_angle}
                title={"Minimum Angle"}
                unit="deg"
                step={1}
                min={0}
                max={
                  state?.slave_puller_state?.max_angle
                    ? state.slave_puller_state.max_angle - 5
                    : 85
                }
                defaultValue={20}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) => setSlavePullerMinAngle(value)}
              />
            </Label>

            <Label label="Max Angle (Detection Zone)">
              <EditValue
                value={state?.slave_puller_state?.max_angle}
                title={"Maximum Angle"}
                unit="deg"
                step={1}
                min={
                  state?.slave_puller_state?.min_angle
                    ? state.slave_puller_state.min_angle + 5
                    : 25
                }
                max={180}
                defaultValue={90}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={(value) => setSlavePullerMaxAngle(value)}
              />
            </Label>

            <Label label="Min Speed Factor (Optional)">
              <EditValue
                value={state?.slave_puller_state?.min_speed_factor ?? 0}
                title={"Minimum Speed Factor"}
                unit={undefined}
                step={0.05}
                min={0.1}
                max={
                  state?.slave_puller_state?.max_speed_factor
                    ? state.slave_puller_state.max_speed_factor - 0.1
                    : 2
                }
                defaultValue={0}
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={(value) =>
                  setSlavePullerMinSpeedFactor(value === 0 ? null : value)
                }
              />
            </Label>

            <Label label="Max Speed Factor (Optional)">
              <EditValue
                value={state?.slave_puller_state?.max_speed_factor ?? 0}
                title={"Maximum Speed Factor"}
                unit={undefined}
                step={0.05}
                min={
                  state?.slave_puller_state?.min_speed_factor
                    ? state.slave_puller_state.min_speed_factor + 0.1
                    : 0.2
                }
                max={3}
                defaultValue={0}
                renderValue={(value) => roundToDecimals(value, 2)}
                onChange={(value) =>
                  setSlavePullerMaxSpeedFactor(value === 0 ? null : value)
                }
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard title="Motors">
          <Label label="Motor 3 Direction">
            <SelectionGroupBoolean
              value={state?.addon_motor_3_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setStepper3Forward(value)}
            />
          </Label>

          <Label label="Motor 4 Direction">
            <SelectionGroupBoolean
              value={state?.addon_motor_4_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setStepper4Forward(value)}
            />
          </Label>

          <Label label="Motor 5 Direction">
            <SelectionGroupBoolean
              value={state?.addon_motor_5_state?.forward}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => setStepper5Forward(value)}
            />
          </Label>
        </ControlCard>
              </ControlGrid>
            </div>
          </CollapsibleContent>
        </Collapsible>

        {/* ========== HEATING SETTINGS ========== */}
        <Collapsible open={heatingOpen} onOpenChange={setHeatingOpen}>
          <CollapsibleTrigger asChild>
            <div className="cursor-pointer rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-750 transition-colors">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold flex items-center gap-2">
                  <Icon name="lu:Flame" className="h-5 w-5" />
                  Heating Settings
                </h2>
                <Icon
                  name={heatingOpen ? "lu:ChevronUp" : "lu:ChevronDown"}
                  className="h-5 w-5 transition-transform"
                />
              </div>
            </div>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-4">
              <ControlGrid>
        <ControlCard title="Zone 1">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_1?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_1?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(1)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_1?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_1}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_1: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(1, autoTuneTargetTemps.zone_1)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_1?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_1?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_1",
                      value,
                      state?.heating_pid_settings?.zone_1?.ki ?? 0,
                      state?.heating_pid_settings?.zone_1?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_1?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_1?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_1",
                      state?.heating_pid_settings?.zone_1?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_1?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_1?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_1?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_1",
                      state?.heating_pid_settings?.zone_1?.kp ?? 0,
                      state?.heating_pid_settings?.zone_1?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Zone 2">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_2?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_2?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(2)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_2?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_2}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_2: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(2, autoTuneTargetTemps.zone_2)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_2?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_2?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_2",
                      value,
                      state?.heating_pid_settings?.zone_2?.ki ?? 0,
                      state?.heating_pid_settings?.zone_2?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_2?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_2?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_2",
                      state?.heating_pid_settings?.zone_2?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_2?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_2?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_2?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_2",
                      state?.heating_pid_settings?.zone_2?.kp ?? 0,
                      state?.heating_pid_settings?.zone_2?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Zone 3">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_3?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_3?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(3)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_3?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_3}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_3: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(3, autoTuneTargetTemps.zone_3)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_3?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_3?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_3",
                      value,
                      state?.heating_pid_settings?.zone_3?.ki ?? 0,
                      state?.heating_pid_settings?.zone_3?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_3?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_3?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_3",
                      state?.heating_pid_settings?.zone_3?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_3?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_3?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_3?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_3",
                      state?.heating_pid_settings?.zone_3?.kp ?? 0,
                      state?.heating_pid_settings?.zone_3?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Zone 4">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_4?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_4?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(4)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_4?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_4}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_4: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(4, autoTuneTargetTemps.zone_4)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_4?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_4?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_4",
                      value,
                      state?.heating_pid_settings?.zone_4?.ki ?? 0,
                      state?.heating_pid_settings?.zone_4?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_4?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_4?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_4",
                      state?.heating_pid_settings?.zone_4?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_4?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_4?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_4?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_4",
                      state?.heating_pid_settings?.zone_4?.kp ?? 0,
                      state?.heating_pid_settings?.zone_4?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Zone 5">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_5?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_5?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(5)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_5?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_5}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_5: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(5, autoTuneTargetTemps.zone_5)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_5?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_5?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_5",
                      value,
                      state?.heating_pid_settings?.zone_5?.ki ?? 0,
                      state?.heating_pid_settings?.zone_5?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_5?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_5?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_5",
                      state?.heating_pid_settings?.zone_5?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_5?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_5?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_5?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_5",
                      state?.heating_pid_settings?.zone_5?.kp ?? 0,
                      state?.heating_pid_settings?.zone_5?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Zone 6">
          <div className="flex flex-col gap-4">
            {state?.heating_states?.zone_6?.autotuning_active && (
              <div className="rounded bg-blue-100 p-4 dark:bg-blue-900">
                <p className="text-sm font-semibold">
                  Auto-tuning in progress...
                </p>
                <p className="text-sm">
                  Progress:{" "}
                  {state?.heating_states?.zone_6?.autotuning_progress.toFixed(
                    0,
                  )}
                  %
                </p>
                <button
                  onClick={() => stopHeatingAutoTune(6)}
                  className="mt-2 rounded bg-red-500 px-4 py-2 text-white hover:bg-red-600"
                >
                  Stop Auto-Tuning
                </button>
              </div>
            )}
            {!state?.heating_states?.zone_6?.autotuning_active && (
              <div className="flex flex-row items-center gap-2">
                <Label label="Target Temperature (°C)">
                  <EditValue
                    value={autoTuneTargetTemps.zone_6}
                    title="Target"
                    unit="C"
                    step={5}
                    min={50}
                    max={250}
                    defaultValue={75}
                    renderValue={(value) => value.toFixed(0)}
                    onChange={(value) =>
                      setAutoTuneTargetTemps({
                        ...autoTuneTargetTemps,
                        zone_6: value,
                      })
                    }
                  />
                </Label>
                <button
                  onClick={() =>
                    startHeatingAutoTune(6, autoTuneTargetTemps.zone_6)
                  }
                  className="rounded bg-green-500 px-4 py-2 whitespace-nowrap text-white hover:bg-green-600"
                >
                  Start PID Tuning
                </button>
              </div>
            )}
            <div className="flex flex-row flex-wrap gap-4">
              <Label label="Proportional Gain (Kp)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_6?.kp}
                  title={"Kp"}
                  unit={undefined}
                  step={0.01}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_6?.kp}
                  renderValue={(value) => roundToDecimals(value, 3)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_6",
                      value,
                      state?.heating_pid_settings?.zone_6?.ki ?? 0,
                      state?.heating_pid_settings?.zone_6?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Integral Gain (Ki)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_6?.ki}
                  title={"Ki"}
                  unit={undefined}
                  step={0.0001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_6?.ki}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_6",
                      state?.heating_pid_settings?.zone_6?.kp ?? 0,
                      value,
                      state?.heating_pid_settings?.zone_6?.kd ?? 0,
                    )
                  }
                />
              </Label>

              <Label label="Derivative Gain (Kd)">
                <EditValue
                  value={state?.heating_pid_settings?.zone_6?.kd}
                  title={"Kd"}
                  unit={undefined}
                  step={0.001}
                  min={0}
                  max={15}
                  defaultValue={defaultState?.heating_pid_settings?.zone_6?.kd}
                  renderValue={(value) => roundToDecimals(value, 4)}
                  onChange={(value) =>
                    setHeatingPid(
                      "zone_6",
                      state?.heating_pid_settings?.zone_6?.kp ?? 0,
                      state?.heating_pid_settings?.zone_6?.ki ?? 0,
                      value,
                    )
                  }
                />
              </Label>
            </div>
          </div>
        </ControlCard>
              </ControlGrid>
            </div>
          </CollapsibleContent>
        </Collapsible>

        {/* ========== OTHER SETTINGS ========== */}
        <ControlGrid>
        <ControlCard title="Tension Arm Monitor">
          <Label label="Enable Monitoring">
            <SelectionGroupBoolean
              value={state?.tension_arm_monitor_state?.enabled}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Disabled",
                icon: "lu:CircleOff",
              }}
              optionTrue={{
                children: "Enabled",
                icon: "lu:Shield",
              }}
              onChange={(value) => setTensionArmMonitorEnabled(value)}
            />
          </Label>
          {state?.tension_arm_monitor_state?.triggered && (
            <div className="rounded-md bg-red-500/10 p-3 text-red-600 dark:text-red-400">
              <div className="flex items-center gap-2">
                <span className="text-lg">⚠️</span>
                <span className="font-semibold">
                  Tension Arm Limit Exceeded - Machine Stopped
                </span>
              </div>
            </div>
          )}
          <Label label="Minimum Angle">
            <EditValue
              value={state?.tension_arm_monitor_state?.min_angle}
              title={"Minimum Angle"}
              unit="deg"
              step={1}
              min={0}
              max={180}
              defaultValue={defaultState?.tension_arm_monitor_state?.min_angle}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={(value) => setTensionArmMonitorMinAngle(value)}
            />
          </Label>
          <Label label="Maximum Angle">
            <EditValue
              value={state?.tension_arm_monitor_state?.max_angle}
              title={"Maximum Angle"}
              unit="deg"
              step={1}
              min={0}
              max={180}
              defaultValue={defaultState?.tension_arm_monitor_state?.max_angle}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={(value) => setTensionArmMonitorMaxAngle(value)}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Sleep Timer">
          <Label label="Enable Sleep Timer">
            <SelectionGroupBoolean
              value={state?.sleep_timer_state?.enabled}
              disabled={isDisabled}
              loading={isLoading}
              optionFalse={{
                children: "Disabled",
                icon: "lu:CircleOff",
              }}
              optionTrue={{
                children: "Enabled",
                icon: "lu:Timer",
              }}
              onChange={(value) => setSleepTimerEnabled(value)}
            />
          </Label>
          <Label label="Sleep Timeout">
            <EditValue
              value={
                state?.sleep_timer_state?.timeout_seconds
                  ? state.sleep_timer_state.timeout_seconds / 60
                  : 15
              }
              title={"Sleep Timeout"}
              unit="min"
              step={1}
              min={1}
              max={120}
              defaultValue={15}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={(value) => setSleepTimerTimeout(value * 60)}
            />
          </Label>
          <div className="text-sm text-muted-foreground">
            Machine will automatically enter standby mode after{" "}
            {state?.sleep_timer_state?.timeout_seconds
              ? Math.round(state.sleep_timer_state.timeout_seconds / 60)
              : 15}{" "}
            minutes of inactivity.
          </div>
        </ControlCard>
        </ControlGrid>
      </div>
    </Page>
  );
}
