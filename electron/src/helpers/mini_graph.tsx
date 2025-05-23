import React, { useEffect, useRef } from "react";
import * as d3 from "d3";
import { TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width?: number;
};

const WIDTH = 250;
const HEIGHT = 100;
const MARGIN = { top: 20, right: 30, bottom: 30, left: 40 };

export function MiniGraph({ newData, width }: MiniGraphProps) {
  const svgRef = useRef<SVGSVGElement | null>(null);

  useEffect(() => {
    if (!newData || !newData.short.values.length) return;

    const short = newData.short.values;
    const short_series = newData.short;

    const index = short_series.index;
    const size = short_series.size;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const actualWidth = (width || WIDTH) - MARGIN.left - MARGIN.right;
    const height = HEIGHT - MARGIN.top - MARGIN.bottom;

    const newestTimestamp = short_series.lastTimestamp || Date.now();
    const actualStart = newestTimestamp - short_series.timeWindow;

    let min = Infinity,
      max = -Infinity;
    let found = false;

    for (let i = 0; i < size; i++) {
      const idx = (index + i) % size;
      const p = short[idx];
      if (
        p.timestamp > 0 &&
        p.timestamp >= actualStart &&
        p.timestamp <= newestTimestamp
      ) {
        min = Math.min(min, p.value);
        max = Math.max(max, p.value);
        found = true;
      }
    }
    if (!found) return;

    const y = d3
      .scaleLinear()
      .domain([min * 0.9, max * 1.1])
      .range([height, 0]);

    const x = d3
      .scaleLinear()
      .domain([actualStart, newestTimestamp])
      .range([0, actualWidth]);

    const g = svg
      .append("g")
      .attr("transform", `translate(${MARGIN.left},${MARGIN.top})`);

    g.append("g")
      .call(d3.axisLeft(y).ticks(5).tickSize(6).tickPadding(6))
      .call((g) => g.select(".domain").remove())
      .call((g) =>
        g.selectAll("line").style("stroke", "#ccc").style("stroke-width", 0.5),
      );

    const lineGen = d3.line<[number, number]>().curve(d3.curveMonotoneX);

    const path: [number, number][] = [];

    let anchored = false;

    for (let i = 0; i < size; i++) {
      const idx = (index + i) % size;
      const p = short[idx];
      if (
        p.timestamp > 0 &&
        p.timestamp >= actualStart &&
        p.timestamp <= newestTimestamp
      ) {
        if (!anchored && p.timestamp > actualStart) {
          path.push([x(actualStart), y(p.value)]);
          anchored = true;
        }
        path.push([x(p.timestamp), y(p.value)]);
      }
    }

    g.append("path")
      .attr("fill", "none")
      .attr("stroke", "black")
      .attr("stroke-width", 2)
      .attr("d", lineGen(path)!);
  }, [newData]);

  return <svg ref={svgRef} width={WIDTH} height={HEIGHT} />;
}
