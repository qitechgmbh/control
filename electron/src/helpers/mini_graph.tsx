import React, { useEffect, useRef, useCallback } from "react";
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
    const lastUpdateTimestamp = useRef<number>(0);
    const isInitialized = useRef<boolean>(false);

    // Memoized update function to avoid recreating on every render
    const updateChart = useCallback(() => {
        if (!uplotRef.current || !newData?.short || !newData?.current) return;

        const cur = newData.current;

        // Only update if we have new data
        if (cur.timestamp <= lastUpdateTimestamp.current) return;

        lastUpdateTimestamp.current = cur.timestamp;

        const short = newData.short;
        const timeWindow = short.timeWindow;

        // Get data efficiently
        const [timestamps, values] = seriesToUPlotData(short);

        if (timestamps.length === 0) return;

        // Get min/max in O(1)
        const { min: minY, max: maxY } = getSeriesMinMax(short);
        const range = maxY - minY || 1;

        // Use current timestamp to ensure line reaches edge
        const cutoff = cur.timestamp - timeWindow + 1000;

        // Batch all updates together to minimize redraws
        uplotRef.current.batch(() => {
            uplotRef.current!.setData([timestamps, values]);
            uplotRef.current!.setScale("x", { min: cutoff, max: cur.timestamp });
            uplotRef.current!.setScale("y", {
                min: minY - range * 0.1,
                max: maxY + range * 0.1,
            });
        });
    }, [newData]);

    // Initialize chart only once
    useEffect(() => {
        if (!divRef.current || !newData?.short?.timeWindow || isInitialized.current) return;

        const short = newData.short;
        const timeWindow = short.timeWindow;

        // Extract initial data
        const [allTimestamps, allValues] = seriesToUPlotData(short);

        // Get min/max
        const { min: minY, max: maxY } = getSeriesMinMax(short);
        const range = maxY - minY || 1;

        // Use current time or latest timestamp
        const now = Date.now();
        const latestTimestamp = allTimestamps.length > 0 ? allTimestamps[allTimestamps.length - 1] : now;
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
        isInitialized.current = true;

        return () => {
            uplotRef.current?.destroy();
            uplotRef.current = null;
            isInitialized.current = false;
        };
    }, [width, newData?.short?.timeWindow]); // Only recreate on width/timeWindow changes

    // Update chart when new data arrives (event-driven, not polling)
    useEffect(() => {
        if (!isInitialized.current) return;
        updateChart();
    }, [newData?.current?.timestamp, updateChart]); // Only update when timestamp changes

    // Handle width changes without recreating the entire chart
    useEffect(() => {
        if (!uplotRef.current) return;
        uplotRef.current.setSize({ width, height: HEIGHT });
    }, [width]);

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
