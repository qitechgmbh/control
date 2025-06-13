import * as XLSX from "xlsx";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig } from "./BigGraph";

export type GraphExportData = {
  config: GraphConfig;
  data: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
};

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

    // Create group summary sheet
    const groupSummaryData = createGroupSummaryData(groupId, graphDataMap);

    const hasValidData = addGraphSummaryToWorkbook(
      groupSummaryData,
      graphDataMap,
    );

    if (!hasValidData) {
      alert("No data available to export from any graphs in this group");
      return;
    }

    const groupSummaryWorksheet = XLSX.utils.aoa_to_sheet(groupSummaryData);
    XLSX.utils.book_append_sheet(
      workbook,
      groupSummaryWorksheet,
      "Group Summary",
    );

    // Keep track of used sheet names to ensure uniqueness
    const usedSheetNames = new Set(["Group Summary"]);

    // Export each graph's data
    exportIndividualGraphs(workbook, graphDataMap, usedSheetNames);

    const filename = `${groupId.toLowerCase().replace(/\s+/g, "_")}_group_export_${exportTimestamp}.xlsx`;
    XLSX.writeFile(workbook, filename);
  } catch (error) {
    alert(
      `Error exporting group data to Excel: ${error instanceof Error ? error.message : "Unknown error"}. Please try again.`,
    );
  }
}

function createGroupSummaryData(
  groupId: string,
  graphDataMap: Map<string, () => GraphExportData | null>,
): any[][] {
  return [
    [`${groupId} Export Summary`, ""],
    ["Export Date", new Date()],
    ["Group ID", groupId],
    ["Total Graphs", graphDataMap.size.toString()],
    ["", ""],
    ["Graphs in this export:", ""],
  ];
}

function addGraphSummaryToWorkbook(
  groupSummaryData: any[][],
  graphDataMap: Map<string, () => GraphExportData | null>,
): boolean {
  let hasValidData = false;

  graphDataMap.forEach((getDataFn, graphId) => {
    const exportData = getDataFn();
    if (exportData?.data?.long) {
      groupSummaryData.push([exportData.config.title, graphId]);
      hasValidData = true;
    }
  });

  return hasValidData;
}

function exportIndividualGraphs(
  workbook: XLSX.WorkBook,
  graphDataMap: Map<string, () => GraphExportData | null>,
  usedSheetNames: Set<string>,
): void {
  graphDataMap.forEach((getDataFn, graphId) => {
    const exportData = getDataFn();
    if (!exportData?.data?.long) return;

    const [timestamps, values] = seriesToUPlotData(exportData.data.long);
    if (timestamps.length === 0) return;

    // Create data sheet for this graph
    const graphDataRows = createGraphDataRows(timestamps, values, exportData);
    const graphDataWorksheet = XLSX.utils.json_to_sheet(graphDataRows);

    // Create unique sheet names
    const { dataSheetName, summarySheetName } = generateUniqueSheetNames(
      exportData.config.title,
      graphId,
      usedSheetNames,
    );

    XLSX.utils.book_append_sheet(workbook, graphDataWorksheet, dataSheetName);

    // Create summary sheet for this graph
    const graphSummaryData = createGraphSummaryData(
      exportData,
      graphId,
      timestamps,
      values,
    );
    const graphSummaryWorksheet = XLSX.utils.aoa_to_sheet(graphSummaryData);
    XLSX.utils.book_append_sheet(
      workbook,
      graphSummaryWorksheet,
      summarySheetName,
    );
  });
}

function createGraphDataRows(
  timestamps: number[],
  values: number[],
  exportData: GraphExportData,
): any[] {
  return timestamps.map((timestamp, index) => {
    const row: any = {
      Timestamp: new Date(timestamp),
      [`Value (${renderUnitSymbol(exportData.unit)})`]: exportData.renderValue
        ? exportData.renderValue(values[index])
        : values[index]?.toFixed(3) || "",
    };

    exportData.config.lines?.forEach((line) => {
      row[`${line.label} (${renderUnitSymbol(exportData.unit)})`] =
        exportData.renderValue
          ? exportData.renderValue(line.value)
          : line.value.toFixed(3);

      if (line.type === "threshold") {
        row[`Within ${line.label}`] =
          Math.abs(values[index] - line.value) <= line.value * 0.05
            ? "Yes"
            : "No";
      }
    });

    return row;
  });
}

