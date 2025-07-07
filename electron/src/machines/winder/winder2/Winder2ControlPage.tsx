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
    state,
    defaultState,
    enableTraverseLaserpointer,
    tensionArmAngle,
    zeroTensionArmAngle,
    spoolRpm,
    spoolDiameter,
    setMode,
    pullerSpeed,
    pullerProgress,
    setPullerRegulationMode,
    setPullerTargetSpeed,
    traversePosition,
    setTraverseLimitInner,
    setTraverseLimitOuter,
    gotoTraverseLimitInner,
    gotoTraverseLimitOuter,
    gotoTraverseHome,
    setPullerAutoStopExpectedMeters,
    setPullerAutoStopEnabled,
    setPullerAutoStop,
    isLoading,
    isDisabled,
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
          <TimeSeriesValueNumeric
            label="Estimated Diameter"
            unit="cm"
            timeseries={spoolDiameter}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
        </ControlCard>

        <ControlCard className="bg-red" height={2} title="Traverse">
          <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            timeseries={traversePosition}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          {state?.traverse_state && (
            <TraverseBar
              inside={0}
              outside={180}
              min={state?.traverse_state.limit_inner}
              max={state?.traverse_state.limit_outer}
              current={traversePosition.current?.value ?? 0}
            />
          )}
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Outer Limit">
              <EditValue
                value={state?.traverse_state?.limit_outer}
                unit="mm"
                title="Outer Limit"
                defaultValue={defaultState?.traverse_state?.limit_outer}
                // Traverse limit validation: Outer limit must be at least 0.9mm greater than inner limit
                // We use 1mm buffer to ensure the backend validation (which requires >0.9mm) will pass
                // Formula: min_outer = inner_limit + 1mm
                min={Math.max(0, (state?.traverse_state?.limit_inner ?? 0) + 1)}
                minLabel="IN"
                maxLabel="OUT"
                max={180}
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={setTraverseLimitOuter}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowLeftToLine"
                onClick={gotoTraverseLimitOuter}
                disabled={isDisabled || !state?.traverse_state?.can_go_out}
                isLoading={isLoading || state?.traverse_state?.is_going_out}
              >
                Go to Outer Limit
              </TouchButton>
            </Label>
            <Label label="Inner Limit">
              <EditValue
                value={state?.traverse_state?.limit_inner}
                unit="mm"
                title="Inner Limit"
                min={0}
                // Traverse limit validation: Inner limit must be at least 0.9mm smaller than outer limit
                // We use 1mm buffer to ensure the backend validation (which requires outer > inner + 0.9mm) will pass
                // Formula: max_inner = outer_limit - 1mm
                max={Math.min(
                  180,
                  (state?.traverse_state?.limit_outer ?? 180) - 1,
                )}
                defaultValue={defaultState?.traverse_state?.limit_inner}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={setTraverseLimitInner}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowRightToLine"
                onClick={gotoTraverseLimitInner}
                disabled={isDisabled || !state?.traverse_state?.can_go_in}
                isLoading={isLoading || state?.traverse_state?.is_going_in}
              >
                Go to Inner Limit
              </TouchButton>
            </Label>
          </div>
          <Label label="Laserpointer">
            <SelectionGroupBoolean
              value={state?.traverse_state.laserpointer}
              disabled={isLoading || isDisabled}
              loading={isLoading}
              optionFalse={{ children: "Off", icon: "lu:LightbulbOff" }}
              optionTrue={{ children: "On", icon: "lu:Lightbulb" }}
              onChange={enableTraverseLaserpointer}
            />
          </Label>
          <Label label="Home">
            <TouchButton
              variant="outline"
              icon="lu:House"
              onClick={() => gotoTraverseHome()}
              disabled={isDisabled || !state?.traverse_state?.can_go_home}
              isLoading={isLoading || state?.traverse_state?.is_going_home}
            >
              Go to Home
            </TouchButton>
            {state?.traverse_state?.is_homed !== true ? (
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
            onClick={zeroTensionArmAngle}
            disabled={isDisabled}
            isLoading={isLoading}
          >
            Set Zero Point
          </TouchButton>
          {!state?.tension_arm_state?.zeroed && (
            <StatusBadge variant="error">Not Zeroed</StatusBadge>
          )}
        </ControlCard>

        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Pull: {
                children: "Pull",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Wind: {
                children: "Wind",
                icon: "lu:RefreshCcw",
                isActiveClassName: "bg-green-600",
                disabled: !state?.mode_state?.can_wind,
                className: "h-full",
              },
            }}
          />
        </ControlCard>

        <ControlCard className="bg-red" title="Puller">
          <TimeSeriesValueNumeric
            label="Speed"
            unit="m/min"
            timeseries={pullerSpeed}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          <Label label="Regulation">
            <SelectionGroup
              value={state?.puller_state?.regulation}
              options={{
                Speed: {
                  children: "Speed",
                  icon: "lu:Gauge",
                },
                Diameter: {
                  children: "Diameter",
                  icon: "lu:Sun",
                  disabled: true,
                },
              }}
              onChange={setPullerRegulationMode}
              disabled={isDisabled}
              loading={isLoading}
            />
          </Label>
          <Label label="Target Speed">
            <EditValue
              value={state?.puller_state?.target_speed}
              unit="m/min"
              title="Target Speed"
              defaultValue={defaultState?.puller_state?.target_speed}
              min={0}
              max={75}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setPullerTargetSpeed}
            />
          </Label>
        </ControlCard>

        <ControlCard className="bg-red" title="Puller Auto Stop/Pull">
          <Label label="Meters Until Automatic Action">
            <EditValue
              value={state?.puller_auto_stop_state.puller_expected_meters}
              unit="m"
              title="Expected Meters"
              defaultValue={0}
              min={0}
              max={100}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={setPullerAutoStopExpectedMeters}
            />
          </Label>

          <Label label="Automatic Action Enabled">
            <SelectionGroupBoolean
              value={state?.puller_auto_stop_state.puller_auto_enabled}
              optionTrue={{ children: "Enabled" }}
              optionFalse={{ children: "Disabled" }}
              onChange={setPullerAutoStopEnabled}
              disabled={isDisabled}
              loading={isLoading}
            />
          </Label>
          <Label label="Automatic Action">
            <SelectionGroupBoolean
              value={state?.puller_auto_stop_state.puller_auto_stop}
              optionTrue={{ children: "Stop" }}
              optionFalse={{ children: "Pull" }}
              onChange={setPullerAutoStop}
              disabled={isDisabled}
              loading={isLoading}
            />{" "}
          </Label>

          <TimeSeriesValueNumeric
            label="Progress"
            renderValue={(value) => roundToDecimals(value, 2) + "%"}
            timeseries={pullerProgress}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
