import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlValueNumeric } from "@/control/ControlValue";
import { TraverseBar } from "../TraverseBar";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/TouchButton";
import { Badge } from "@/components/ui/badge";
import { Icon } from "@/components/Icon";

export function Winder1ControlPage() {
  return (
    <Page>
      <ControlGrid>
        <ControlCard className="bg-red" height={2} title="Traverse">
          <ControlValueNumeric
            label="Position"
            unit="mm"
            value={55}
            renderValue={(value) => value.toFixed(0)}
          />
          <TraverseBar
            inside={0}
            outside={100}
            min={16}
            max={72}
            current={55}
          />
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Limit Aussen">
              <EditValue
                value={16}
                unit="mm"
                title="Edit"
                renderValue={(value) => value.toFixed(0)}
              />
              <TouchButton variant="outline" icon="lu:ChevronsLeft">
                Fahre nach Aussen
              </TouchButton>
            </Label>
            <Label label="Limit Innen">
              <EditValue
                value={72}
                unit="mm"
                title="Edit"
                renderValue={(value) => value.toFixed(0)}
              />
              <TouchButton variant="outline" icon="lu:ChevronsRight">
                Fahre nach Innen
              </TouchButton>
            </Label>
          </div>
          <Label label="Laser">
            <SelectionGroupBoolean
              value={false}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
              optionTrue={{ children: "On", icon: "lu:Lightbulb" }}
            />
          </Label>
          <Label label="Home">
            <TouchButton variant="outline" icon="lu:House" isLoading>
              Fahre nach Home
            </TouchButton>
            <Badge variant="destructive" className="text-md">
              <Icon name="lu:TriangleAlert" />
              Nicht gehomed
            </Badge>
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Puller">
          <ControlValueNumeric
            label="Geschwindigkeit"
            unit="m/s"
            value={16}
            renderValue={(value) => value.toFixed(0)}
          />
          <Label label="Konstant">
            <SelectionGroupBoolean
              value={false}
              optionFalse={{ children: "Konstant" }}
              optionTrue={{ children: "Synchronisiert mit DRE" }}
            />
          </Label>
          <Label label="Zielgeschwindigkeit">
            <EditValue
              value={16}
              unit="m/s"
              title="Zielgeschwindigkeit"
              renderValue={(value) => value.toFixed(0)}
            />
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Modus">
          <SelectionGroup<"standby" | "pull" | "wind">
            value="standby"
            orientation="vertical"
            options={{
              standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
              },
              pull: {
                children: "Pull",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
              },
              wind: {
                children: "Wind",
                icon: "lu:RefreshCcw",
                isActiveClassName: "bg-green-600",
              },
            }}
          />
        </ControlCard>
        <ControlCard className="bg-red" title="Auto Stop">
          <ControlValueNumeric
            label="Gewickelt"
            unit="m"
            value={14}
            renderValue={(value) => value.toFixed(0)}
          />
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Aktiv">
              <SelectionGroupBoolean
                value={false}
                optionFalse={{ children: "Off" }}
                optionTrue={{ children: "On" }}
              />
            </Label>
            <Label label="Limit">
              <EditValue
                value={72}
                unit="m"
                title="Edit"
                renderValue={(value) => value.toFixed(0)}
              />
            </Label>
          </div>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Alarmton">
              <SelectionGroupBoolean
                value={false}
                optionFalse={{ children: "Off" }}
                optionTrue={{ children: "On" }}
              />
            </Label>
            <Label label="Modus danach">
              <SelectionGroup<"standby" | "pull">
                value="standby"
                options={{
                  standby: { children: "Standby", icon: "lu:CirclePause" },
                  pull: { children: "Pull", icon: "lu:ChevronsLeft" },
                }}
              />
            </Label>
          </div>
        </ControlCard>
        <ControlCard className="bg-red" title="Messwerte">
          <ControlValueNumeric
            label="Winder Drehzahl"
            unit="rpm"
            value={55}
            renderValue={(value) => value.toFixed(0)}
          />
          <ControlValueNumeric
            label="Winkel Lastarm"
            unit="deg"
            value={5}
            renderValue={(value) => value.toFixed(0) + "°"}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
