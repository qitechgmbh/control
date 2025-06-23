import * as XLSX from "xlsx";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig, BigGraphProps, SeriesData } from "./types";

export type GraphExportData = {
  config: GraphConfig;
  data: BigGraphProps["newData"];
  unit?: Unit;
  renderValue?: (value: number) => string;
};

// Helper functions for multi-series support
function normalizeDataSeries(data: BigGraphProps["newData"]): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

function getAllValidSeries(data: BigGraphProps["newData"]): Array<{
  series: TimeSeries;
  title?: string;
  color?: string;
  index: number;
}> {
  const normalized = normalizeDataSeries(data);
  return normalized
    .filter((series) => series.newData !== null)
    .map((series, index) => ({
      series: series.newData!,
      title: series.title,
      color: series.color,
      index,
    }));
}

export function exportGraphsToExcel(
  graphDataMap: Map<string, () => GraphExportData | null>,
  groupId: string,
): void {
  try {
    const workbook = XLSX.utils.book_new();
    const exportTimestamp = new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, 19);

    // Keep track of used sheet names to ensure uniqueness
    const usedSheetNames = new Set<string>();

    let totalSeriesCount = 0;
    let hasValidData = false;
    const allSeriesData: Array<{
      graphTitle: string;
      seriesTitle: string;
      series: TimeSeries;
      unit?: Unit;
      renderValue?: (value: number) => string;
    }> = [];

    // First pass: collect all series data and count
    graphDataMap.forEach((getDataFn) => {
      const exportData = getDataFn();
      if (exportData?.data) {
        const validSeries = getAllValidSeries(exportData.data);
        totalSeriesCount += validSeries.length;
        if (validSeries.length > 0) {
          hasValidData = true;
          validSeries.forEach((seriesInfo) => {
            allSeriesData.push({
              graphTitle: exportData.config.title,
              seriesTitle: seriesInfo.title || `Series ${seriesInfo.index + 1}`,
              series: seriesInfo.series,
              unit: exportData.unit,
              renderValue: exportData.renderValue,
            });
          });
        }
      }
    });

    if (!hasValidData) {
      alert("No data available to export from any graphs in this group");
      return;
    }

    // 1. Create summary sheet
    const summaryData = createSummarySheet(graphDataMap, totalSeriesCount);
    const summaryWorksheet = XLSX.utils.aoa_to_sheet(summaryData);
    XLSX.utils.book_append_sheet(workbook, summaryWorksheet, "Summary");
    usedSheetNames.add("Summary");

    // 2. Create collected values sheet (all series combined)
    const collectedData = createCollectedValuesSheet(allSeriesData);
    const collectedWorksheet = XLSX.utils.json_to_sheet(collectedData);
    XLSX.utils.book_append_sheet(workbook, collectedWorksheet, "All Values");
    usedSheetNames.add("All Values");

    // 3. Export individual series (data + stats sheets for each)
    exportSeriesWithStats(workbook, graphDataMap, usedSheetNames);

    const filename = `${groupId.toLowerCase().replace(/\s+/g, "_")}_export_${exportTimestamp}.xlsx`;
    XLSX.writeFile(workbook, filename);
  } catch (error) {
    alert(
      `Error exporting data to Excel: ${error instanceof Error ? error.message : "Unknown error"}. Please try again.`,
    );
  }
}

function createSummarySheet(
  graphDataMap: Map<string, () => GraphExportData | null>,
  totalSeriesCount: number,
): any[][] {
  const summaryData = [
    [`Export Summary`, ""],
    ["Export Date", new Date()],
    ["Total Graphs", graphDataMap.size.toString()],
    ["Total Series", totalSeriesCount.toString()],
    ["", ""],
    ["Graphs and Series:", ""],
  ];

  graphDataMap.forEach((getDataFn) => {
    const exportData = getDataFn();
    if (exportData?.data) {
      const validSeries = getAllValidSeries(exportData.data);
      if (validSeries.length > 0) {
        summaryData.push([
          `Graph: ${exportData.config.title}`,
          `${validSeries.length} series`,
        ]);
        validSeries.forEach((seriesInfo) => {
          const seriesTitle =
            seriesInfo.title || `Series ${seriesInfo.index + 1}`;
          const [timestamps] = seriesToUPlotData(seriesInfo.series.long);
          summaryData.push([
            `  - ${seriesTitle}`,
            `${timestamps.length} data points`,
          ]);
        });
        summaryData.push(["", ""]);
      }
    }
  });

  summaryData.push(["", ""], ["Sheet Structure:", ""]);
  summaryData.push(["Summary", "This overview sheet"]);
  summaryData.push(["All Values", "Combined data from all series"]);
  summaryData.push(["[Series Name]", "Individual series data"]);
  summaryData.push(["[Series Name] Stats", "Statistics for each series"]);

  return summaryData;
}

