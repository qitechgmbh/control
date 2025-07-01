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
import { useExtruder2 } from "./useExtruder";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { StatusBadge } from "@/control/StatusBadge";

export function Extruder2ControlPage() {
  const {
    state,
    nozzleTemperature,
    nozzlePower,
    frontTemperature,
    frontPower,
    backTemperature,
    backPower,
    middleTemperature,
    middlePower,
    screwRpm,
    pressure,

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
  } = useExtruder2();

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
          {state?.inverter_status_state.fault_occurence == true && (
            <StatusBadge variant="error">
              Inverter encountered an error!! Press the restart button in Config
            </StatusBadge>
          )}
          {state?.inverter_status_state.running == true &&
            state.inverter_status_state.fault_occurence == false && (
              <StatusBadge variant="success">Running</StatusBadge>
            )}
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
                defaultValue={0}
                unit="rpm"
                title="Target Output RPM"
                min={0}
                max={106}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={setInverterTargetRpm}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={state?.pressure_state.target_bar}
                defaultValue={0}
                unit="bar"
                title="Target Pressure"
                min={0.0}
                max={150.0}
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
              timeseries={screwRpm}
            />

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
              },
            }}
            onChange={setExtruderMode}
            disabled={isDisabled}
            loading={isLoading}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
