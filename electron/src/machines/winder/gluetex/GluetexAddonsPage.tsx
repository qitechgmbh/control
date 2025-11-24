import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TemperatureBar } from "../TemperatureBar";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { useGluetex } from "./useGluetex";
import {
  StepperMode,
  HeatingMode,
  getGearRatioMultiplier,
} from "./gluetexNamespace";
import { roundToDecimals, roundDegreesToDecimals } from "@/lib/decimal";
import { TensionArm } from "../TensionArm";
import { RatioInput } from "@/components/ratio/RatioInput";

export function GluetexAddonsPage() {
  const {
    state,
    defaultState,
    isLoading,
    isDisabled,
    setStepper3Mode,
    setStepper4Mode,
    setHeatingMode,
    temperature1,
    temperature2,
    setTemperature1Min,
    setTemperature1Max,
    setTemperature2Min,
    setTemperature2Max,
    slavePullerSpeed,
    slaveTensionArmAngle,
    zeroSlaveTensionArm,
    setStepper3Master,
    setStepper3Slave,
    setStepper4Master,
    setStepper4Slave,
  } = useGluetex();

  // Calculate max speed based on gear ratio (same as main puller)
  const gearRatioMultiplier = getGearRatioMultiplier(
    state?.puller_state?.gear_ratio,
  );

  return (
    <Page>
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
        </ControlCard>

        <ControlCard className="bg-red" title="Heating Mode">
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
        </ControlCard>

        <ControlCard className="bg-red" height={2} title="Quality Control">
          <div className="space-y-6">
            {/* Temperature 1 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <TimeSeriesValueNumeric
                  label="Temperature 1"
                  unit="C"
                  timeseries={temperature1}
                  renderValue={(value) => roundToDecimals(value, 1)}
                />
              </div>

              <TemperatureBar
                min={0}
                max={150}
                minLimit={
                  state?.quality_control_state?.temperature1.min_temperature ??
                  0
                }
                maxLimit={
                  state?.quality_control_state?.temperature1.max_temperature ??
                  150
                }
                current={temperature1.current?.value ?? 0}
              />

              <div className="flex flex-row flex-wrap gap-4">
                <Label label="Min Temperature">
                  <EditValue
                    value={
                      state?.quality_control_state?.temperature1.min_temperature
                    }
                    unit="C"
                    title="Min Temperature 1"
                    defaultValue={
                      defaultState?.quality_control_state?.temperature1
                        .min_temperature
                    }
                    min={0}
                    max={Math.min(
                      149,
                      (state?.quality_control_state?.temperature1
                        .max_temperature ?? 150) - 1,
                    )}
                    renderValue={(value) => roundToDecimals(value, 1)}
                    onChange={setTemperature1Min}
                  />
                </Label>
                <Label label="Max Temperature">
                  <EditValue
                    value={
                      state?.quality_control_state?.temperature1.max_temperature
                    }
                    unit="C"
                    title="Max Temperature 1"
                    defaultValue={
                      defaultState?.quality_control_state?.temperature1
                        .max_temperature
                    }
                    min={Math.max(
                      1,
                      (state?.quality_control_state?.temperature1
                        .min_temperature ?? 0) + 1,
                    )}
                    max={150}
                    renderValue={(value) => roundToDecimals(value, 1)}
                    onChange={setTemperature1Max}
                  />
                </Label>
              </div>
            </div>

            {/* Temperature 2 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <TimeSeriesValueNumeric
                  label="Temperature 2"
                  unit="C"
                  timeseries={temperature2}
                  renderValue={(value) => roundToDecimals(value, 1)}
                />
              </div>

              <TemperatureBar
                min={0}
                max={200}
                minLimit={
                  state?.quality_control_state?.temperature2.min_temperature ??
                  0
                }
                maxLimit={
                  state?.quality_control_state?.temperature2.max_temperature ??
                  200
                }
                current={temperature2.current?.value ?? 0}
              />

              <div className="flex flex-row flex-wrap gap-4">
                <Label label="Min Temperature">
                  <EditValue
                    value={
                      state?.quality_control_state?.temperature2.min_temperature
                    }
                    unit="C"
                    title="Min Temperature 2"
                    defaultValue={
                      defaultState?.quality_control_state?.temperature2
                        .min_temperature
                    }
                    min={0}
                    max={Math.min(
                      199,
                      (state?.quality_control_state?.temperature2
                        .max_temperature ?? 200) - 1,
                    )}
                    renderValue={(value) => roundToDecimals(value, 1)}
                    onChange={setTemperature2Min}
                  />
                </Label>
                <Label label="Max Temperature">
                  <EditValue
                    value={
                      state?.quality_control_state?.temperature2.max_temperature
                    }
                    unit="C"
                    title="Max Temperature 2"
                    defaultValue={
                      defaultState?.quality_control_state?.temperature2
                        .max_temperature
                    }
                    min={Math.max(
                      1,
                      (state?.quality_control_state?.temperature2
                        .min_temperature ?? 0) + 1,
                    )}
                    max={200}
                    renderValue={(value) => roundToDecimals(value, 1)}
                    onChange={setTemperature2Max}
                  />
                </Label>
              </div>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Tension Arm">
          <TensionArm degrees={slaveTensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Tension Arm"
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
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
