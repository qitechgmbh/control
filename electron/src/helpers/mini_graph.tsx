import React, { useEffect, useRef } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { getSeriesMinMax, seriesToUPlotData, TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
    newData: TimeSeries | null;
    width: number;
};


const HEIGHT = 64;

export function MiniGraph({ newData, width }: MiniGraphProps) {
    const divRef = useRef<HTMLDivElement | null>(null);
    const uplotRef = useRef<uPlot | null>(null);
    const latestDataRef = useRef(newData?.current);
    const UPDATE_INTERVAL_MS = newData?.short.sampleInterval;

    useEffect(() => {
        latestDataRef.current = newData?.current;
    }, [newData?.current]);

    useEffect(() => {
        if (!divRef.current || !newData?.short?.timeWindow) return;

        const short = newData.short;
        const timeWindow = short.timeWindow;

        // Extract ALL data first, then filter by time window in the scale
        const [allTimestamps, allValues] = seriesToUPlotData(short); // No time window filter

        if (allTimestamps.length === 0) return;

        // Get min/max in O(1)
        const { min: minY, max: maxY } = getSeriesMinMax(short);
        const range = maxY - minY || 1;

        // Use the latest timestamp as the end point
        const latestTimestamp = allTimestamps[allTimestamps.length - 1];
        const cutoff = latestTimestamp - timeWindow;

        const uData: uPlot.AlignedData = [allTimestamps, allValues];

        const opts: uPlot.Options = {
            width,
            height: HEIGHT,
            padding: [4, 0, 4, 0],
            cursor: { show: false },
            legend: { show: false },
            scales: {
                x: {
                    time: true,
                    min: cutoff,
                    max: latestTimestamp,
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

        uplotRef.current = new uPlot(opts, uData, divRef.current);

        const intervalId = setInterval(() => {
            const cur = latestDataRef.current;
            if (!cur || cur.timestamp <= 0 || !newData?.short) return;

            // Get ALL data, let uPlot handle the time window via scales
            const [timestamps, values] = seriesToUPlotData(newData.short); // No time window filter

            if (timestamps.length === 0) return;

            // Get min/max in O(1)
            const { min: minY, max: maxY } = getSeriesMinMax(newData.short);
            const range = maxY - minY || 1;

            // Use current timestamp as the end point
            const latestTimestamp = timestamps[timestamps.length - 1];
            const cutoff = latestTimestamp - timeWindow;

            uplotRef.current?.setData([timestamps, values]);
            uplotRef.current?.setScale("x", { min: cutoff, max: latestTimestamp });
            uplotRef.current?.setScale("y", {
                min: minY - range * 0.1,
                max: maxY + range * 0.1,
            });
        }, UPDATE_INTERVAL_MS);

        return () => {
            clearInterval(intervalId);
            uplotRef.current?.destroy();
            uplotRef.current = null;
        };
    }, [width, newData?.short?.timeWindow, newData]);

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
