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
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { useWinder1 } from "./hooks";

export function Winder1ControlPage() {
  // use optimistic state
  const {
    laserpointer,
    setLaserpointer,
    laserpointerIsLoading,
    laserpointerIsDisabled,
  } = useWinder1();

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
            <Label label="Outer Limit">
              <EditValue
                value={16}
                unit="mm"
                title="Outer Limit"
                defaultValue={16}
                min={0}
                minLabel="IN"
                maxLabel="OUT"
                max={80}
                renderValue={(value) => value.toFixed(0)}
                inverted
              />
              <TouchButton variant="outline" icon="lu:ArrowLeftToLine">
                Go to Outer Limit
              </TouchButton>
            </Label>
            <Label label="Inner Limit">
              <EditValue
                value={72}
                unit="mm"
                title="Limit Innen"
                min={0}
                max={80}
                defaultValue={72}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => value.toFixed(0)}
                inverted
              />
              <TouchButton variant="outline" icon="lu:ArrowRightToLine">
                Go to Inner Limit
              </TouchButton>
            </Label>
          </div>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={laserpointer}
              disabled={laserpointerIsLoading}
              loading={laserpointerIsDisabled}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
              optionTrue={{ children: "On", icon: "lu:Lightbulb" }}
              onChange={setLaserpointer}
            />
          </Label>
          <Label label="Home">
            <TouchButton variant="outline" icon="lu:House" isLoading>
              Go to Home
            </TouchButton>
            <StatusBadge variant="error">Not Homed</StatusBadge>
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Puller">
          <ControlValueNumeric
            label="Speed"
            unit="m/s"
            value={16}
            renderValue={(value) => value.toFixed(0)}
          />
          <Label label="Regulation">
            <SelectionGroupBoolean
              value={false}
              optionFalse={{ children: "Speed", icon: "lu:Gauge" }}
              optionTrue={{
                children: "Diameter (Sync to DRE™)",
                icon: "lu:Diameter",
              }}
            />
          </Label>
          <Label label="Target Speed">
            <EditValue
              value={16}
              unit="m/s"
              title="Target Speed"
              defaultValue={0}
              min={0}
              max={100}
              step={1}
              renderValue={(value) => value.toFixed(0)}
            />
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Mode">
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
            label="Wounded Length"
            unit="m"
            value={14}
            renderValue={(value) => value.toFixed(0)}
          />
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Enable">
              <SelectionGroupBoolean
                value={false}
                optionFalse={{ children: "Off" }}
                optionTrue={{ children: "On" }}
              />
            </Label>
            <Label label="Limit">
              <EditValue
                value={72}
                defaultValue={200}
                min={0}
                max={1000}
                unit="m"
                title="Edit"
                renderValue={(value) => value.toFixed(0)}
              />
            </Label>
          </div>
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Alarm Signal">
              <SelectionGroupBoolean
                value={false}
                optionFalse={{ children: "Off" }}
                optionTrue={{ children: "On" }}
              />
            </Label>
            <Label label="After Stop Transition Into">
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
        <ControlCard className="bg-red" title="Measurements">
          <ControlValueNumeric
            label="Winding RPM"
            unit="rpm"
            value={55}
            renderValue={(value) => value.toFixed(0)}
          />
          <ControlValueNumeric
            label="Tension Arm"
            unit="deg"
            value={5}
            renderValue={(value) => value.toFixed(0)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
