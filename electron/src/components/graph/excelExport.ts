import * as XLSX from "xlsx";
import ExcelJS from "exceljs";
import uPlot from "uplot";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig, SeriesData, GraphLine } from "./types";
import { useLogsStore } from "@/stores/logsStore";

export type GraphExportData = {
  config: GraphConfig;
  data: SeriesData; // Always a single series
  unit?: Unit;
  renderValue?: (value: number) => string;
};

// Type for combined sheet data used in Analysis
type CombinedSheetData = {
  sheetName: string;
  timestamps: number[];
  values: number[];
  unit: string;
  seriesTitle: string;
  graphTitle: string;
  targetLines: GraphLine[];
  color?: string;
};

// Utility function to format date/time in German locale
function formatDateTime(date: Date): string {
  return date.toLocaleString("de-DE", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export async function exportGraphsToExcel(
  graphDataMap: Map<string, () => GraphExportData | null>,
  groupId: string,
): Promise<void> {
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

    // Collect all sheet data for Analysis sheet
    const allSheetData: CombinedSheetData[] = [];

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

      // Generate better sheet name based on graph title, series title, and unit
      const sheetName = generateSheetName(
        exportData.config.title,
        seriesTitle,
        exportData.unit,
        usedSheetNames,
      );

      // Create combined sheet with data and stats
      // Pass seriesTitle and unit to avoid fragile reverse-engineering from sheet name
      const combinedWorksheet = createCombinedSheet(graphLineData, sheetName, seriesTitle, exportData.unit);
      XLSX.utils.book_append_sheet(workbook, combinedWorksheet, sheetName);

      // Collect data for Analysis sheet
      const [timestamps, values] = seriesToUPlotData(series.newData.long);
      allSheetData.push({
        sheetName,
        timestamps,
        values,
        unit: renderUnitSymbol(exportData.unit) || "",
        seriesTitle,
        graphTitle: exportData.config.title,
        targetLines,
        color: series.color,
      });

      processedCount++;
    });

    if (processedCount === 0) {
      alert("No data available to export from any graphs in this group");
      return;
    }

    // Create Analysis sheet with combined data and chart
    const analysisSheet = await createAnalysisSheet(
      allSheetData,
      groupId
    );
    XLSX.utils.book_append_sheet(workbook, analysisSheet, "Analysis");

    // Write XLSX to buffer first
    const xlsxBuffer = XLSX.write(workbook, { type: "buffer", bookType: "xlsx" });

    // Convert to ExcelJS workbook to add chart image
    let excelJSWorkbook: ExcelJS.Workbook;
    try {
      excelJSWorkbook = new ExcelJS.Workbook();
      await excelJSWorkbook.xlsx.load(xlsxBuffer);
    } catch (error) {
      console.error(
        "Failed to load generated XLSX buffer into ExcelJS workbook",
        error
      );
      alert(
        "Export failed while preparing the Excel file. The generated workbook data was invalid or could not be processed."
      );
      return;
    }

    // Find the Analysis sheet
    const analysisWorksheet = excelJSWorkbook.getWorksheet("Analysis");

    if (analysisWorksheet) {
      // Generate chart image
      const sortedTimestamps = Array.from(
        new Set(allSheetData.flatMap((d) => d.timestamps))
      ).sort((a, b) => a - b);

      const startTime = sortedTimestamps[0];
      const startDate = new Date(startTime);
      const endDate = new Date(sortedTimestamps[sortedTimestamps.length - 1]);

      const timeRangeTitle = `${formatDateTime(startDate)} bis ${formatDateTime(endDate)}`;

      const chartImage = await generateChartImage(
        allSheetData,
        groupId,
        timeRangeTitle,
        sortedTimestamps,
        startTime
      );

      if (chartImage) {
        // Add chart image to worksheet
        const chartImageId = excelJSWorkbook.addImage({
          base64: chartImage,
          extension: "png",
        });

        // Find a good position for the image (after the data and metadata)
        const lastRow = analysisWorksheet.rowCount;

        analysisWorksheet.addImage(chartImageId, {
          tl: { col: 0, row: lastRow + 2 }, // top-left
          ext: { width: 1200, height: 600 }, // chart size
        });

        // Generate and add legend image below the chart
        const legendImage = generateLegendImage(allSheetData);
        if (legendImage) {
          const legendImageId = excelJSWorkbook.addImage({
            base64: legendImage,
            extension: "png",
          });

          // Position legend below the chart (approximately 32 rows for 600px chart at ~19px per row)
          analysisWorksheet.addImage(legendImageId, {
            tl: { col: 0, row: lastRow + 2 + 32 }, // below chart
            ext: { width: 1200, height: 50 }, // legend size
          });
        }
      }
    }

    // Write final file with ExcelJS
    const filename = `${groupId.toLowerCase().replace(/\s+/g, "_")}_export_${exportTimestamp}.xlsx`;
    const buffer = await excelJSWorkbook.xlsx.writeBuffer();

    // Create blob and download
    const blob = new Blob([buffer], {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    link.click();
    window.URL.revokeObjectURL(url);
  } catch (error) {
    alert(
      `Error exporting data to Excel: ${error instanceof Error ? error.message : "Unknown error"}. Please try again.`,
    );
  }
}

// Get series color from machine data or fallback to default
function getSeriesColor(color?: string): string {
  // Use the color from the machine data if available
  if (color) {
    return color;
  }
  
  // Fallback to default color
  return "#9b59b6"; // Purple fallback
}

// Generate a separate legend image
function generateLegendImage(allSheetData: CombinedSheetData[]): string | null {
  try {
    const canvasWidth = 1200;
    const itemSpacing = 15;
    const rowHeight = 20;
    const topPadding = 5;
    const bottomPadding = 5;
    const maxItemWidth = 1100; // Maximum X position before wrapping
    const itemStartX = 20;

    // Create a temporary canvas to measure text
    const tempCanvas = document.createElement("canvas");
    const tempCtx = tempCanvas.getContext("2d");
    if (!tempCtx) return null;

    tempCtx.font = "12px sans-serif";

    // First pass: Calculate required height based on layout
    let legendX = itemStartX;
    let rowCount = 1;

    allSheetData.forEach((sheetData, index) => {
      const label = sheetData.sheetName;
      const textWidth = tempCtx.measureText(label).width;
      const itemWidth = 16 + textWidth + itemSpacing; // color box (16px) + spacing

      // Check if item fits in current row
      if (legendX + itemWidth > maxItemWidth && index < allSheetData.length - 1) {
        // Wrap to next line
        legendX = itemStartX;
        rowCount++;
      }

      legendX += itemWidth;
    });

    // Calculate required canvas height
    const requiredHeight = topPadding + rowHeight + (rowCount - 1) * rowHeight + bottomPadding;

    // Create canvas with calculated height
    const legendCanvas = document.createElement("canvas");
    legendCanvas.width = canvasWidth;
    legendCanvas.height = requiredHeight;
    const ctx = legendCanvas.getContext("2d");

    if (!ctx) return null;

    // Draw white background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, canvasWidth, requiredHeight);

    // Draw legend items
    ctx.font = "12px sans-serif";
    ctx.textAlign = "left";

    let currentX = itemStartX;
    let currentY = topPadding + 15; // Vertical center of first row

    allSheetData.forEach((sheetData, index) => {
      const color = getSeriesColor(sheetData.color);
      const label = sheetData.sheetName;
      const textWidth = ctx.measureText(label).width;
      const itemWidth = 16 + textWidth + itemSpacing;

      // Check if item fits in current row
      if (currentX + itemWidth > maxItemWidth && index < allSheetData.length - 1) {
        // Wrap to next line
        currentX = itemStartX;
        currentY += rowHeight;
      }

      // Draw color indicator (small rectangle)
      ctx.fillStyle = color;
      ctx.fillRect(currentX, currentY - 6, 12, 12);

      // Draw label text
      ctx.fillStyle = "#333";
      ctx.fillText(label, currentX + 16, currentY + 4);

      // Move to next position
      currentX += itemWidth;
    });

    const imageData = legendCanvas.toDataURL("image/png");
    return imageData.split(",")[1];
  } catch (error) {
    console.error("Error generating legend image:", error);
    return null;
  }
}

// Generate chart image from data using uPlot
async function generateChartImage(
  allSheetData: CombinedSheetData[],
  groupId: string,
  timeRangeTitle: string,
  sortedTimestamps: number[],
  startTime: number,
): Promise<string | null> {
  let container: HTMLDivElement | null = null;
  let plot: uPlot | null = null;

  try {
    // Create an off-screen div for the chart
    container = document.createElement("div");
    container.style.width = "1200px";
    container.style.height = "600px";
    container.style.position = "absolute";
    container.style.left = "-9999px";
    document.body.appendChild(container);

    // Prepare data for uPlot: [timestamps in seconds, ...value arrays]
    const chartData: number[][] = [
      sortedTimestamps.map((ts) => (ts - startTime) / 1000), // X-axis: seconds from start
    ];

    const series: uPlot.Series[] = [
      {
        label: "Time (s)",
      },
    ];

    // Add each data series
    allSheetData.forEach((sheetData) => {
      const values = new Array(sortedTimestamps.length).fill(null);

      // Map values to corresponding timestamps
      sheetData.timestamps.forEach((ts, idx) => {
        const timeIndex = sortedTimestamps.indexOf(ts);
        if (timeIndex !== -1) {
          values[timeIndex] = sheetData.values[idx];
        }
      });

      chartData.push(values);

      const color = getSeriesColor(sheetData.color);

      series.push({
        label: sheetData.sheetName,
        stroke: color,
        width: 2,
        points: {
          show: true,
          size: 3,
          width: 1,
        },
      });
    });

    // Create uPlot instance
    const opts: uPlot.Options = {
      title: `${groupId} - ${timeRangeTitle}`,
      width: 1200,
      height: 600,
      series,
      scales: {
        x: {
          time: false,
        },
      },
      axes: [
        {
          label: "Time (seconds)",
          stroke: "#333",
          grid: { stroke: "#e0e0e0", width: 1 },
        },
        {
          label: "Values",
          stroke: "#333",
          grid: { stroke: "#e0e0e0", width: 1 },
        },
      ],
      legend: {
        show: false, // Legend is a separate image
      },
      cursor: {
        show: false,
      },
    };

    plot = new uPlot(opts, chartData as uPlot.AlignedData, container);

    // Wait for render
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Get the canvas element
    const canvas = container.querySelector("canvas");
    if (!canvas) {
      return null;
    }

    // Get the image data directly from uPlot's canvas
    const imageData = canvas.toDataURL("image/png");

    // Return base64 data (remove the data:image/png;base64, prefix)
    return imageData.split(",")[1];
  } catch (error) {
    console.error("Error generating chart image:", error);
    return null;
  } finally {
    // Ensure cleanup happens regardless of success or failure
    if (plot) {
      plot.destroy();
    }
    if (container && document.body.contains(container)) {
      document.body.removeChild(container);
    }
  }
}

// Create Analysis sheet with combined data from all sheets
async function createAnalysisSheet(
  allSheetData: CombinedSheetData[],
  groupId: string,
): Promise<XLSX.WorkSheet> {
  // Find all unique timestamps across all series
  const allTimestamps = new Set<number>();
  allSheetData.forEach((data) => {
    data.timestamps.forEach((ts) => allTimestamps.add(ts));
  });

  const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);

  // Calculate time range
  const startTime = sortedTimestamps[0];
  const endTime = sortedTimestamps[sortedTimestamps.length - 1];
  const startDate = new Date(startTime);
  const endDate = new Date(endTime);

  // Format time range for title
  const timeRangeTitle = `${formatDateTime(startDate)} bis ${formatDateTime(endDate)}`;

  // Get user comments/logs from store
  // Filter for logs that are explicitly marked as user comments:
  // - Must be within the time range
  // - Must have level "info" (user annotations are logged as info)
  // - Must explicitly contain the word "comment" to distinguish from other info logs
  const logs = useLogsStore.getState().entries;
  const relevantComments = logs.filter(
    (log) =>
      log.timestamp.getTime() >= startTime &&
      log.timestamp.getTime() <= endTime &&
      log.level === "info" &&
      log.message.toLowerCase().includes("comment")
  );

  // Map data by timestamp for efficient lookup
  const dataByTimestamp = new Map<
    number,
    Map<string, number>
  >();

  allSheetData.forEach((sheetData) => {
    sheetData.timestamps.forEach((ts, idx) => {
      if (!dataByTimestamp.has(ts)) {
        dataByTimestamp.set(ts, new Map());
      }
      dataByTimestamp.get(ts)!.set(sheetData.sheetName, sheetData.values[idx]);
    });
  });

  // Build column headers based on available data
  const columns: string[] = ["Timestamp"];

  // Simply use sheet names from the data
  const availableColumns = allSheetData.map(d => d.sheetName);
  columns.push(...availableColumns);

  // Add comments column
  columns.push("User Comments");

  // Create sheet data array
  const sheetData: any[][] = [];

  // Title row
  const titleRow = [
    `${groupId} - ${timeRangeTitle}`,
    ...Array(columns.length - 1).fill(""),
  ];
  sheetData.push(titleRow);

  // Empty row
  sheetData.push(Array(columns.length).fill(""));

  // Target values row (if any target lines exist)
  const targetValues: any[] = ["Target Values"];
  let hasTargets = false;

  allSheetData.forEach((sheetDataEntry) => {
    if (sheetDataEntry.targetLines.length > 0) {
      const targetLine = sheetDataEntry.targetLines.find(
        (line) => line.type === "target"
      );
      if (targetLine) {
        targetValues.push(targetLine.value.toFixed(2));
        hasTargets = true;
      } else {
        targetValues.push("");
      }
    } else {
      targetValues.push("");
    }
  });

  targetValues.push(""); // Empty for comments column

  if (hasTargets) {
    sheetData.push(targetValues);
    sheetData.push(Array(columns.length).fill("")); // Empty row after targets
  }

  // Header row
  sheetData.push(columns);

  // Data rows with time in seconds from start
  const dataStartRow = sheetData.length;
  let maxSeconds = 0;

  sortedTimestamps.forEach((timestamp) => {
    const row: any[] = [];

    // Calculate seconds from start
    const secondsFromStart = Math.floor((timestamp - startTime) / 1000);
    maxSeconds = Math.max(maxSeconds, secondsFromStart);
    row.push(secondsFromStart);

    // Add data for each column
    allSheetData.forEach((sheetDataEntry) => {
      const tsData = dataByTimestamp.get(timestamp);
      if (tsData && tsData.has(sheetDataEntry.sheetName)) {
        row.push(Number(tsData.get(sheetDataEntry.sheetName)!.toFixed(2)));
      } else {
        row.push("");
      }
    });

    // Check for comments at this timestamp (within 1 second tolerance)
    const comment = relevantComments.find(
      (log) => Math.abs(log.timestamp.getTime() - timestamp) < 1000
    );
    row.push(comment ? comment.message : "");

    sheetData.push(row);
  });

  // Add metadata section after data
  sheetData.push(Array(columns.length).fill("")); // Empty row
  sheetData.push(Array(columns.length).fill("")); // Empty row

  // Get environment info for version details
  let versionInfo = "";
  let commitInfo = "";
  try {
    const envInfo = await window.environment.getInfo();
    if (envInfo.qitechOsGitAbbreviation) {
      versionInfo = envInfo.qitechOsGitAbbreviation;
    }
    if (envInfo.qitechOsGitCommit) {
      commitInfo = envInfo.qitechOsGitCommit.substring(0, 8); // First 8 chars of commit hash
    }
  } catch (error) {
    console.warn("Failed to fetch environment info", error);
  }

  // Software information
  sheetData.push(["Software Information", ...Array(columns.length - 1).fill("")]);
  sheetData.push(["Software", "QiTech Control", ...Array(columns.length - 2).fill("")]);
  sheetData.push(["Version", versionInfo || "Unknown", ...Array(columns.length - 2).fill("")]);
  if (commitInfo) {
    sheetData.push(["Git Commit", commitInfo, ...Array(columns.length - 2).fill("")]);
  }
  sheetData.push([
    "Export Date",
    new Date().toLocaleString("de-DE"),
    ...Array(columns.length - 2).fill(""),
  ]);

  // Comment statistics
  sheetData.push(Array(columns.length).fill("")); // Empty row
  sheetData.push(["Comment Statistics", ...Array(columns.length - 1).fill("")]);
  sheetData.push([
    "Total Comments",
    relevantComments.length.toString(),
    ...Array(columns.length - 2).fill(""),
  ]);

  // Convert to worksheet
  const worksheet = XLSX.utils.aoa_to_sheet(sheetData);

  // Merge title cells
  if (!worksheet["!merges"]) worksheet["!merges"] = [];
  worksheet["!merges"].push({
    s: { r: 0, c: 0 },
    e: { r: 0, c: columns.length - 1 },
  });

  // Set column widths
  const colWidths = [
    { wch: 12 }, // Timestamp (seconds)
    ...availableColumns.map(() => ({ wch: 12 })),
    { wch: 40 }, // Comments column
  ];
  worksheet["!cols"] = colWidths;

  // Add chart creation instructions (kept as fallback)
  sheetData.push(Array(columns.length).fill("")); // Empty row
  sheetData.push(Array(columns.length).fill("")); // Empty row
  sheetData.push([
    "Chart Instructions",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "1. Select all data from row " + dataStartRow + " to the last data row",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "2. Insert > Chart > Scatter Chart with Straight Lines and Markers",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "3. X-axis: Time (seconds), Y-axis: All measurement columns",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "4. Set X-axis range: 0 to " + maxSeconds,
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "5. Set Y-axis range: 0 to 1000",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "6. Position legend at bottom",
    "",
    "",
    "",
    "",
    "",
    "",
  ]);
  sheetData.push([
    "7. Chart Title: " + groupId + " - " + timeRangeTitle,
    "",
    "",
    "",
    "",
    "",
    "",
  ]);

  return worksheet;
}

