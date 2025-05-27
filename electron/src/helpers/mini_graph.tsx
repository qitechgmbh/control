import React, { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
    newData: TimeSeries | null;
    width: number;
};

// Make update interval configurable here:
const UPDATE_INTERVAL_MS = 100;

const HEIGHT = 64;

export function MiniGraph({ newData, width }: MiniGraphProps) {
    const divRef = useRef<HTMLDivElement | null>(null);
    const uplotRef = useRef<uPlot | null>(null);
    const dataRef = useRef<[number[], number[]]>([[], []]); // [timestamps, values]

    // Store latest newData.current, to access inside interval
    const latestDataRef = useRef(newData?.current);

    useEffect(() => {
        latestDataRef.current = newData?.current;
    }, [newData?.current]);

    useEffect(() => {
        if (!divRef.current || !newData?.short?.timeWindow) return;

        const timeWindow = newData.short.timeWindow;

        // Use the configurable interval here
        const intervalId = setInterval(() => {
            const cur = latestDataRef.current;
            if (!cur || cur.timestamp <= 0) return;

            const [timestamps, values] = dataRef.current;

            // Add new data point at each tick
            timestamps.push(cur.timestamp);
            values.push(cur.value);

            // Remove old points outside time window
            const cutoff = cur.timestamp - timeWindow;
            while (timestamps.length && timestamps[0] < cutoff) {
                timestamps.shift();
                values.shift();
            }

            // Update graph scales
            const minY = Math.min(...values);
            const maxY = Math.max(...values);
            const range = maxY - minY || 1;

            const uData: uPlot.AlignedData = [timestamps, values];

            const opts: uPlot.Options = {
                width,
                height: HEIGHT,
                padding: [5, 50, 5, 0],

                cursor: { show: false },
                legend: { show: false },

                scales: {
                    x: {
                        time: true,
                        min: cutoff,
                        max: cur.timestamp,
                    },
                    y: {
                        auto: false,
                        min: minY - range * 0.1,
                        max: maxY + range * 0.1,
                    },
                },

                axes: [
                    { show: false },
                    {
                        side: 1,
                        grid: { stroke: "#ccc", width: 0.5 },
                        ticks: { stroke: "#ccc", width: 0.5 },
                        values: (u, ticks) => ticks.map((v) => v.toFixed(1)),
                    },
                ],

                series: [
                    {},
                    {
                        stroke: "black",
                        width: 2,
                        spanGaps: true,
                    },
                ],
            };

            if (!uplotRef.current) {
                if (divRef.current) {
                    uplotRef.current = new uPlot(opts, uData, divRef.current);
                }
            } else {
                uplotRef.current.setData(uData);
                uplotRef.current.setScale("x", { min: cutoff, max: cur.timestamp });
                uplotRef.current.setScale("y", {
                    min: minY - range * 0.1,
                    max: maxY + range * 0.1,
                });
            }
        }, UPDATE_INTERVAL_MS);

        return () => clearInterval(intervalId);
    }, [width, newData?.short?.timeWindow]);

    return (
        <div
            ref={divRef}
            style={{
                width: "100%",
                height: HEIGHT,
                overflow: "hidden",
            }}
        />
    );
}
