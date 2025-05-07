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
import { ReadOnlyValue } from "@/control/ReadonlyValue";

export function Extruder2ControlPage() {
  const {
    mode,
    SetMode,
    modeIsLoading,
    modeIsDisabled,
    inverterSetRotation,
    rotationState,
    frontHeatingTarget,
    backHeatingTarget,
    middleHeatingTarget,
    frontHeatingState,
    backHeatingState,
    middleHeatingState,
    SetHeatingFrontTemp,
    SetHeatingBackTemp,
    SetHeatingMiddleTemp,
    SetRegulation,
    uses_rpm,
    rpm,
    bar,
    targetBar,
    targetRpm,
    SetTargetPressure,
    SetTargetRpm,
  } = useExtruder2();

  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heating Front"}
          heatingState={frontHeatingState}
          onChangeTargetTemp={SetHeatingFrontTemp}
        />
        <HeatingZone
          title={"Heating Middle"}
          heatingState={middleHeatingState}
          onChangeTargetTemp={SetHeatingMiddleTemp}
        />
        <HeatingZone
          title={"Heating Back"}
          heatingState={backHeatingState}
          onChangeTargetTemp={SetHeatingBackTemp}
        />
        <ControlCard className="bg-red" title="Screw Drive">
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
                value={targetRpm}
                defaultValue={0}
                unit="rpm"
                title="Target RPM"
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={SetTargetRpm}
              />
            </Label>
            <Label label="Target Pressure">
              <EditValue
                value={targetBar}
                defaultValue={200}
                unit="bar"
                title="Target Pressure"
                renderValue={(value) => roundToDecimals(value, 0)}
                onChange={SetTargetPressure}
              />
            </Label>
          </div>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Actual RPM">
              <ReadOnlyValue
                value={rpm}
                unit="rpm"
                renderValue={(value) => roundToDecimals(value, 0)}
              />
            </Label>

            <Label label="Actual Pressure">
              <ReadOnlyValue
                value={bar}
                unit="bar"
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
