import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";

import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { StatusBadge } from "@/control/StatusBadge";
import { TraverseBar } from "@/machines/winder/TraverseBar";

export function Buffer1ControlPage() {
  const {
    state,
    defaultState,
    setBufferMode,
    setCurrentInputSpeed,
    pullerSpeed,
    setPullerRegulationMode,
    setPullerTargetSpeed,
    liftPosition,
    setLiftLimitTop,
    setLiftLimitBottom,
    gotoLiftLimitTop,
    gotoLiftLimitBottom,
    gotoLiftHome,
    isLoading,
    isDisabled,
  } = useBuffer1();

  const current_input_speed =
    state?.current_input_speed_state.current_input_speed ?? 0.0;
  const default_speed = 0.0;

  return (
    <Page>
      <ControlGrid>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "Hold" | "Filling" | "Emptying">
            value={state?.mode_state.mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:Pause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Filling: {
                children: "Filling",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Emptying: {
                children: "EmptyingBuffer",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={setBufferMode}
          />
        </ControlCard>
        <ControlCard title="Set Current Input Speed - (For Debugging)">
          <EditValue
            title="Input Speed"
            unit="m/min"
            value={current_input_speed}
            defaultValue={default_speed}
            min={0.0}
            max={60.0}
            step={0.1}
            renderValue={(value) => roundToDecimals(value, 1)}
            onChange={setCurrentInputSpeed}
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

        <ControlCard className="bg-red" height={2} title="Lift">
          <TimeSeriesValueNumeric
            label="Position"
            unit="mm"
            timeseries={liftPosition}
            renderValue={(value) => roundToDecimals(value, 1)}
          />
          {state?.lift_state && (
            <TraverseBar
              inside={0}
              outside={180}
              min={state?.lift_state.limit_bottom}
              max={state?.lift_state.limit_top}
              current={liftPosition.current?.value ?? 0}
            />
          )}
          <div className="flex flex-row flex-wrap gap-4">
            <Label label="Bottom Limit">
              <EditValue
                value={state?.lift_state?.limit_bottom}
                unit="mm"
                title="Bottom Limit"
                defaultValue={defaultState?.lift_state?.limit_bottom}
                // lift limit validation: Bottom limit must be at least 0.9mm greater than inner limit
                // We use 1mm buffer to ensure the backend validation (which requires >0.9mm) will pass
                // Formula: min_outer = inner_limit + 1mm
                min={Math.max(0, (state?.lift_state?.limit_bottom ?? 0) + 1)}
                minLabel="IN"
                maxLabel="OUT"
                max={130}
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={setLiftLimitBottom}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowLeftToLine"
                onClick={gotoLiftLimitBottom}
                disabled={isDisabled}
                isLoading={isLoading || state?.lift_state?.is_going_down}
              >
                Go to Bottom Limit
              </TouchButton>
            </Label>
            <Label label="Top Limit">
              <EditValue
                value={state?.lift_state?.limit_top}
                unit="mm"
                title="Top Limit"
                min={0}
                // lift limit validation: Top limit must be at least 0.9mm smaller than outer limit
                // We use 1mm buffer to ensure the backend validation (which requires outer > inner + 0.9mm) will pass
                // Formula: max_inner = outer_limit - 1mm
                max={Math.min(180, (state?.lift_state?.limit_top ?? 180) - 1)}
                defaultValue={defaultState?.lift_state?.limit_top}
                minLabel="IN"
                maxLabel="OUT"
                renderValue={(value) => roundToDecimals(value, 0)}
                inverted
                onChange={setLiftLimitTop}
              />
              <TouchButton
                variant="outline"
                icon="lu:ArrowRightToLine"
                onClick={gotoLiftLimitTop}
                disabled={isDisabled}
                isLoading={isLoading || state?.lift_state?.is_going_up}
              >
                Go to Top Limit
              </TouchButton>
            </Label>
          </div>
          <Label label="Home">
            <TouchButton
              variant="outline"
              icon="lu:House"
              onClick={() => gotoLiftHome()}
              disabled={isDisabled}
              isLoading={isLoading || state?.lift_state?.is_going_home}
            >
              Go to Home
            </TouchButton>
            {state?.lift_state?.is_homed !== true ? (
              <StatusBadge variant={"error"}>{"Not Homed"}</StatusBadge>
            ) : null}
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
