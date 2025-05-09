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

export function Extruder1ControlPage() {
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
          title={"Headting Back"}
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
              value={false}
              optionFalse={{ children: "RPM" }}
              optionTrue={{ children: "Pressure" }}
            />
          </Label>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Target RPM">
              <EditValue
                value={undefined}
                defaultValue={0}
                unit="rpm"
                title="Target RPM"
                renderValue={(value) => roundToDecimals(value, 0)}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={undefined}
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
          <SelectionGroup<"standby" | "heating" | "extrude">
            value="standby"
            orientation="vertical"
            options={{
              standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
              },
              heating: {
                children: "Heat",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
              },
              extrude: {
                children: "Extrude",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
              },
            }}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
