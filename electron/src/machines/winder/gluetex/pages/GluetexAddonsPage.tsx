import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TemperatureBar } from "../../TemperatureBar";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { useGluetex } from "../hooks/useGluetex";
import {
  StepperMode,
  HeatingMode,
  getGearRatioMultiplier,
} from "../state/gluetexNamespace";
import { roundToDecimals, roundDegreesToDecimals } from "@/lib/decimal";
import { TensionArm } from "../../TensionArm";
import { RatioInput } from "@/components/ratio/RatioInput";
import { GluetexErrorBanner } from "../components/GluetexErrorBanner";

export function GluetexAddonsPage() {
  const {
    state,
    defaultState,
    isLoading,
    isDisabled,
    setStepper3Mode,
    setStepper4Mode,
    setStepper5Mode,
    setHeatingMode,
    optris1Voltage,
    optris2Voltage,
    slavePullerSpeed,
    slaveTensionArmAngle,
    addonTensionArmAngle,
    zeroSlaveTensionArm,
    zeroAddonTensionArm,
    setStepper3Master,
    setStepper3Slave,
    setStepper4Master,
    setStepper4Slave,
    setStepper5Master,
    setStepper5Slave,
    setStepper3Konturlaenge,
    setStepper3Pause,
  } = useGluetex();

  // Calculate max speed based on gear ratio (same as main puller)
  const gearRatioMultiplier = getGearRatioMultiplier(
    state?.puller_state?.gear_ratio,
  );

  return (
    <Page>
      <GluetexErrorBanner />
      <ControlGrid>
        <ControlCard className="bg-red" title="Motors">
          <Label label="Stepper 3">
            <SelectionGroup<StepperMode>
              value={state?.stepper_state?.stepper3_mode}
              disabled={isDisabled}
              loading={isLoading}
              onChange={setStepper3Mode}
              orientation="horizontal"
              className="grid grid-cols-2 gap-2"
              options={{
                Standby: {
                  children: "Standby",
                  icon: "lu:Power",
                  isActiveClassName: "bg-green-600",
                },
                Run: {
                  children: "Run",
                  icon: "lu:Play",
                  isActiveClassName: "bg-green-600",
                },
              }}
            />
          </Label>
          <Label label="Stepper 4">
            <SelectionGroup<StepperMode>
              value={state?.stepper_state?.stepper4_mode}
              disabled={isDisabled}
              loading={isLoading}
              onChange={setStepper4Mode}
              orientation="horizontal"
              className="grid grid-cols-2 gap-2"
              options={{
                Standby: {
                  children: "Standby",
                  icon: "lu:Power",
                  isActiveClassName: "bg-green-600",
                },
                Run: {
                  children: "Run",
                  icon: "lu:Play",
                  isActiveClassName: "bg-green-600",
                },
              }}
            />
          </Label>
          <Label label="Stepper 5">
            <SelectionGroup<StepperMode>
              value={state?.stepper_state?.stepper5_mode}
              disabled={isDisabled}
              loading={isLoading}
              onChange={setStepper5Mode}
              orientation="horizontal"
              className="grid grid-cols-2 gap-2"
              options={{
                Standby: {
                  children: "Standby",
                  icon: "lu:Power",
                  isActiveClassName: "bg-green-600",
                },
                Run: {
                  children: "Run",
                  icon: "lu:Play",
                  isActiveClassName: "bg-green-600",
                },
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Slave Tension Arm">
          <TensionArm degrees={slaveTensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Slave Tension Arm"
            unit="deg"
            timeseries={slaveTensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TimeSeriesValueNumeric
            label="Slave Puller Speed"
            unit="m/min"
            timeseries={slavePullerSpeed}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroSlaveTensionArm}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.slave_puller_state?.tension_arm?.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard className="bg-red" height={2} title="Quality Control">
          <div className="space-y-6">
            {/* Optris 1 Voltage */}
            <div className="space-y-3">
              <TimeSeriesValueNumeric
                label="Optris 1 Voltage"
                unit="V"
                timeseries={optris1Voltage}
                renderValue={(value) => roundToDecimals(value, 2)}
              />
              {state?.optris_1_monitor_state?.triggered && (
                <StatusBadge variant="error">
                  Voltage Out of Range - Machine Stopped
                </StatusBadge>
              )}
              <TemperatureBar
                min={0}
                max={10}
                minLimit={
                  state?.optris_1_monitor_state?.min_voltage ??
                  state?.quality_control_state?.optris1.min_voltage ??
                  0
                }
                maxLimit={
                  state?.optris_1_monitor_state?.max_voltage ??
                  state?.quality_control_state?.optris1.max_voltage ??
                  10
                }
                current={optris1Voltage.current?.value ?? 0}
              />
            </div>

            {/* Optris 2 Voltage */}
            <div className="space-y-3">
              <TimeSeriesValueNumeric
                label="Optris 2 Voltage"
                unit="V"
                timeseries={optris2Voltage}
                renderValue={(value) => roundToDecimals(value, 2)}
              />
              {state?.optris_2_monitor_state?.triggered && (
                <StatusBadge variant="error">
                  Voltage Out of Range - Machine Stopped
                </StatusBadge>
              )}
              <TemperatureBar
                min={0}
                max={10}
                minLimit={
                  state?.optris_2_monitor_state?.min_voltage ??
                  state?.quality_control_state?.optris2.min_voltage ??
                  0
                }
                maxLimit={
                  state?.optris_2_monitor_state?.max_voltage ??
                  state?.quality_control_state?.optris2.max_voltage ??
                  10
                }
                current={optris2Voltage.current?.value ?? 0}
              />
            </div>

            {/* Heating Mode */}
            <div className="mt-8 space-y-3">
              <h3 className="text-lg font-semibold">Heating Mode</h3>
              <SelectionGroup<HeatingMode>
                value={state?.heating_state?.heating_mode}
                disabled={isDisabled}
                loading={isLoading}
                onChange={setHeatingMode}
                orientation="horizontal"
                className="grid grid-cols-2 gap-2"
                options={{
                  Standby: {
                    children: "Standby",
                    icon: "lu:Power",
                    isActiveClassName: "bg-green-600",
                  },
                  Heating: {
                    children: "Heating",
                    icon: "lu:Flame",
                    isActiveClassName: "bg-green-600",
                  },
                }}
              />
            </div>
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Motor Ratios">
          <Label label="Stepper 3 Ratio">
            <RatioInput
              master={state?.motor_ratios_state?.stepper3_master}
              slave={state?.motor_ratios_state?.stepper3_slave}
              title="Stepper 3 Motor Ratio"
              onRatioChange={(master, slave) => {
                setStepper3Master(master);
                setStepper3Slave(slave);
              }}
            />
          </Label>
          <Label label="Stepper 4 Ratio">
            <RatioInput
              master={state?.motor_ratios_state?.stepper4_master}
              slave={state?.motor_ratios_state?.stepper4_slave}
              title="Stepper 4 Motor Ratio"
              onRatioChange={(master, slave) => {
                setStepper4Master(master);
                setStepper4Slave(slave);
              }}
            />
          </Label>
          <Label label="Stepper 5 Ratio">
            <RatioInput
              master={state?.motor_ratios_state?.stepper5_master}
              slave={state?.motor_ratios_state?.stepper5_slave}
              title="Stepper 5 Motor Ratio"
              onRatioChange={(master, slave) => {
                setStepper5Master(master);
                setStepper5Slave(slave);
              }}
            />
          </Label>
          <div className="flex flex-col gap-4">
            <Label label="Stepper 3: Contour Length">
              <EditValue
                value={state?.addon_motor_3_state?.konturlaenge_mm}
                title="Contour Length"
                unit="mm"
                step={1}
                min={0}
                max={10000}
                defaultValue={0}
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={(value) => setStepper3Konturlaenge(value)}
              />
            </Label>
            <Label label="Pause">
              <EditValue
                value={state?.addon_motor_3_state?.pause_mm}
                title="Pause"
                unit="mm"
                step={1}
                min={0}
                max={10000}
                defaultValue={0}
                renderValue={(value) => roundToDecimals(value, 1)}
                onChange={(value) => setStepper3Pause(value)}
              />
            </Label>
          </div>
          {state?.addon_motor_3_state?.konturlaenge_mm !== undefined &&
            state?.addon_motor_3_state?.konturlaenge_mm > 0 && (
              <Label label="Stepper 3 Pattern State">
                <StatusBadge
                  variant={
                    state?.addon_motor_3_state?.pattern_state === "Running"
                      ? "success"
                      : "error"
                  }
                >
                  {state?.addon_motor_3_state?.pattern_state || "Idle"}
                </StatusBadge>
              </Label>
            )}
        </ControlCard>

        <ControlCard title="Addon Tension Arm">
          <TensionArm degrees={addonTensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Angle"
            unit="deg"
            timeseries={addonTensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={zeroAddonTensionArm}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.addon_tension_arm_state?.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