// Generate better sheet names based on graph title, series title, and unit
function generateSheetName(
  graphTitle: string,
  seriesTitle: string,
  unit: Unit | undefined,
  usedSheetNames: Set<string>,
): string {
  // Determine the type of data based on unit
  const unitSymbol = renderUnitSymbol(unit) || "";

  // Map unit symbols to friendly names for sheet naming
  const unitFriendlyNames: Record<string, string> = {
    "°C": "Temp",
    "W": "Watt",
    "A": "Ampere",
    "bar": "Bar",
    "rpm": "Rpm",
    "1/min": "Rpm",
    "mm": "mm",
    "%": "Percent",
  };

  // Create descriptive sheet name
  let sheetName = "";

  // If the series title is generic (e.g., "Series 1", "Series 2"), use unit name
  if (/^Series \d+$/i.test(seriesTitle)) {
    // For generic series names, prefer the unit-based name if available
    const friendlyUnitName = unitFriendlyNames[unitSymbol];
    sheetName = friendlyUnitName || seriesTitle;
  } else {
    // For specific series names (e.g., "Nozzle", "Front", "Diameter")
    // Combine the series title with the unit if it adds clarity
    const friendlyUnitName = unitFriendlyNames[unitSymbol];
    
    // Only append unit name if it provides additional context
    // Don't append for standalone measurements like "Diameter" or "Roundness"
    if (friendlyUnitName && !seriesTitle.toLowerCase().includes(friendlyUnitName.toLowerCase())) {
      sheetName = `${seriesTitle} ${friendlyUnitName}`;
    } else {
      sheetName = seriesTitle;
    }
  }

  // Sanitize and limit length
  sheetName = sheetName
    .replace(/[\\/?*$:[\]]/g, "_")
    .substring(0, 31);

  if (!sheetName || sheetName.trim().length === 0) {
    sheetName = "Sheet";
  }

  // Make unique if needed
  let finalName = sheetName;
  let counter = 1;

  while (usedSheetNames.has(finalName)) {
    const suffix = `_${counter}`;
    const maxBaseLength = 31 - suffix.length;
    finalName = `${sheetName.substring(0, maxBaseLength)}${suffix}`;
    counter++;
  }

  usedSheetNames.add(finalName);
  return finalName;
}

