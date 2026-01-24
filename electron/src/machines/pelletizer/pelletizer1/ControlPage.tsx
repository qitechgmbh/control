import React from "react";

import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { MinMaxValue, TIMEFRAME_OPTIONS } from "@/control/MinMaxValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { usePellet1 as usePellet1 } from "./usePelletizer";

import { SelectionGroupBoolean } from "@/control/SelectionGroup";
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
        SetRunMode,
        SetFrequencyTarget,
        SetAccelerationLevel,
        SetDecelerationLevel,
    } = usePellet1();

    // Extract values from consolidated state
    const running_state  = state?.inverter_state?.running_state ?? 0;
    const frequency_target = state?.inverter_state?.frequency_target ?? 0;
    const acceleration_level = state?.inverter_state?.acceleration_level ?? 0;
    const deceleration_level = state?.inverter_state?.deceleration_level ?? 0;
    const system_status = state?.inverter_state?.system_status ?? 0;

    // Shared timeframe state (default 5 minutes)
    const [timeframe, setTimeframe] = React.useState<number>(5 * 60 * 1000);

    return (
        <Page>
            <ControlGrid columns={2}>
                <ControlCard title="Inverter Configuration">
                    <Label label="Rotation Direction">
                        <SelectionGroupBoolean
                            value={true}
                            optionTrue={{ children: "Forward" }}
                            optionFalse={{ children: "Backward" }}
                            onChange={(v) => {}}
                        />
                    </Label>

                    <Label label="Set Target Frequency">
                        <EditValue
                            title="Set Target Frequency"
                            value={0}
                            unit="Hz"
                            step={1}
                            min={0}
                            max={99}
                            renderValue={(value) => value.toFixed(0)}
                            onChange={(val) => {
                                SetFrequencyTarget(val);
                            }}
                            defaultValue={5}
                        />
                    </Label>
                    <Label label="Set Acceleration Level">
                        <EditValue
                            title="Set Acceleration Level"
                            value={1}
                            unit={undefined}
                            step={1}
                            min={1}
                            max={15}
                            renderValue={(value) => value.toFixed(0)}
                            onChange={(val) => SetAccelerationLevel(val)}
                            defaultValue={7}
                        />
                    </Label>
                    <Label label="Set Deceleration Level">
                        <EditValue
                            title="Set Deceleration Level"
                            value={1}
                            unit={undefined}
                            step={1}
                            min={1}
                            max={15}
                            renderValue={(value) => value.toFixed(0)}
                            onChange={(val) => SetDecelerationLevel(val)}
                            defaultValue={7}
                        />
                    </Label>
                </ControlCard>

                <ControlCard title="Inverter Status">
                    <Badge
                        className={`text-md ${true ? "bg-white-600 border-green-600 text-green-600" : "bg-red-600 font-bold text-white"} mx-auto h-12 w-[100%] border-3 text-lg`}
                    >
                        {true ? "Running" : "PULSE OVERCURRENT"}
                    </Badge>

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