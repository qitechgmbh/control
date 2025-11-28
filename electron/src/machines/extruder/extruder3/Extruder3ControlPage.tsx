import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { HeatingZone } from "../HeatingZone";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder3 } from "./useExtruder";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { StatusBadge } from "@/control/StatusBadge";

export function Extruder3ControlPage() {
  const {
    state,
    defaultState,
    nozzleTemperature,
    nozzlePower,
    frontTemperature,
    frontPower,
    backTemperature,
    backPower,
    middleTemperature,
    middlePower,
    pressure,

    motorScrewRpm,
    motorPower,
    motorCurrent,
    totalEnergyKWh,
    combinedPower,

    setExtruderMode,
    setBackHeatingTemperature,
    setFrontHeatingTemperature,
    setMiddleHeatingTemperature,
    setNozzleHeatingTemperature,
    setInverterRegulation,
    setInverterTargetPressure,
    setInverterTargetRpm,

    isLoading,
    isDisabled,
  } = useExtruder3();

  function isZoneReadyForExtrusion(
    temperature: number,
    targetTemperature: number,
  ) {
    // if temperature is 90% of the target temperature, then we are ready for extrusion
    return temperature >= 0.9 * targetTemperature && targetTemperature > 0.0;
  }

  const allZonesReadyForExtrude = () => {
    const frontReady = isZoneReadyForExtrusion(
      frontTemperature.current?.value ?? 0,
      state?.heating_states.front.target_temperature ?? 1,
    );
    const middleReady = isZoneReadyForExtrusion(
      middleTemperature.current?.value ?? 0,
      state?.heating_states.middle.target_temperature ?? 1,
    );
    const backReady = isZoneReadyForExtrusion(
      backTemperature.current?.value ?? 0,
      state?.heating_states.back.target_temperature ?? 1,
    );
    const nozzleReady = isZoneReadyForExtrusion(
      nozzleTemperature.current?.value ?? 0,
      state?.heating_states.nozzle.target_temperature ?? 1,
    );

    return frontReady && middleReady && backReady && nozzleReady;
  };

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heating Front"}
          heatingState={state?.heating_states.front}
          heatingTimeSeries={frontTemperature}
          heatingPower={frontPower}
          onChangeTargetTemp={setFrontHeatingTemperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Middle"}
          heatingState={state?.heating_states.middle}
          heatingTimeSeries={middleTemperature}
          heatingPower={middlePower}
          onChangeTargetTemp={setMiddleHeatingTemperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Back"}
          heatingState={state?.heating_states.back}
          heatingTimeSeries={backTemperature}
          heatingPower={backPower}
          onChangeTargetTemp={setBackHeatingTemperature}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Nozzle"}
          heatingState={state?.heating_states.nozzle}
          heatingTimeSeries={nozzleTemperature}
          heatingPower={nozzlePower}
          onChangeTargetTemp={setNozzleHeatingTemperature}
          min={0}
          max={300}
        />
        <ControlCard className="bg-red" title="Screw Drive">
          {state?.inverter_status_state.overload_warning == true ? (
            <StatusBadge variant="error">
              Inverter is overloaded! Please check the extruder and reduce load
              if necessary.
            </StatusBadge>
          ) : state?.inverter_status_state.fault_occurence == true ? (
            <StatusBadge variant="error">
              Inverter encountered an error!! Press the restart button in Config
            </StatusBadge>
          ) : state?.inverter_status_state.running == true &&
            state.inverter_status_state.fault_occurence == false ? (
            <StatusBadge variant="success">Running</StatusBadge>
          ) : null}
          {state?.inverter_status_state.running == false &&
            state.inverter_status_state.fault_occurence == false && (
              <StatusBadge variant="success">Healthy</StatusBadge>
            )}

          <Label label="Regulation">
            <SelectionGroupBoolean
              value={state?.regulation_state.uses_rpm}
              optionTrue={{ children: "RPM" }}
              optionFalse={{ children: "Pressure" }}
              onChange={setInverterRegulation}
              disabled={isDisabled}
              loading={isLoading}
            />
          </Label>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Target Output RPM">
              <EditValue
                value={state?.screw_state.target_rpm}
                defaultValue={defaultState?.screw_state.target_rpm}
                unit="rpm"
                title="Target Output RPM"
                min={0}
                max={100}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setInverterTargetRpm}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={state?.pressure_state.target_bar}
                defaultValue={defaultState?.pressure_state.target_bar}
                unit="bar"
                title="Target Pressure"
                min={0.0}
                max={270.0}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setInverterTargetPressure}
              />
            </Label>
          </div>
          <div className="flex flex-row flex-wrap gap-4">
            <TimeSeriesValueNumeric
              label="Rpm"
              unit="rpm"
              renderValue={(value) => roundToDecimals(value, 0)}
              timeseries={motorScrewRpm}
            />

            {state?.pressure_state?.wiring_error && (
              <StatusBadge variant="error">
                Cant Measure Pressure! Check Pressure Sensor Wiring!
              </StatusBadge>
            )}
            <TimeSeriesValueNumeric
              label="Pressure"
              unit="bar"
              renderValue={(value) => roundToDecimals(value, 0)}
              timeseries={pressure}
            />
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Heat" | "Extrude">
            value={state?.mode_state.mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Heat: {
                children: "Heat",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Extrude: {
                children: "Extrude",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
                confirmation: allZonesReadyForExtrude()
                  ? undefined
                  : "Temperature is too low. Are you sure you want to extrude?",
              },
            }}
            onChange={setExtruderMode}
            disabled={isDisabled}
            loading={isLoading}
          />
        </ControlCard>

        <ControlCard className="bg-blue" title="Power Consumption">
          <TimeSeriesValueNumeric
            label="Total Power"
            unit="W"
            renderValue={(value) => roundToDecimals(value, 0)}
            timeseries={combinedPower}
          />
          <TimeSeriesValueNumeric
            label="Motor Power"
            unit="W"
            renderValue={(value) => roundToDecimals(value, 0)}
            timeseries={motorPower}
          />
          <TimeSeriesValueNumeric
            label="Motor Current"
            unit="A"
            renderValue={(value) => roundToDecimals(value, 1)}
            timeseries={motorCurrent}
          />
          <TimeSeriesValueNumeric
            label="Total Energy Consumption"
            unit="kWh"
            renderValue={(value) => roundToDecimals(value, 3)}
            timeseries={totalEnergyKWh}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