// Create combined sheet with data (columns A-C) and stats (columns E-F)
function createCombinedSheet(
  graphLine: {
    graphTitle: string;
    lineTitle: string;
    series: TimeSeries;
    color?: string;
    unit?: Unit;
    renderValue?: (value: number) => string;
    config: GraphConfig;
    targetLines: GraphLine[];
  },
  sheetName: string,
  seriesTitle: string,
  unit: Unit | undefined,
): XLSX.WorkSheet {
  const [timestamps, values] = seriesToUPlotData(graphLine.series.long);
  const unitSymbol = renderUnitSymbol(unit) || "";

  // Create a 2D array for the combined sheet
  const sheetData: any[][] = [];

  // Create column header using the original series title and unit passed as parameters.
  // This approach is more robust than reverse-engineering from the sheet name,
  // which may not follow predictable naming patterns depending on the generateSheetName logic.
  // Format: "unit seriesTitle" (e.g., "°C Nozzle", "W Ampere") or just "seriesTitle" if no unit
  const col1Header = unitSymbol ? `${unitSymbol} ${seriesTitle}` : seriesTitle;

  // Add header row - Column A: Timestamp, Column B: Value, Column C: (empty), Column D: (empty), Column E-F: Stats
  sheetData.push([
    "Timestamp",
    col1Header,
    "",
    "",
    "Statistic",
    "Value",
  ]);

  // Prepare stats data
  const statsRows: string[][] = [];

  statsRows.push(["Graph", graphLine.graphTitle]);
  statsRows.push(["Line Name", graphLine.lineTitle]);
  statsRows.push(["Line Color", graphLine.color || "Default"]);
  statsRows.push([
    "Generated",
    formatDateTime(new Date()),
  ]);
  statsRows.push(["", ""]);
  statsRows.push(["Total Data Points", timestamps.length.toString()]);

  if (timestamps.length > 0) {
    const firstDate = new Date(timestamps[0]);
    const lastDate = new Date(timestamps[timestamps.length - 1]);

    statsRows.push([
      "Time Range Start",
      formatDateTime(firstDate),
    ]);
    statsRows.push([
      "Time Range End",
      formatDateTime(lastDate),
    ]);

    const duration = timestamps[timestamps.length - 1] - timestamps[0];
    const durationHours = (duration / (1000 * 60 * 60)).toFixed(2);
    statsRows.push(["Duration (hours)", durationHours]);

    if (values.length > 0) {
      const minValue = Math.min(...values);
      const maxValue = Math.max(...values);
      const avgValue = values.reduce((a, b) => a + b, 0) / values.length;
      const stdDev = Math.sqrt(
        values.reduce((sum, val) => sum + Math.pow(val - avgValue, 2), 0) /
          values.length,
      );

      statsRows.push(["", ""]);
      statsRows.push([
        `Minimum Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(minValue)
          : minValue.toFixed(3),
      ]);
      statsRows.push([
        `Maximum Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(maxValue)
          : maxValue.toFixed(3),
      ]);
      statsRows.push([
        `Average Value (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(avgValue)
          : avgValue.toFixed(3),
      ]);
      statsRows.push([
        `Standard Deviation (${unitSymbol})`,
        graphLine.renderValue
          ? graphLine.renderValue(stdDev)
          : stdDev.toFixed(3),
      ]);
      statsRows.push([
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

      statsRows.push(["", ""]);
      statsRows.push([
        `25th Percentile (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p25) : p25.toFixed(3),
      ]);
      statsRows.push([
        `50th Percentile (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p50) : p50.toFixed(3),
      ]);
      statsRows.push([
        `75th Percentile (${unitSymbol})`,
        graphLine.renderValue ? graphLine.renderValue(p75) : p75.toFixed(3),
      ]);
    }
  }

  // Add data rows with stats in columns E-F
  const maxRows = Math.max(timestamps.length, statsRows.length);

  for (let i = 0; i < maxRows; i++) {
    const row: any[] = ["", "", "", ""];

    // Add timestamp and value data (columns A-B)
    if (i < timestamps.length) {
      const timestamp = timestamps[i];
      const value = values[i];

      // Format timestamp as dd.mm.yyyy hh:mm:ss
      const date = new Date(timestamp);
      const formattedDate = formatDateTime(date);

      row[0] = formattedDate;
      row[1] = graphLine.renderValue
        ? graphLine.renderValue(value)
        : value?.toFixed(3) || "";
    }

    // Add stats (columns E-F)
    if (i < statsRows.length) {
      row[4] = statsRows[i][0];
      row[5] = statsRows[i][1];
    } else {
      row[4] = "";
      row[5] = "";
    }

    sheetData.push(row);
  }

  // Convert to worksheet
  const worksheet = XLSX.utils.aoa_to_sheet(sheetData);

  // Set column widths for better readability
  worksheet["!cols"] = [
    { wch: 20 }, // Timestamp
    { wch: 15 }, // Value
    { wch: 5 },  // Empty column C
    { wch: 5 },  // Empty column D
    { wch: 30 }, // Statistic name
    { wch: 20 }, // Statistic value
  ];

  return worksheet;
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
