import * as XLSX from "xlsx";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig, SeriesData, GraphLine } from "./types";

export type GraphExportData = {
  config: GraphConfig;
  data: SeriesData; // Always a single series
  unit?: Unit;
  renderValue?: (value: number) => string;
};

export function exportGraphsToExcel(
  graphDataMap: Map<string, () => GraphExportData | null>,
  groupId: string,
): void {
  try {
    // Filter out invalid series IDs (those without "-series-")
    const filteredMap = new Map<string, () => GraphExportData | null>();
    graphDataMap.forEach((getDataFn, seriesId) => {
      if (seriesId.includes("-series-")) {
        filteredMap.set(seriesId, getDataFn);
      }
    });

    const workbook = XLSX.utils.book_new();
    const exportTimestamp = new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, 19);

    const usedSheetNames = new Set<string>(); // Track unique sheet names
    let processedCount = 0;

    // Process each valid series
    filteredMap.forEach((getDataFn, seriesId) => {
      const exportData = getDataFn();
      if (!exportData?.data?.newData) {
        console.warn(`No data for series: ${seriesId}`);
        return;
      }

      const series = exportData.data;
      const seriesTitle = series.title || `Series ${processedCount + 1}`;

      if (!series.newData) {
        console.warn(`Series ${seriesTitle} has null data`);
        return;
      }

      const targetLines: GraphLine[] = [
        ...(exportData.config.lines || []),
        ...(series.lines || []),
      ];

      const graphLineData = {
        graphTitle: exportData.config.title,
        lineTitle: seriesTitle,
        series: series.newData,
        color: series.color,
        unit: exportData.unit,
        renderValue: exportData.renderValue,
        config: exportData.config,
        targetLines: targetLines,
      };

      // Create and append statistics sheet
      const statsData = createGraphLineStatsSheet(graphLineData);
      const statsWorksheet = XLSX.utils.aoa_to_sheet(statsData);
      const statsSheetName = generateUniqueSheetName(
        `${seriesTitle} Stats`,
        usedSheetNames,
      );
      XLSX.utils.book_append_sheet(workbook, statsWorksheet, statsSheetName);

      // Create and append data sheet
      const dataRows = createGraphLineDataSheet(graphLineData);
      if (dataRows.length > 0) {
        const dataWorksheet = XLSX.utils.json_to_sheet(dataRows);
        const dataSheetName = generateUniqueSheetName(
          `${seriesTitle} Data`,
          usedSheetNames,
        );
        XLSX.utils.book_append_sheet(workbook, dataWorksheet, dataSheetName);
      }

      // Excel worksheet for timestamps and timestamp markers
      if (graphLineData.targetLines.length > 0) {
        const markerReportData = createGraphLineMarkerReportSheet(graphLineData);
        const markerReportWorksheet = XLSX.utils.aoa_to_sheet(markerReportData);
        // Set column widths here (e.g., Column A = 15, Column B = 25)
        markerReportWorksheet["!cols"] = [
            { wch: 20 }, // Column A (Labels: 'Timestamp', 'Value', etc.)
            { wch: 30 }, // Column B (Values, where the Date object resides)
        ];
        const markerReportSheetName = generateUniqueSheetName(
          `${seriesTitle} Marker Report`,
          usedSheetNames,
        );
        XLSX.utils.book_append_sheet(
          workbook,
          markerReportWorksheet,
          markerReportSheetName,
        );
      }

      processedCount++;
    });

    if (processedCount === 0) {
      alert("No data available to export from any graphs in this group");
      return;
    }

    const filename = `${groupId.toLowerCase().replace(/\s+/g, "_")}_export_${exportTimestamp}.xlsx`;
    XLSX.writeFile(workbook, filename);
  } catch (error) {
    alert(
      `Error exporting data to Excel: ${error instanceof Error ? error.message : "Unknown error"}. Please try again.`,
    );
  }
}

