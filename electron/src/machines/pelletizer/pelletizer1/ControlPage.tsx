import React from "react";

import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { MinMaxValue, TIMEFRAME_OPTIONS } from "@/control/MinMaxValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { usePellet1 as usePellet1 } from "./usePelletizer";

import { SelectionGroup, SelectionGroupBoolean } from "@/control/SelectionGroup";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { roundToDecimals } from "@/lib/decimal";
import { Badge } from "@/components/ui/badge";

export function Pellet1ControlPage() {
    const {
        //state
        state,
        defaultState,
        
        //live values
        frequency,
        temperature,
        voltage,
        current,

        // mutation functions
        SetRunning,
        SetDirection,
        SetFrequencyTarget,
        SetAccelerationLevel,
        SetDecelerationLevel,
    } = usePellet1();

    // Extract values from consolidated state
    const running            = state?.inverter_state?.running ?? false;
    const direction          = state?.inverter_state?.direction ?? true;
    const frequency_target   = state?.inverter_state?.frequency_target ?? 0;
    const acceleration_level = state?.inverter_state?.acceleration_level ?? 0;
    const deceleration_level = state?.inverter_state?.deceleration_level ?? 0;

    // Shared timeframe state (default 5 minutes)
    const [timeframe, setTimeframe] = React.useState<number>(5 * 60 * 1000);

    return (
        <Page>
            <ControlGrid columns={2}>
                <ControlCard title="Motor">
                    <SelectionGroup<"Off" | "On">
                        value={running ? "On" : "Off"}
                        orientation="vertical"
                        className="grid grid-cols-2 gap-2"
                        options={{
                        Off: {
                            children: "Off",
                            icon: "lu:CirclePause",
                            isActiveClassName: "bg-red-600",
                            className: "h-full",
                        },
                        On: {
                            children: "On",
                            icon: "lu:CirclePlay",
                            isActiveClassName: "bg-green-600",
                            className: "h-full",
                        },
                        }}
                        onChange={(v) => SetRunning(v == "On")}
                    />

                    <Label label="Speed">
                        <div className="flex flex-row flex-wrap gap-4">
                            <EditValue
                                title="Speed"
                                value={frequency_target / 14.3}
                                unit="rpm"
                                step={0.1}
                                min={0}
                                max={3.5}
                                renderValue={(value) => value.toFixed(1)}
                                onChange={(val) => {
                                    SetFrequencyTarget(val * 14.3);
                                }}
                                defaultValue={2}
                                
                            />
                        
                            <EditValue
                                h-full v-full
                                title="Speed"
                                value={frequency_target * 2}
                                unit="%"
                                step={0.5}
                                min={0}
                                max={100}
                                renderValue={(value) => value.toFixed(1)}
                                onChange={(val) => {
                                    SetFrequencyTarget(val / 2);
                                }}
                                defaultValue={5}
                            />
                        </div>
                    </Label>
                    
                </ControlCard>

                <ControlCard title="Inverter Status">
                    
                    {
                    /* 
                    
                    <Badge
                        className={`text-md ${true ? "bg-white-600 border-green-600 text-green-600" : "bg-red-600 font-bold text-white"} mx-auto h-12 w-[100%] border-3 text-lg`}
                    >
                        { true ? "Running" : "PULSE OVERCURRENT"}
                    </Badge>
                    */
                    }
                    

                    <TimeSeriesValueNumeric
                        label="Frequency"
                        unit="Hz"
                        renderValue={(value) => roundToDecimals(value, 1)}
                        timeseries={ frequency }
                    />

                    <TimeSeriesValueNumeric
                        label="Temperature"
                        unit="C"
                        renderValue={(value) => roundToDecimals(value, 1)}
                        timeseries={ temperature }
                    />

                    <TimeSeriesValueNumeric
                        label="Voltage"
                        unit="V"
                        renderValue={(value) => roundToDecimals(value, 1)}
                        timeseries={ voltage }
                    />

                    <TimeSeriesValueNumeric
                        label="Current"
                        unit="A"
                        renderValue={(value) => roundToDecimals(value, 1)}
                        timeseries={ current }
                    />
                </ControlCard>
            </ControlGrid>
        </Page>
    );
}