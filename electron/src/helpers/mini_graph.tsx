import React, { useEffect, useRef, useState } from "react";
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
    const dataRef = useRef<[number, number][]>([]);
    const [, setMinMax] = useState({ min: Infinity, max: -Infinity });
    useEffect(() => {
        if (!newData?.current || !svgRef.current || !newData.short.values.length) return;

        const cur = newData.current;
        if (cur.timestamp <= 0) return;

        const { timeWindow, size } = newData.short;
        const points = dataRef.current;

        const estimatedInterval = timeWindow / size;
        const maxPoints = Math.floor(timeWindow / estimatedInterval);

        // Push new point and maintain time-based sliding window
        points.push([cur.timestamp, cur.value]);
        const startTime = cur.timestamp - timeWindow;
        while (points.length > 0 && points[0][0] < startTime) {
            points.shift();
        }

        // Min/max update
        const values = points.map(p => p[1]);
        const newMin = Math.min(...values);
        const newMax = Math.max(...values);
        setMinMax({ min: newMin, max: newMax });

        // D3 drawing
        const svg = d3.select(svgRef.current);
        const graphWidth = width - MARGIN.left - MARGIN.right;
        const graphHeight = HEIGHT - MARGIN.top - MARGIN.bottom;

        const x = d3.scaleLinear()
            .domain([cur.timestamp - timeWindow, cur.timestamp])
            .range([0, graphWidth])
            .clamp(true);

        const y = d3.scaleLinear()
            .domain([newMin * 0.9, newMax * 1.1])
            .range([graphHeight, 0]);

        let g = svg.select<SVGGElement>("g.graph-group");
        if (g.empty()) {
            g = svg.append("g")
                .attr("class", "graph-group")
                .attr("transform", `translate(${MARGIN.left},${MARGIN.top})`);
        }

        let yAxisGroup = g.select<SVGGElement>("g.y-axis");
        if (yAxisGroup.empty()) {
            yAxisGroup = g.append("g").attr("class", "y-axis");
        }
        yAxisGroup
            .attr("transform", `translate(${graphWidth}, 0)`)
            .call(d3.axisRight(y).ticks(3).tickSize(4))
            .call(g => g.select(".domain").remove())
            .call(g => g.selectAll("line").style("stroke", "#ccc").style("stroke-width", 0.5));

        const lineGen = d3.line<[number, number]>()
            .x(d => x(d[0]))
            .y(d => y(d[1]))
            .curve(d3.curveLinear);

        let path = g.select<SVGPathElement>("path.line");
        if (path.empty()) {
            path = g.append("path")
                .attr("class", "line")
                .attr("fill", "none")
                .attr("stroke", "black")
                .attr("stroke-width", 2);
        }

        path.attr("d", lineGen(points)!);
    }, [newData?.current, width]);


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