function generateUniqueSheetNames(
  title: string,
  graphId: string,
  usedSheetNames: Set<string>,
): { dataSheetName: string; summarySheetName: string } {
  const baseSheetName = title.replace(/[\\/?*$:]/g, "_").substring(0, 20);

  const safeGraphId = graphId.replace(/[\\/?*$:]/g, "_").substring(0, 10);

  let dataSheetName = `${baseSheetName}_${safeGraphId}_Data`;
  let summarySheetName = `${baseSheetName}_${safeGraphId}_Sum`;

  // Ensure sheet names are within Excel's 31 character limit
  if (dataSheetName.length > 31) {
    dataSheetName = `${baseSheetName.substring(0, 15)}_${safeGraphId}_Data`;
  }
  if (summarySheetName.length > 31) {
    summarySheetName = `${baseSheetName.substring(0, 15)}_${safeGraphId}_Sum`;
  }

  // Ensure uniqueness
  let counter = 1;
  const originalDataName = dataSheetName;
  const originalSummaryName = summarySheetName;

  while (usedSheetNames.has(dataSheetName)) {
    dataSheetName = `${originalDataName.substring(0, 28)}_${counter}`;
    counter++;
  }

  counter = 1;
  while (usedSheetNames.has(summarySheetName)) {
    summarySheetName = `${originalSummaryName.substring(0, 28)}_${counter}`;
    counter++;
  }

  usedSheetNames.add(dataSheetName);
  usedSheetNames.add(summarySheetName);

  return { dataSheetName, summarySheetName };
}

function createGraphSummaryData(
  exportData: GraphExportData,
  graphId: string,
  timestamps: number[],
  values: number[],
): any[][] {
  const graphSummaryData = [
    [`${exportData.config.title} Summary (${graphId})`, ""],
    ["Graph ID", graphId],
    ["Export Date", new Date()],
    ["Description", exportData.config.description || ""],
    ["", ""],
    ["Parameters", ""],
  ];

  exportData.config.lines?.forEach((line) => {
    graphSummaryData.push([
      `${line.label} (${renderUnitSymbol(exportData.unit)})`,
      exportData.renderValue
        ? exportData.renderValue(line.value)
        : line.value.toFixed(3),
    ]);
  });

  graphSummaryData.push(["", ""], ["Statistics", ""]);
  graphSummaryData.push(["Total Data Points", timestamps.length.toString()]);

  if (timestamps.length > 0) {
    graphSummaryData.push(["Time Range Start", new Date(timestamps[0])]);
    graphSummaryData.push([
      "Time Range End",
      new Date(timestamps[timestamps.length - 1]),
    ]);

    if (values.length > 0) {
      const minValue = Math.min(...values);
      const maxValue = Math.max(...values);
      const avgValue = values.reduce((a, b) => a + b, 0) / values.length;

      graphSummaryData.push([
        `Min Value (${renderUnitSymbol(exportData.unit)})`,
        exportData.renderValue
          ? exportData.renderValue(minValue)
          : minValue.toFixed(3),
      ]);
      graphSummaryData.push([
        `Max Value (${renderUnitSymbol(exportData.unit)})`,
        exportData.renderValue
          ? exportData.renderValue(maxValue)
          : maxValue.toFixed(3),
      ]);
      graphSummaryData.push([
        `Average Value (${renderUnitSymbol(exportData.unit)})`,
        exportData.renderValue
          ? exportData.renderValue(avgValue)
          : avgValue.toFixed(3),
      ]);
    }
  }

  return graphSummaryData;
}
