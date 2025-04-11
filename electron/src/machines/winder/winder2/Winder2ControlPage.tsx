import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { TraverseBar } from "../TraverseBar";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { useWinder2 } from "./useWinder";
import { Mode } from "./winder2Namespace";
import { TensionArm } from "../TensionArm";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { Spool } from "../Spool";

export function Winder1ControlPage() {
  // use optimistic state
  const {
    laserpointer,
    setLaserpointer,
    laserpointerIsLoading,
    laserpointerIsDisabled,
    tensionArmAngle,
    tensionArmAngleZero,
    spoolRpm,
    mode,
    setMode,
    modeIsLoading,
    modeIsDisabled,
  } = useWinder2();

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Spool">
          <Spool rpm={spoolRpm.current?.value} />
          <TimeSeriesValueNumeric
            label="Spool Speed"
            unit="rpm"
            timeseries={spoolRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>

        <ControlCard className="bg-red" height={2} title="Traverse">
          {/* <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            value={55}
            renderValue={(value) => roundToDecimals(value, 0)}
          /> */}
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
                value={undefined}
                unit="mm"
                title="Outer Limit"
                defaultValue={16}
                min={0}
                minLabel="IN"
                maxLabel="OUT"
                max={80}
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
              />
              <TouchButton variant="outline" icon="lu:ArrowLeftToLine">
                Go to Outer Limit
              </TouchButton>
            </Label>
            <Label label="Inner Limit">
              <EditValue
                value={undefined}
                unit="mm"
                title="Limit Innen"
                min={0}
                max={80}
                defaultValue={72}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => roundToDecimals(value, 0)}
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

        <ControlCard title="Tension Arm">
          <TensionArm degrees={tensionArmAngle.current?.value} />
          <TimeSeriesValueNumeric
            label="Tension Arm"
            unit="deg"
            timeseries={tensionArmAngle}
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:House"
            onClick={tensionArmAngleZero}
          >
            Set Zero Point
          </TouchButton>
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<Mode>
            value={mode}
            disabled={modeIsDisabled}
            loading={modeIsLoading}
            onChange={setMode}
            orientation="vertical"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
              },
              Wind: {
                children: "Wind",
                icon: "lu:RefreshCcw",
                isActiveClassName: "bg-green-600",
              },
            }}
          />
        </ControlCard>

        <ControlCard className="bg-red" title="Puller">
          {/* <TimeSeriesValueNumeric
            label="Speed"
            unit="m/s"
            value={16}
            renderValue={(value) => roundToDecimals(value, 0)}
          /> */}
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
              value={undefined}
              unit="m/s"
              title="Target Speed"
              defaultValue={0}
              min={0}
              max={100}
              step={1}
              renderValue={(value) => roundToDecimals(value, 0)}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
