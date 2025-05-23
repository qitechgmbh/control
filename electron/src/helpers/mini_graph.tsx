import React, { useEffect, useRef } from "react";
import * as d3 from "d3";
import { TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
    newData: TimeSeries | null;
    width: number;
};

const HEIGHT = 56;
const MARGIN = { top: 5, right: 35, bottom: 5, left: 0 };
export function MiniGraph({ newData, width }: MiniGraphProps) {
    const svgRef = useRef<SVGSVGElement | null>(null);

    useEffect(() => {
        if (!newData || !newData.short.values.length) return;

        const { values, index, size, lastTimestamp, timeWindow } = newData.short;

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        const graphWidth = width - MARGIN.left - MARGIN.right;
        const graphHeight = HEIGHT - MARGIN.top - MARGIN.bottom;


        const path: [number, number][] = [];

        let minValue = Infinity;
        let maxValue = -Infinity;
        let oldestTime = Infinity;
        let newestTime = lastTimestamp;

        for (let i = 0; i < size; i++) {
            const idx = (index + i) % size;
            const cur = values[idx];
            if (cur == null) break;
            if (cur.timestamp > 0 && cur.timestamp < lastTimestamp) {
                path.push([cur.timestamp, cur.value]);

                minValue = Math.min(minValue, cur.value);
                maxValue = Math.max(maxValue, cur.value);
                oldestTime = Math.min(oldestTime, cur.timestamp);
            }
        }
        if (path.length === 0) return; // no valid data, do not draw anything

        const xDomainStart = newestTime - timeWindow + 1000;
        const xDomainEnd = newestTime;

        const x = d3.scaleLinear()
            .domain([xDomainStart, xDomainEnd])
            .range([0, graphWidth])
            .clamp(true);
        const y = d3.scaleLinear()
            .domain([minValue * 0.9, maxValue * 1.1])
            .range([graphHeight, 0]);

        const g = svg.append("g")
            .attr("transform", `translate(${MARGIN.left},${MARGIN.top})`);

        // Y Axis on the right side:
        g.append("g")
            .attr("transform", `translate(${graphWidth}, 0)`)
            .call(d3.axisRight(y).ticks(3).tickSize(4))
            .call(g => g.select(".domain").remove())
            .call(g => g.selectAll("line").style("stroke", "#ccc").style("stroke-width", 0.5));

        // <-- X Axis block removed here -->

        // Draw the line only with existing points
        const lineGen = d3.line<[number, number]>()
            .x(d => x(d[0]))
            .y(d => y(d[1]))
            .curve(d3.curveLinear);

        g.append("path")
            .attr("fill", "none")
            .attr("stroke", "black")
            .attr("stroke-width", 2)
            .attr("d", lineGen(path)!);

    }, [newData, width]);


    return (
        <svg
            ref={svgRef}
            width="100%"
            height={HEIGHT}
            viewBox={`0 0 ${width ?? 250} ${HEIGHT}`}
            preserveAspectRatio="none"
            style={{ display: "block" }}
        />
    );
}
