import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlValueNumeric } from "@/control/ControlValue";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { HeatingZone } from "../HeatingZone";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";

export function Extruder1ControlPage() {
  return (
    <Page>
      <ControlGrid>
        <HeatingZone
          title={"Heizzone Vorne"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <HeatingZone
          title={"Heizzone Mitte"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <HeatingZone
          title={"Heizzone Hinten"}
          temperature={150}
          heating={true}
          targetTemperature={155}
        />
        <ControlCard className="bg-red" title="Antrieb">
          <ControlValueNumeric
            label="Drehzahl"
            unit="rpm"
            value={11}
            renderValue={(value) => value.toFixed(0)}
          />
          <Label label="Regelung">
            <SelectionGroupBoolean
              value={false}
              optionFalse={{ children: "Drehzahl" }}
              optionTrue={{ children: "Massedruck" }}
            />
          </Label>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Zieldrehzahl">
              <EditValue
                value={16}
                unit="rpm"
                title="Zieldrehzahl"
                renderValue={(value) => value.toFixed(0)}
              />
            </Label>
            <Label label="Zieldruck">
              <EditValue
                value={300}
                unit="bar"
                title="Zieldruck"
                renderValue={(value) => value.toFixed(0)}
              />
            </Label>
          </div>
        </ControlCard>

        <ControlCard className="bg-red" title="Messwerte">
          <ControlValueNumeric
            label="Massedruck"
            unit="bar"
            value={55}
            renderValue={(value) => value.toFixed(0)}
          />
        </ControlCard>
        <ControlCard className="bg-red" title="Modus">
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
                children: "Heizen",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
              },
              extrude: {
                children: "Extrudieren",
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