// Generate statistics sheet for a graph line
function createGraphLineStatsSheet(graphLine: {
  graphTitle: string;
  lineTitle: string;
  series: TimeSeries;
  color?: string;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  targetLines: GraphLine[];
}): any[][] {
  const [timestamps, values] = seriesToUPlotData(graphLine.series.long);
  const unitSymbol = renderUnitSymbol(graphLine.unit) || "";

  const statsData = [
    [`Graph Line Statistics: ${graphLine.lineTitle}`, ""],
    ["Graph", graphLine.graphTitle],
    ["Line Name", graphLine.lineTitle],
    ["Line Color", graphLine.color || "Default"],
    ["Generated", new Date()],
    ["", ""],
    ["Data Points Information", ""],
    ["Total Data Points", timestamps.length.toString()],
  ];

  if (timestamps.length > 0) {
    statsData.push(["Time Range Start", new Date(timestamps[0])]);
    statsData.push([
      "Time Range End",
      new Date(timestamps[timestamps.length - 1]),
    ]);

    const duration = timestamps[timestamps.length - 1] - timestamps[0];
    const durationHours = (duration / (1000 * 60 * 60)).toFixed(2);
    statsData.push(["Duration (hours)", durationHours]);

    if (values.length > 0) {
      const minValue = Math.min(...values);
      const maxValue = Math.max(...values);
      const avgValue = values.reduce((a, b) => a + b, 0) / values.length;
      const stdDev = Math.sqrt(
        values.reduce((sum, val) => sum + Math.pow(val - avgValue, 2), 0) /
          values.length,
      );

      statsData.push(["", ""], ["Value Statistics", ""]);
      statsData.push([
        `Minimum Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(minValue)
          : minValue.toFixed(3),
      ]);
      statsData.push([
        `Maximum Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(maxValue)
          : maxValue.toFixed(3),
      ]);
      statsData.push([
        `Average Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(avgValue)
          : avgValue.toFixed(3),
      ]);
      statsData.push([
        `Standard Deviation (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(stdDev)
          : stdDev.toFixed(3),
      ]);
      statsData.push([
        `Range (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(maxValue - minValue)
          : (maxValue - minValue).toFixed(3),
      ]);

      // Percentiles
      const sortedValues = [...values].sort((a, b) => a - b);
      const p25 = sortedValues[Math.floor(sortedValues.length * 0.25)];
      const p50 = sortedValues[Math.floor(sortedValues.length * 0.5)];
      const p75 = sortedValues[Math.floor(sortedValues.length * 0.75)];

      statsData.push(["", ""], ["Percentiles", ""]);
      statsData.push([
        `25th Percentile (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p25) : p25.toFixed(3),
      ]);
      statsData.push([
        `50th Percentile/Median (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p50) : p50.toFixed(3),
      ]);
      statsData.push([
        `75th Percentile (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p75) : p75.toFixed(3),
      ]);
    }
  }

  // Add target line information
  if (graphLine.targetLines.length > 0) {
    statsData.push(["", ""], ["Target Lines", ""]);
    graphLine.targetLines.forEach((line, index) => {
      statsData.push([
        `Target Line ${index + 1}`,
        line.label || `Line ${line.value}`,
      ]);
      statsData.push([
        `  Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(line.value)
          : line.value.toFixed(3),
      ]);
      statsData.push([`  Type`, line.type || "reference"]);
      statsData.push([`  Color`, line.color || "default"]);
      statsData.push([`  Show`, line.show !== false ? "Yes" : "No"]);

      if (line.type === "threshold" && values.length > 0) {
        const withinThreshold = values.filter(
          (val) => Math.abs(val - line.value) <= line.value * 0.05,
        ).length;
        const percentageWithin = (
          (withinThreshold / values.length) *
          100
        ).toFixed(1);

        statsData.push([
          `  Points Within Threshold (5%)`,
          `${withinThreshold} (${percentageWithin}%)`,
        ]);

        const differences = values.map((val) => Math.abs(val - line.value));
        const minDifference = Math.min(...differences);
        const maxDifference = Math.max(...differences);

        statsData.push([
          `  Closest Approach (${unitSymbol})`,
          graphLine.renderValue
            ? graphLine.renderValue(minDifference)
            : minDifference.toFixed(3),
        ]);
        statsData.push([
          `  Furthest Distance (${unitSymbol})`,
          graphLine.renderValue
            ? graphLine.renderValue(maxDifference)
            : maxDifference.toFixed(3),
        ]);
      }

      if (index < graphLine.targetLines.length - 1) {
        statsData.push([""]);
      }
    });
  } else {
    statsData.push(["", ""], ["Target Lines", ""]);
    statsData.push(["No target lines defined", ""]);
  }

  return statsData;
}

// Generate data sheet for a graph line
function createGraphLineDataSheet(graphLine: {
  graphTitle: string;
  lineTitle: string;
  series: TimeSeries;
  color?: string;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  targetLines: GraphLine[];
}): any[] {
  const [timestamps, values] = seriesToUPlotData(graphLine.series.long);

  if (timestamps.length === 0) return [];

  const unitSymbol = renderUnitSymbol(graphLine.unit) || "";

  return timestamps.map((timestamp, index) => {
    const value = values[index];
    return {
      Timestamp: new Date(timestamp),
      [`Value (${unitSymbol})`]: graphLine.renderValue
        ? graphLine.renderValue(value)
        : value?.toFixed(3) || "",
    };
  });
}

function createGraphLineMarkerReportSheet(graphLine: {
  graphTitle: string;
  lineTitle: string;
  series: TimeSeries;
  color?: string;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  targetLines: GraphLine[];
}): any[][] {
  const [timestamps, values] = seriesToUPlotData(graphLine.series.long);
  const unitSymbol = renderUnitSymbol(graphLine.unit) || "";
  // Initialize Report Data and Header
  const reportData: any[][] = [
    [`Marker Report: ${graphLine.lineTitle}`],
    ["Graph", graphLine.graphTitle],
    ["Line Name", graphLine.lineTitle],
    ["", ""],
    ["--- Data Point Marker Status ---", ""],
    ["", ""],
  ];

  if (timestamps.length === 0) {
    reportData.push(["No data points to report"]);
    return reportData;
  }

  // Filter User Markers
  const allTargetLines = graphLine.targetLines.filter(line => line.show !== false);
  const userMarkers = allTargetLines.filter(line => line.type === 'user_marker' && line.label);

  // Map Markers to Closest Data Point Index
  const markerIndexMap = new Map<number, { label: string; originalTimestamp: number }>();

  userMarkers.forEach(line => {
    const markerTime = line.markerTimestamp || line.value; // Use the correct high-precision timestamp
    let closestDataPointIndex = -1;
    let minTimeDifference = Infinity;

    // Find the data point with the closest timestamp
    timestamps.forEach((ts, index) => {
        const difference = Math.abs(ts - markerTime);
        if (difference < minTimeDifference) {
            minTimeDifference = difference;
            closestDataPointIndex = index;
        }
    });
    
    // Store the marker data at the index of the closest data point
    if (closestDataPointIndex !== -1) { 
        markerIndexMap.set(closestDataPointIndex, { 
            label: line.label || 'User Marker',
            originalTimestamp: markerTime, 
        });
    }
  });

  // Add the final header before the timestamp report starts
  reportData.push(
      ["--- BEGIN DETAILED REPORT ---", ""],
      ["", ""],
  );

  // Handle case where no user markers were created
  if (userMarkers.length === 0) {
      reportData.push(["No user-created markers found.", ""]);
  }
  
  timestamps.forEach((dataPointTimestamp, index) => {
    const value = values[index];
    const markerData = markerIndexMap.get(index);

    let finalMarkerLabel = "";
    let timeToDisplay = dataPointTimestamp; // Default to data sample time

    if (markerData) {
        finalMarkerLabel = `${markerData.label}`;
        timeToDisplay = markerData.originalTimestamp; 
    }

    // Format the time (using timeToDisplay)
    const formattedTime = new Date(timeToDisplay).toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    }).replace(/ /g, '');

    // Row 1: Timestamp
    reportData.push(["Timestamp", formattedTime]);
    
    // Row 2: Value
    const formattedValue = graphLine.renderValue
        ? graphLine.renderValue(value)
        : value?.toFixed(3) || "";
    reportData.push([`Value (${unitSymbol})`, formattedValue]);
    
    // Row 3: Marker Name
    reportData.push(["Marker", finalMarkerLabel]);
    
    // Separator
    reportData.push(["", ""]);
  });
  
  return reportData;
}

// Ensure sheet names are unique and valid for Excel
function generateUniqueSheetName(
  name: string,
  usedSheetNames: Set<string>,
): string {
  let baseSheetName = name
    .replace(/[\\/?*$:[\]]/g, "_") // Remove invalid characters
    .substring(0, 31); // Excel sheet name limit

  if (!baseSheetName || baseSheetName.trim().length === 0) {
    baseSheetName = "Sheet";
  }

  let sheetName = baseSheetName;
  let counter = 1;

  while (usedSheetNames.has(sheetName)) {
    const suffix = `_${counter}`;
    const maxBaseLength = 31 - suffix.length;
    sheetName = `${baseSheetName.substring(0, maxBaseLength)}${suffix}`;
    counter++;
  }

  usedSheetNames.add(sheetName);
  return sheetName;
}
