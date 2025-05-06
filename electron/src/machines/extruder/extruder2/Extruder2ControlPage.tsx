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

export function Extruder2ControlPage() {
  const {
    mode,
    SetMode,
    modeIsLoading,
    modeIsDisabled,
    inverterSetRotation,
    rotationState,
    frontHeatingState,
    backHeatingState,
    middleHeatingState,
    SetRegulation,
    uses_rpm,
    rpm,
    bar,
  } = useExtruder2();

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heating Front"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <HeatingZone
          title={"Heating Middle"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <HeatingZone
          title={"Heating Back"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <ControlCard className="bg-red" title="Screw Drive">
          {/* <TimeSeriesValueNumeric
            label="Drehzahl"
            unit="rpm"
            timeseries={null}
            renderValue={(value) => roundToDecimals(value, 0) || "N/A"}
          /> */}
          <Label label="Regulation">
            <SelectionGroupBoolean
              value={uses_rpm}
              optionFalse={{ children: "RPM" }}
              optionTrue={{ children: "Pressure" }}
              onChange={SetRegulation}
            />
          </Label>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Target RPM">
              <EditValue
                value={rpm}
                defaultValue={0}
                unit="rpm"
                title="Target RPM"
                renderValue={(value) => roundToDecimals(value, 0)}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={bar}
                defaultValue={200}
                unit="bar"
                title="Target Pressure"
                renderValue={(value) => roundToDecimals(value, 0)}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Measurements">
          {/* <TimeSeriesValueNumeric
            label="Nozzle Pressure"
            unit="bar"
            value={55}
            renderValue={(value) => roundToDecimals(value, 0)}
          /> */}
        </ControlCard>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Heat" | "Extrude">
            value={mode}
            orientation="vertical"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
              },
              Heat: {
                children: "Heat",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
              },
              Extrude: {
                children: "Extrude",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
              },
            }}
            onChange={SetMode}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
