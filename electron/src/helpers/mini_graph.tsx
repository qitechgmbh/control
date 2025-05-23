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
        if (!newData || !newData.short.path.length) return;

        const {
            path,
            lastTimestamp,
            timeWindow,
            minDeque,
            maxDeque,
            values,
        } = newData.short;

        const svg = d3.select(svgRef.current);
        svg.selectAll("*").remove();

        const graphWidth = width - MARGIN.left - MARGIN.right;
        const graphHeight = HEIGHT - MARGIN.top - MARGIN.bottom;

        const newestTime = lastTimestamp;
        const xDomainStart = newestTime - timeWindow + 1000;
        const xDomainEnd = newestTime;

        // Get min/max values using deques
        const minIdx = minDeque?.[0];
        const maxIdx = maxDeque?.[0];
        const minValue = minIdx !== undefined ? values[minIdx]?.value ?? 0 : 0;
        const maxValue = maxIdx !== undefined ? values[maxIdx]?.value ?? 1 : 1;

        const x = d3.scaleLinear()
            .domain([xDomainStart, xDomainEnd])
            .range([0, graphWidth])
            .clamp(true);

        const y = d3.scaleLinear()
            .domain([minValue * 0.9, maxValue * 1.1])
            .range([graphHeight, 0]);

        const g = svg.append("g")
            .attr("transform", `translate(${MARGIN.left},${MARGIN.top})`);

        g.append("g")
            .attr("transform", `translate(${graphWidth}, 0)`)
            .call(d3.axisRight(y).ticks(3).tickSize(4))
            .call(g => g.select(".domain").remove())
            .call(g => g.selectAll("line").style("stroke", "#ccc").style("stroke-width", 0.5));

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
