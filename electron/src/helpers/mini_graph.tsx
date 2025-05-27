import React, { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
    newData: TimeSeries | null;
    width: number;
};

const HEIGHT = 64;

export function MiniGraph({ newData, width }: MiniGraphProps) {
    const uplotRef = useRef<uPlot | null>(null);
    const divRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (!newData?.short || !divRef.current) return;

        const { values, index, size, timeWindow, lastTimestamp } = newData.short;
        const timestamps: number[] = [];
        const data: number[] = [];

        for (let i = 0; i < size; i++) {
            const idx = (index + i) % size;
            const point = values[idx];
            if (point && point.timestamp > 0 && point.timestamp >= lastTimestamp - timeWindow) {
                timestamps.push(point.timestamp);
                data.push(point.value);
            }
        }

        if (timestamps.length === 0 || data.length === 0) return;

        const minY = Math.min(...data);
        const maxY = Math.max(...data);
        const range = maxY - minY || 1;

        const uData: uPlot.AlignedData = [timestamps, data];

        const opts: uPlot.Options = {
            width,
            height: HEIGHT,

            padding: [5, 40, 5, 0],

            cursor: { show: false },
            legend: { show: false },

            scales: {
                x: {
                    time: false,
                    min: timestamps[0],
                    max: timestamps[timestamps.length - 1],
                },
                y: {
                    auto: false,
                    min: minY - range * 0.1,
                    max: maxY + range * 0.1,
                },
            },

            axes: [
                { show: false }, // hide x-axis
                {
                    side: 1,
                    size: 0, // remove y-axis width so no margin on right side
                    grid: { stroke: "#ccc", width: 0.5 },
                    ticks: { stroke: "#ccc", width: 0.5 },
                    values: (u, ticks) => ticks.map(v => v.toFixed(1)),
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

        if (uplotRef.current) {
            uplotRef.current.destroy();
            uplotRef.current = null;
        }

        uplotRef.current = new uPlot(opts, uData, divRef.current);

        return () => {
            uplotRef.current?.destroy();
            uplotRef.current = null;
        };
    }, [newData?.short, width]);

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