function createCollectedValuesSheet(
  allSeriesData: Array<{
    graphTitle: string;
    seriesTitle: string;
    series: TimeSeries;
    unit?: Unit;
    renderValue?: (value: number) => string;
  }>,
): any[] {
  if (allSeriesData.length === 0) return [];

  // Get all unique timestamps from all series
  const allTimestampsSet = new Set<number>();
  const seriesDataMap = new Map<string, Map<number, number>>();

  allSeriesData.forEach((seriesData) => {
    const [timestamps, values] = seriesToUPlotData(seriesData.series.long);
    const valueMap = new Map<number, number>();

    timestamps.forEach((timestamp, index) => {
      allTimestampsSet.add(timestamp);
      valueMap.set(timestamp, values[index]);
    });

    const seriesKey = `${seriesData.graphTitle} - ${seriesData.seriesTitle}`;
    seriesDataMap.set(seriesKey, valueMap);
  });

  // Sort all timestamps
  const allTimestamps = Array.from(allTimestampsSet).sort((a, b) => a - b);

  // Create rows with all series data
  return allTimestamps.map((timestamp) => {
    const row: any = {
      Timestamp: new Date(timestamp),
    };

    // Add data for each series
    allSeriesData.forEach((seriesData) => {
      const seriesKey = `${seriesData.graphTitle} - ${seriesData.seriesTitle}`;
      const valueMap = seriesDataMap.get(seriesKey);
      const value = valueMap?.get(timestamp);

      const unitSymbol = renderUnitSymbol(seriesData.unit) || "";
      const columnName = `${seriesKey} (${unitSymbol})`;

      if (value !== undefined) {
        row[columnName] = seriesData.renderValue
          ? seriesData.renderValue(value)
          : value?.toFixed(3) || "";
      } else {
        row[columnName] = ""; // Empty cell for missing data points
      }
    });

    return row;
  });
}

function exportSeriesWithStats(
  workbook: XLSX.WorkBook,
  graphDataMap: Map<string, () => GraphExportData | null>,
  usedSheetNames: Set<string>,
): void {
  graphDataMap.forEach((getDataFn) => {
    const exportData = getDataFn();
    if (!exportData?.data) return;

    const validSeries = getAllValidSeries(exportData.data);
    if (validSeries.length === 0) return;

    // Create data and stats sheets for each series
    validSeries.forEach((seriesInfo) => {
      const seriesTitle = seriesInfo.title || `Series ${seriesInfo.index + 1}`;

      // 1. Create data sheet for this series
      const seriesDataRows = createSingleSeriesDataRows(seriesInfo, exportData);
      if (seriesDataRows.length > 0) {
        const dataWorksheet = XLSX.utils.json_to_sheet(seriesDataRows);
        const dataSheetName = generateUniqueSheetName(
          seriesTitle,
          usedSheetNames,
        );
        XLSX.utils.book_append_sheet(workbook, dataWorksheet, dataSheetName);
      }

      // 2. Create stats sheet for this series
      const statsData = createSeriesStatsSheet(seriesInfo, exportData);
      const statsWorksheet = XLSX.utils.aoa_to_sheet(statsData);
      const statsSheetName = generateUniqueSheetName(
        `${seriesTitle} Stats`,
        usedSheetNames,
      );
      XLSX.utils.book_append_sheet(workbook, statsWorksheet, statsSheetName);
    });
  });
}

