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
    mode,
    nozzleHeatingState,
    nozzleTemperature,
    nozzlePower,

    frontHeatingState,
    frontTemperature,
    frontPower,

    backHeatingState,
    backTemperature,
    backPower,

    middleHeatingState,
    middleTemperature,
    middlePower,

    extruderSetMode,
    heatingSetBackTemp,
    heatingSetFrontTemp,
    heatingSetMiddleTemp,
    heatingSetNozzleTemp,
    screwSetRegulation,
    screwSetTargetPressure,
    screwSetTargetRpm,

    uses_rpm,
    bar,
    rpm,
    targetBar,
    targetRpm,
    inverterState,
  } = useExtruder2();

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heating Front"}
          heatingState={frontHeatingState}
          heatingTimeSeries={frontTemperature}
          heatingPower={frontPower}
          onChangeTargetTemp={heatingSetFrontTemp}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Middle"}
          heatingState={middleHeatingState}
          heatingTimeSeries={middleTemperature}
          heatingPower={middlePower}
          onChangeTargetTemp={heatingSetMiddleTemp}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Back"}
          heatingState={backHeatingState}
          heatingTimeSeries={backTemperature}
          heatingPower={backPower}
          onChangeTargetTemp={heatingSetBackTemp}
          min={0}
          max={300}
        />
        <HeatingZone
          title={"Heating Nozzle"}
          heatingState={nozzleHeatingState}
          heatingTimeSeries={nozzleTemperature}
          heatingPower={nozzlePower}
          onChangeTargetTemp={heatingSetNozzleTemp}
          min={0}
          max={300}
        />
        <ControlCard className="bg-red" title="Screw Drive">
          {inverterState?.fault_occurence == true && (
            <StatusBadge variant="error">
              Inverter encountered an error!! Press the restart button in Config
            </StatusBadge>
          )}
          {inverterState?.running == true &&
            inverterState.fault_occurence == false && (
              <StatusBadge variant="success">Running</StatusBadge>
            )}
          {inverterState?.running == false &&
            inverterState.fault_occurence == false && (
              <StatusBadge variant="success">Healthy</StatusBadge>
            )}

          <Label label="Regulation">
            <SelectionGroupBoolean
              value={uses_rpm}
              optionTrue={{ children: "RPM" }}
              optionFalse={{ children: "Pressure" }}
              onChange={screwSetRegulation}
            />
          </Label>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Target Output RPM">
              <EditValue
                value={targetRpm}
                defaultValue={0}
                unit="rpm"
                title="Target Output RPM"
                min={0}
                max={106}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={screwSetTargetRpm}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={targetBar}
                defaultValue={0}
                unit="bar"
                title="Target Pressure"
                min={0.0}
                max={150.0}
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={screwSetTargetPressure}
              />
            </Label>
          </div>
          <div className="flex flex-row flex-wrap gap-4">
            <TimeSeriesValueNumeric
              label="Rpm"
              unit="rpm"
              renderValue={(value) => roundToDecimals(value, 0)}
              timeseries={rpm}
            />

            <TimeSeriesValueNumeric
              label="Pressure"
              unit="bar"
              renderValue={(value) => roundToDecimals(value, 0)}
              timeseries={bar}
            />
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Heat" | "Extrude">
            value={mode}
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
            onChange={extruderSetMode}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
