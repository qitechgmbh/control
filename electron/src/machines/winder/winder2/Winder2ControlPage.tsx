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

export function Winder2ControlPage() {
  // use optimistic state
  const {
    laserpointer,
    setLaserpointer,
    laserpointerIsLoading,
    laserpointerIsDisabled,
    tensionArmAngle,
    tensionArmAngleZero,
    tensionArmState,
    tensionArmStateIsLoading,
    tensionArmStateIsDisabled,
    spoolRpm,
    mode,
    ExtruderSetMode,
    modeIsLoading,
    modeIsDisabled,
    pullerState,
    pullerSpeed,
    pullerSetRegulationMode,
    pullerSetTargetSpeed,
    pullerStateIsLoading,
    pullerStateIsDisabled,
    traversePosition,
    traverseState,
    traverseSetLimitInner,
    traverseSetLimitOuter,
    traverseGotoLimitInner,
    traverseGotoLimitOuter,
    traverseGotoHome,
    traverseStateIsLoading,
    traverseStateIsDisabled,
    modeState,
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
          <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            timeseries={traversePosition}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          {traverseState && (
            <TraverseBar
              inside={0}
              outside={180}
              min={traverseState.data.limit_inner}
              max={traverseState.data.limit_outer}
              current={traversePosition.current?.value ?? 0}
            />
          )}
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Outer Limit">
              <EditValue
                value={traverseState?.data.limit_outer}
                unit="mm"
                title="Outer Limit"
                defaultValue={80}
                // Traverse limit validation: Outer limit must be at least 0.9mm greater than inner limit
                // We use 1mm buffer to ensure the backend validation (which requires >0.9mm) will pass
                // Formula: min_outer = inner_limit + 1mm
                min={Math.max(0, (traverseState?.data.limit_inner ?? 0) + 1)}
                minLabel="IN"
                maxLabel="OUT"
                max={180}
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={traverseSetLimitOuter}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowLeftToLine"
                onClick={traverseGotoLimitOuter}
                disabled={
                  traverseStateIsDisabled || !traverseState?.data.can_go_out
                }
                isLoading={
                  traverseStateIsLoading || traverseState?.data.is_going_out
                }
              >
                Go to Outer Limit
              </TouchButton>
            </Label>
            <Label label="Inner Limit">
              <EditValue
                value={traverseState?.data.limit_inner}
                unit="mm"
                title="Inner Limit"
                min={0}
                // Traverse limit validation: Inner limit must be at least 0.9mm smaller than outer limit
                // We use 1mm buffer to ensure the backend validation (which requires outer > inner + 0.9mm) will pass
                // Formula: max_inner = outer_limit - 1mm
                max={Math.min(
                  180,
                  (traverseState?.data.limit_outer ?? 180) - 1,
                )}
                defaultValue={16}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={traverseSetLimitInner}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowRightToLine"
                onClick={traverseGotoLimitInner}
                disabled={
                  traverseStateIsDisabled || !traverseState?.data.can_go_in
                }
                isLoading={
                  traverseStateIsLoading || traverseState?.data.is_going_in
                }
              >
                Go to Inner Limit
              </TouchButton>
            </Label>
          </div>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={laserpointer}
              disabled={
                laserpointerIsLoading ||
                traverseStateIsLoading ||
                traverseStateIsDisabled
              }
              loading={laserpointerIsDisabled}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
              optionTrue={{ children: "On", icon: "lu:Lightbulb" }}
              onChange={setLaserpointer}
            />
          </Label>
          <Label label="Home">
            <TouchButton
              variant="outline"
              icon="lu:House"
              onClick={() => traverseGotoHome()}
              disabled={
                traverseStateIsDisabled || !traverseState?.data.can_go_home
              }
              isLoading={
                traverseStateIsLoading || traverseState?.data.is_going_home
              }
            >
              Go to Home
            </TouchButton>
            {traverseState?.data.is_homed !== true ? (
              <StatusBadge variant={"error"}>{"Not Homed"}</StatusBadge>
            ) : null}
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
            disabled={tensionArmStateIsDisabled}
            isLoading={tensionArmStateIsLoading}
          >
            Set Zero Point
          </TouchButton>
          {!tensionArmState?.data.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<Mode>
            value={mode}
            disabled={modeIsDisabled}
            loading={modeIsLoading}
            onChange={ExtruderSetMode}
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
                disabled: !modeState?.data.can_wind,
              },
            }}
          />
        </ControlCard>

        <ControlCard className="bg-red" title="Puller">
          <TimeSeriesValueNumeric
            label="Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
          <Label label="Regulation">
            <SelectionGroup
              value={pullerState?.data.regulation}
              options={{
                Speed: {
                  children: "Speed",
                  icon: "lu:Gauge",
                },
                Diameter: {
                  children: "Diameter (Sync to DRE™)",
                  icon: "lu:Diameter",
                  disabled: true,
                },
              }}
              onChange={pullerSetRegulationMode}
              disabled={pullerStateIsDisabled}
              loading={pullerStateIsLoading}
            />
          </Label>
          <Label label="Target Speed">
            <EditValue
              value={pullerState?.data.target_speed}
              unit="m/min"
              title="Target Speed"
              defaultValue={1}
              min={0}
              max={75}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={pullerSetTargetSpeed}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