function createSingleSeriesDataRows(
  seriesInfo: {
    series: TimeSeries;
    title?: string;
    color?: string;
    index: number;
  },
  exportData: GraphExportData,
): any[] {
  const [timestamps, values] = seriesToUPlotData(seriesInfo.series.long);

  if (timestamps.length === 0) return [];

  const unitSymbol = renderUnitSymbol(exportData.unit) || "";

  return timestamps.map((timestamp, index) => {
    const value = values[index];
    const row: any = {
      Timestamp: new Date(timestamp),
      [`Value (${unitSymbol})`]: exportData.renderValue
        ? exportData.renderValue(value)
        : value?.toFixed(3) || "",
    };

    // Add config lines for reference
    exportData.config.lines?.forEach((line) => {
      const lineColumnName = `${line.label} (${unitSymbol})`;
      row[lineColumnName] = exportData.renderValue
        ? exportData.renderValue(line.value)
        : line.value.toFixed(3);

      // Add threshold analysis
      if (line.type === "threshold" && value !== undefined) {
        const withinThreshold =
          Math.abs(value - line.value) <= line.value * 0.05;
        row[`Within ${line.label}`] = withinThreshold ? "Yes" : "No";
        row[`Difference from ${line.label}`] = exportData.renderValue
          ? exportData.renderValue(value - line.value)
          : (value - line.value).toFixed(3);
      }
    });

    return row;
  });
}

function createSeriesStatsSheet(
  seriesInfo: {
    series: TimeSeries;
    title?: string;
    color?: string;
    index: number;
  },
  exportData: GraphExportData,
): any[][] {
  const seriesTitle = seriesInfo.title || `Series ${seriesInfo.index + 1}`;
  const [timestamps, values] = seriesToUPlotData(seriesInfo.series.long);
  const unitSymbol = renderUnitSymbol(exportData.unit) || "";

  const statsData = [
    [`Statistics: ${seriesTitle}`, ""],
    ["Generated", new Date()],
    ["", ""],
    ["Basic Information", ""],
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

      statsData.push(["", ""], ["Statistical Analysis", ""]);
      statsData.push([
        `Minimum Value (${unitSymbol})`,
        exportData.renderValue
          ? exportData.renderValue(minValue)
          : minValue.toFixed(3),
      ]);
      statsData.push([
        `Maximum Value (${unitSymbol})`,
        exportData.renderValue
          ? exportData.renderValue(maxValue)
          : maxValue.toFixed(3),
      ]);
      statsData.push([
        `Average Value (${unitSymbol})`,
        exportData.renderValue
          ? exportData.renderValue(avgValue)
          : avgValue.toFixed(3),
      ]);
      statsData.push([
        `Standard Deviation (${unitSymbol})`,
        exportData.renderValue
          ? exportData.renderValue(stdDev)
          : stdDev.toFixed(3),
      ]);
      statsData.push([
        `Range (${unitSymbol})`,
        exportData.renderValue
          ? exportData.renderValue(maxValue - minValue)
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
        exportData.renderValue ? exportData.renderValue(p25) : p25.toFixed(3),
      ]);
      statsData.push([
        `50th Percentile/Median (${unitSymbol})`,
        exportData.renderValue ? exportData.renderValue(p50) : p50.toFixed(3),
      ]);
      statsData.push([
        `75th Percentile (${unitSymbol})`,
        exportData.renderValue ? exportData.renderValue(p75) : p75.toFixed(3),
      ]);
    }
  }

  // Add config lines analysis
  if (exportData.config.lines && exportData.config.lines.length > 0) {
    statsData.push(["", ""], ["Threshold Analysis", ""]);
    exportData.config.lines.forEach((line) => {
      if (line.type === "threshold" && values.length > 0) {
        const withinThreshold = values.filter(
          (val) => Math.abs(val - line.value) <= line.value * 0.05,
        ).length;
        const percentageWithin = (
          (withinThreshold / values.length) *
          100
        ).toFixed(1);

        statsData.push([
          `${line.label} Threshold (${unitSymbol})`,
          exportData.renderValue
            ? exportData.renderValue(line.value)
            : line.value.toFixed(3),
        ]);
        statsData.push([
          `Points Within ${line.label}`,
          `${withinThreshold} (${percentageWithin}%)`,
        ]);
      }
    });
  }

  return statsData;
}

function generateUniqueSheetName(
  name: string,
  usedSheetNames: Set<string>,
): string {
  // Clean the name for use as sheet name
  let baseSheetName = name
    .replace(/[\\/?*$:[\]]/g, "_") // Remove invalid characters
    .substring(0, 31); // Excel sheet name limit

  // Ensure we have a valid name
  if (!baseSheetName || baseSheetName.trim().length === 0) {
    baseSheetName = "Sheet";
  }

  // Make it unique
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
