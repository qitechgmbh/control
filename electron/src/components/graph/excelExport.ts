import * as XLSX from "xlsx";
// @ts-ignore - ExcelJS types not installed
import ExcelJS from "exceljs";
import uPlot from "uplot";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig, SeriesData, GraphLine } from "./types";
import { LogEntry } from "@/stores/logsStore";

/**
 * Type definitions for export data structures
 */
export type GraphExportData = {
  config: GraphConfig;
  data: SeriesData;
  unit?: Unit;
  renderValue?: (value: number) => string;
};

export type PidSettings = {
  kp: number;
  ki: number;
  kd: number;
  zone?: string; // For temperature zones (front, middle, back, nozzle)
};

export type PidData = {
  temperature?: Record<string, PidSettings>; // keyed by zone
  pressure?: PidSettings;
};

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

/**
 * Utility class for date/time formatting and manipulation
 */
class DateFormatter {
  static readonly GERMAN_LOCALE = "de-DE";
  
  static format(date: Date): string {
    return date.toLocaleString(this.GERMAN_LOCALE, {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  static getExportTimestamp(): string {
    return new Date()
      .toISOString()
      .replace(/[:.]/g, "-")
      .slice(0, 19);
  }

  static formatTimeRange(startTime: number, endTime: number): string {
    const startDate = new Date(startTime);
    const endDate = new Date(endTime);
    return `${this.format(startDate)} bis ${this.format(endDate)}`;
  }
}

/**
 * Manages unique sheet name generation for Excel workbooks
 */
class SheetNameManager {
  private usedNames = new Set<string>();
  
  private readonly UNIT_FRIENDLY_NAMES: Record<string, string> = {
    "°C": "Temp",
    "W": "Watt",
    "A": "Ampere",
    "bar": "Bar",
    "rpm": "Rpm",
    "1/min": "Rpm",
    "mm": "mm",
    "%": "Percent",
  };

  generate(
    graphTitle: string,
    seriesTitle: string,
    unit: Unit | undefined
  ): string {
    const unitSymbol = renderUnitSymbol(unit) || "";
    let sheetName = "";

    // Use unit-based name for generic series, otherwise use series title
    if (/^Series \d+$/i.test(seriesTitle)) {
      const friendlyUnitName = this.UNIT_FRIENDLY_NAMES[unitSymbol];
      sheetName = friendlyUnitName || seriesTitle;
    } else {
      const friendlyUnitName = this.UNIT_FRIENDLY_NAMES[unitSymbol];
      if (
        friendlyUnitName &&
        !seriesTitle.toLowerCase().includes(friendlyUnitName.toLowerCase())
      ) {
        sheetName = `${seriesTitle} ${friendlyUnitName}`;
      } else {
        sheetName = seriesTitle;
      }
    }

    return this.makeUnique(this.sanitize(sheetName));
  }

  private sanitize(name: string): string {
    return name
      .replace(/[\\/?*$:[\]]/g, "_")
      .substring(0, 31)
      .trim() || "Sheet";
  }

  private makeUnique(name: string): string {
    let finalName = name;
    let counter = 1;

    while (this.usedNames.has(finalName)) {
      const suffix = `_${counter}`;
      const maxBaseLength = 31 - suffix.length;
      finalName = `${name.substring(0, maxBaseLength)}${suffix}`;
      counter++;
    }

    this.usedNames.add(finalName);
    return finalName;
  }
}

/**
 * Handles statistical calculations for time series data
 */
class StatisticsCalculator {
  static calculate(values: number[]): {
    min: number;
    max: number;
    avg: number;
    stdDev: number;
    range: number;
    p25: number;
    p50: number;
    p75: number;
  } {
    if (values.length === 0) {
      throw new Error("Cannot calculate statistics for empty array");
    }

    const min = Math.min(...values);
    const max = Math.max(...values);
    const avg = values.reduce((a, b) => a + b, 0) / values.length;
    const stdDev = Math.sqrt(
      values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) /
        values.length
    );
    const range = max - min;

    const sortedValues = [...values].sort((a, b) => a - b);
    const p25 = sortedValues[Math.floor(sortedValues.length * 0.25)];
    const p50 = sortedValues[Math.floor(sortedValues.length * 0.5)];
    const p75 = sortedValues[Math.floor(sortedValues.length * 0.75)];

    return { min, max, avg, stdDev, range, p25, p50, p75 };
  }
}

/**
 * Filters and manages log comments for export
 */
class CommentManager {
  static filterRelevant(
    logs: LogEntry[],
    startTime: number,
    endTime: number
  ): LogEntry[] {
    return logs.filter(
      (log) =>
        log.timestamp.getTime() >= startTime &&
        log.timestamp.getTime() <= endTime &&
        log.level === "info" &&
        log.message.toLowerCase().includes("comment")
    );
  }

  static findAtTimestamp(
    comments: LogEntry[],
    timestamp: number,
    tolerance: number = 1000
  ): LogEntry | undefined {
    return comments.find(
      (log) => Math.abs(log.timestamp.getTime() - timestamp) < tolerance
    );
  }
}

/**
 * Builds metadata sections for Excel sheets
 */
class MetadataBuilder {
  private rows: string[][] = [];

  addSection(title: string, columnCount: number): this {
    this.rows.push([title, ...Array(columnCount - 1).fill("")]);
    return this;
  }

  addRow(key: string, value: string, columnCount: number): this {
    this.rows.push([key, value, ...Array(columnCount - 2).fill("")]);
    return this;
  }

  addEmptyRow(columnCount: number): this {
    this.rows.push(Array(columnCount).fill(""));
    return this;
  }

  async addSoftwareInfo(columnCount: number): Promise<this> {
    let versionInfo = "";
    let commitInfo = "";

    try {
      const envInfo = await window.environment.getInfo();
      if (envInfo.qitechOsGitAbbreviation) {
        versionInfo = envInfo.qitechOsGitAbbreviation;
      }
      if (envInfo.qitechOsGitCommit) {
        commitInfo = envInfo.qitechOsGitCommit.substring(0, 8);
      }
    } catch (error) {
      console.warn("Failed to fetch environment info", error);
    }

    this.addSection("Software Information", columnCount);
    this.addRow("Software", "QiTech Control", columnCount);
    this.addRow("Version", versionInfo || "Unknown", columnCount);
    if (commitInfo) {
      this.addRow("Git Commit", commitInfo, columnCount);
    }
    this.addRow(
      "Export Date",
      DateFormatter.format(new Date()),
      columnCount
    );

    return this;
  }

  addPidSettings(pidData: PidData | undefined, columnCount: number): this {
    if (!pidData) return this;

    this.addEmptyRow(columnCount);
    this.addSection("PID Controller Settings", columnCount);

    // Temperature PID settings
    if (pidData.temperature) {
      this.addEmptyRow(columnCount);
      this.addRow("Temperature Controllers", "", columnCount);
      
      Object.entries(pidData.temperature).forEach(([zone, settings]) => {
        this.addRow(`  ${zone} - Kp`, settings.kp.toFixed(3), columnCount);
        this.addRow(`  ${zone} - Ki`, settings.ki.toFixed(3), columnCount);
        this.addRow(`  ${zone} - Kd`, settings.kd.toFixed(3), columnCount);
      });
    }

    // Pressure PID settings
    if (pidData.pressure) {
      this.addEmptyRow(columnCount);
      this.addRow("Pressure Controller", "", columnCount);
      this.addRow("  Kp", pidData.pressure.kp.toFixed(3), columnCount);
      this.addRow("  Ki", pidData.pressure.ki.toFixed(3), columnCount);
      this.addRow("  Kd", pidData.pressure.kd.toFixed(3), columnCount);
    }

    return this;
  }

  getRows(): string[][] {
    return this.rows;
  }
}

/**
 * Creates individual data sheets for each series
 */
class DataSheetBuilder {
  constructor(
    private graphLine: {
      graphTitle: string;
      lineTitle: string;
      series: TimeSeries;
      color?: string;
      unit?: Unit;
      renderValue?: (value: number) => string;
      config: GraphConfig;
      targetLines: GraphLine[];
    },
    private seriesTitle: string,
    private unit: Unit | undefined
  ) {}

  build(): XLSX.WorkSheet {
    const [timestamps, values] = seriesToUPlotData(this.graphLine.series.long);
    const unitSymbol = renderUnitSymbol(this.unit) || "";

    const sheetData: any[][] = [];

    // Build header
    const col1Header = unitSymbol
      ? `${unitSymbol} ${this.seriesTitle}`
      : this.seriesTitle;

    sheetData.push([
      "Timestamp",
      col1Header,
      "",
      "",
      "Statistic",
      "Value",
    ]);

    // Build stats section
    const statsRows = this.buildStatsRows(
      timestamps,
      values,
      unitSymbol
    );

    // Combine data and stats rows
    const maxRows = Math.max(timestamps.length, statsRows.length);
    for (let i = 0; i < maxRows; i++) {
      const row = this.buildDataRow(
        i,
        timestamps,
        values,
        statsRows
      );
      sheetData.push(row);
    }

    // Convert to worksheet
    const worksheet = XLSX.utils.aoa_to_sheet(sheetData);
    worksheet["!cols"] = [
      { wch: 20 }, // Timestamp
      { wch: 15 }, // Value
      { wch: 5 }, // Empty
      { wch: 5 }, // Empty
      { wch: 30 }, // Statistic name
      { wch: 20 }, // Statistic value
    ];

    return worksheet;
  }

  private buildStatsRows(
    timestamps: number[],
    values: number[],
    unitSymbol: string
  ): string[][] {
    const statsRows: string[][] = [];

    statsRows.push(["Graph", this.graphLine.graphTitle]);
    statsRows.push(["Line Name", this.graphLine.lineTitle]);
    statsRows.push(["Line Color", this.graphLine.color || "Default"]);
    statsRows.push(["Generated", DateFormatter.format(new Date())]);
    statsRows.push(["", ""]);
    statsRows.push(["Total Data Points", timestamps.length.toString()]);

    if (timestamps.length > 0) {
      const firstDate = new Date(timestamps[0]);
      const lastDate = new Date(timestamps[timestamps.length - 1]);

      statsRows.push(["Time Range Start", DateFormatter.format(firstDate)]);
      statsRows.push(["Time Range End", DateFormatter.format(lastDate)]);

      const duration = timestamps[timestamps.length - 1] - timestamps[0];
      const durationHours = (duration / (1000 * 60 * 60)).toFixed(2);
      statsRows.push(["Duration (hours)", durationHours]);

      if (values.length > 0) {
        const stats = StatisticsCalculator.calculate(values);

        statsRows.push(["", ""]);
        statsRows.push([
          `Minimum Value (${unitSymbol})`,
          this.formatValue(stats.min),
        ]);
        statsRows.push([
          `Maximum Value (${unitSymbol})`,
          this.formatValue(stats.max),
        ]);
        statsRows.push([
          `Average Value (${unitSymbol})`,
          this.formatValue(stats.avg),
        ]);
        statsRows.push([
          `Standard Deviation (${unitSymbol})`,
          this.formatValue(stats.stdDev),
        ]);
        statsRows.push([
          `Range (${unitSymbol})`,
          this.formatValue(stats.range),
        ]);

        statsRows.push(["", ""]);
        statsRows.push([
          `25th Percentile (${unitSymbol})`,
          this.formatValue(stats.p25),
        ]);
        statsRows.push([
          `50th Percentile (${unitSymbol})`,
          this.formatValue(stats.p50),
        ]);
        statsRows.push([
          `75th Percentile (${unitSymbol})`,
          this.formatValue(stats.p75),
        ]);
      }
    }

    return statsRows;
  }

  private buildDataRow(
    index: number,
    timestamps: number[],
    values: number[],
    statsRows: string[][]
  ): any[] {
    const row: any[] = ["", "", "", ""];

    // Add timestamp and value data
    if (index < timestamps.length) {
      const date = new Date(timestamps[index]);
      row[0] = DateFormatter.format(date);
      row[1] = this.formatValue(values[index]);
    }

    // Add stats
    if (index < statsRows.length) {
      row[4] = statsRows[index][0];
      row[5] = statsRows[index][1];
    } else {
      row[4] = "";
      row[5] = "";
    }

    return row;
  }

  private formatValue(value: number): string {
    return this.graphLine.renderValue
      ? this.graphLine.renderValue(value)
      : value?.toFixed(3) || "";
  }
}

/**
 * Creates the combined analysis sheet with all series data
 */
class AnalysisSheetBuilder {
  constructor(
    private allSheetData: CombinedSheetData[],
    private groupId: string,
    private logs: LogEntry[],
    private pidData?: PidData
  ) {}

  async build(): Promise<XLSX.WorkSheet> {
    // Get sorted timestamps
    const sortedTimestamps = this.getSortedTimestamps();
    const startTime = sortedTimestamps[0];
    const endTime = sortedTimestamps[sortedTimestamps.length - 1];

    // Filter relevant comments
    const relevantComments = CommentManager.filterRelevant(
      this.logs,
      startTime,
      endTime
    );

    // Build data by timestamp map
    const dataByTimestamp = this.buildDataByTimestampMap();

    // Build columns
    const columns = this.buildColumns();

    // Create sheet data array
    const sheetData: any[][] = [];

    // Add title row
    const timeRangeTitle = DateFormatter.formatTimeRange(startTime, endTime);
    sheetData.push([
      `${this.groupId} - ${timeRangeTitle}`,
      ...Array(columns.length - 1).fill(""),
    ]);
    sheetData.push(Array(columns.length).fill(""));

    // Add target values row if applicable
    this.addTargetValuesRow(sheetData, columns.length);

    // Add header row
    sheetData.push(columns);

    // Add data rows
    const dataStartRow = sheetData.length;
    let maxSeconds = 0;

    sortedTimestamps.forEach((timestamp) => {
      const row = this.buildDataRow(
        timestamp,
        startTime,
        dataByTimestamp,
        relevantComments
      );
      
      const secondsFromStart = Math.floor((timestamp - startTime) / 1000);
      maxSeconds = Math.max(maxSeconds, secondsFromStart);
      
      sheetData.push(row);
    });

    // Add metadata
    await this.addMetadata(sheetData, columns.length, relevantComments);

    // Add chart instructions
    this.addChartInstructions(
      sheetData,
      columns.length,
      dataStartRow,
      maxSeconds,
      timeRangeTitle
    );

    // Convert to worksheet
    const worksheet = XLSX.utils.aoa_to_sheet(sheetData);

    // Configure worksheet
    this.configureWorksheet(worksheet, columns.length);

    return worksheet;
  }

  private getSortedTimestamps(): number[] {
    const allTimestamps = new Set<number>();
    this.allSheetData.forEach((data) => {
      data.timestamps.forEach((ts) => allTimestamps.add(ts));
    });
    return Array.from(allTimestamps).sort((a, b) => a - b);
  }

  private buildDataByTimestampMap(): Map<number, Map<string, number>> {
    const dataByTimestamp = new Map<number, Map<string, number>>();

    this.allSheetData.forEach((sheetData) => {
      sheetData.timestamps.forEach((ts, idx) => {
        if (!dataByTimestamp.has(ts)) {
          dataByTimestamp.set(ts, new Map());
        }
        dataByTimestamp
          .get(ts)!
          .set(sheetData.sheetName, sheetData.values[idx]);
      });
    });

    return dataByTimestamp;
  }

  private buildColumns(): string[] {
    const columns: string[] = ["Timestamp"];
    const availableColumns = this.allSheetData.map((d) => d.sheetName);
    columns.push(...availableColumns);
    columns.push("User Comments");
    return columns;
  }

  private addTargetValuesRow(sheetData: any[][], columnCount: number): void {
    const targetValues: any[] = ["Target Values"];
    let hasTargets = false;

    this.allSheetData.forEach((sheetDataEntry) => {
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
      sheetData.push(Array(columnCount).fill(""));
    }
  }

  private buildDataRow(
    timestamp: number,
    startTime: number,
    dataByTimestamp: Map<number, Map<string, number>>,
    relevantComments: LogEntry[]
  ): any[] {
    const row: any[] = [];

    // Calculate seconds from start
    const secondsFromStart = Math.floor((timestamp - startTime) / 1000);
    row.push(secondsFromStart);

    // Add data for each column
    this.allSheetData.forEach((sheetDataEntry) => {
      const tsData = dataByTimestamp.get(timestamp);
      if (tsData && tsData.has(sheetDataEntry.sheetName)) {
        row.push(Number(tsData.get(sheetDataEntry.sheetName)!.toFixed(2)));
      } else {
        row.push("");
      }
    });

    // Check for comments at this timestamp
    const comment = CommentManager.findAtTimestamp(relevantComments, timestamp);
    row.push(comment ? comment.message : "");

    return row;
  }

  private async addMetadata(
    sheetData: any[][],
    columnCount: number,
    relevantComments: LogEntry[]
  ): Promise<void> {
    sheetData.push(Array(columnCount).fill(""));
    sheetData.push(Array(columnCount).fill(""));

    const metadataBuilder = new MetadataBuilder();
    await metadataBuilder.addSoftwareInfo(columnCount);

    // Add PID settings if available
    metadataBuilder.addPidSettings(this.pidData, columnCount);

    // Add comment statistics
    metadataBuilder.addEmptyRow(columnCount);
    metadataBuilder.addSection("Comment Statistics", columnCount);
    metadataBuilder.addRow(
      "Total Comments",
      relevantComments.length.toString(),
      columnCount
    );

    sheetData.push(...metadataBuilder.getRows());
  }

  private addChartInstructions(
    sheetData: any[][],
    columnCount: number,
    dataStartRow: number,
    maxSeconds: number,
    timeRangeTitle: string
  ): void {
    sheetData.push(Array(columnCount).fill(""));
    sheetData.push(Array(columnCount).fill(""));
    sheetData.push(["Chart Instructions", ...Array(columnCount - 1).fill("")]);
    sheetData.push([
      `1. Select all data from row ${dataStartRow} to the last data row`,
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      "2. Insert > Chart > Scatter Chart with Straight Lines and Markers",
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      "3. X-axis: Time (seconds), Y-axis: All measurement columns",
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      `4. Set X-axis range: 0 to ${maxSeconds}`,
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      "5. Set Y-axis range: 0 to 1000",
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      "6. Position legend at bottom",
      ...Array(columnCount - 1).fill(""),
    ]);
    sheetData.push([
      `7. Chart Title: ${this.groupId} - ${timeRangeTitle}`,
      ...Array(columnCount - 1).fill(""),
    ]);
  }

  private configureWorksheet(worksheet: XLSX.WorkSheet, columnCount: number): void {
    // Merge title cells
    if (!worksheet["!merges"]) worksheet["!merges"] = [];
    worksheet["!merges"].push({
      s: { r: 0, c: 0 },
      e: { r: 0, c: columnCount - 1 },
    });

    // Set column widths
    const colWidths = [
      { wch: 12 }, // Timestamp
      ...this.allSheetData.map(() => ({ wch: 12 })),
      { wch: 40 }, // Comments
    ];
    worksheet["!cols"] = colWidths;
  }
}

/**
 * Generates chart images using uPlot
 */
class ChartImageGenerator {
  static async generate(
    allSheetData: CombinedSheetData[],
    groupId: string,
    timeRangeTitle: string,
    sortedTimestamps: number[],
    startTime: number
  ): Promise<string | null> {
    let container: HTMLDivElement | null = null;
    let plot: uPlot | null = null;

    try {
      container = this.createOffScreenContainer();
      document.body.appendChild(container);

      const chartData = this.prepareChartData(
        allSheetData,
        sortedTimestamps,
        startTime
      );
      const series = this.buildSeriesConfig(allSheetData);
      const opts = this.buildPlotOptions(groupId, timeRangeTitle, series);

      plot = new uPlot(opts, chartData as uPlot.AlignedData, container);

      await this.waitForRender(container);

      const canvas = container.querySelector("canvas");
      if (!canvas) return null;

      const imageData = canvas.toDataURL("image/png");
      return imageData.split(",")[1];
    } catch (error) {
      console.error("Error generating chart image:", error);
      return null;
    } finally {
      if (plot) plot.destroy();
      if (container && document.body.contains(container)) {
        document.body.removeChild(container);
      }
    }
  }

  private static createOffScreenContainer(): HTMLDivElement {
    const container = document.createElement("div");
    container.style.width = "1200px";
    container.style.height = "600px";
    container.style.position = "absolute";
    container.style.left = "-9999px";
    return container;
  }

  private static prepareChartData(
    allSheetData: CombinedSheetData[],
    sortedTimestamps: number[],
    startTime: number
  ): number[][] {
    const chartData: number[][] = [
      sortedTimestamps.map((ts) => (ts - startTime) / 1000),
    ];

    const timestampIndexMap = new Map<number, number>();
    sortedTimestamps.forEach((ts, idx) => {
      timestampIndexMap.set(ts, idx);
    });

    allSheetData.forEach((sheetData) => {
      const values = new Array(sortedTimestamps.length).fill(null);

      sheetData.timestamps.forEach((ts, idx) => {
        const timeIndex = timestampIndexMap.get(ts);
        if (timeIndex !== undefined) {
          values[timeIndex] = sheetData.values[idx];
        }
      });

      chartData.push(values);
    });

    return chartData;
  }

  private static buildSeriesConfig(
    allSheetData: CombinedSheetData[]
  ): uPlot.Series[] {
    const series: uPlot.Series[] = [{ label: "Time (s)" }];

    allSheetData.forEach((sheetData) => {
      const color = sheetData.color || "#9b59b6";
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

    return series;
  }

  private static buildPlotOptions(
    groupId: string,
    timeRangeTitle: string,
    series: uPlot.Series[]
  ): uPlot.Options {
    return {
      title: `${groupId} - ${timeRangeTitle}`,
      width: 1200,
      height: 600,
      series,
      scales: {
        x: { time: false },
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
      legend: { show: false },
      cursor: { show: false },
    };
  }

  private static async waitForRender(container: HTMLDivElement): Promise<void> {
    return new Promise<void>((resolve) => {
      const checkCanvas = () => {
        const canvas = container.querySelector("canvas");
        if (canvas && canvas.width > 0 && canvas.height > 0) {
          requestAnimationFrame(() => resolve());
        } else {
          requestAnimationFrame(checkCanvas);
        }
      };
      checkCanvas();

      // Fallback timeout
      setTimeout(() => resolve(), 500);
    });
  }
}

/**
 * Generates legend images for charts
 */
class LegendImageGenerator {
  static generate(allSheetData: CombinedSheetData[]): string | null {
    try {
      const dimensions = this.calculateDimensions(allSheetData);
      const canvas = this.createCanvas(dimensions.width, dimensions.height);
      const ctx = canvas.getContext("2d");

      if (!ctx) return null;

      this.drawBackground(ctx, dimensions.width, dimensions.height);
      this.drawLegendItems(ctx, allSheetData, dimensions);

      const imageData = canvas.toDataURL("image/png");
      return imageData.split(",")[1];
    } catch (error) {
      console.error("Error generating legend image:", error);
      return null;
    }
  }

  private static calculateDimensions(allSheetData: CombinedSheetData[]): {
    width: number;
    height: number;
    rows: number;
  } {
    const canvasWidth = 1200;
    const itemSpacing = 15;
    const rowHeight = 20;
    const topPadding = 5;
    const bottomPadding = 5;
    const maxItemWidth = 1100;
    const itemStartX = 20;

    const tempCanvas = document.createElement("canvas");
    const tempCtx = tempCanvas.getContext("2d");
    if (!tempCtx) return { width: canvasWidth, height: 50, rows: 1 };

    tempCtx.font = "12px sans-serif";

    let legendX = itemStartX;
    let rowCount = 1;

    allSheetData.forEach((sheetData, index) => {
      const textWidth = tempCtx.measureText(sheetData.sheetName).width;
      const itemWidth = 16 + textWidth + itemSpacing;

      if (
        legendX + itemWidth > maxItemWidth &&
        index < allSheetData.length - 1
      ) {
        legendX = itemStartX;
        rowCount++;
      }

      legendX += itemWidth;
    });

    const height =
      topPadding + rowHeight + (rowCount - 1) * rowHeight + bottomPadding;

    return { width: canvasWidth, height, rows: rowCount };
  }

  private static createCanvas(width: number, height: number): HTMLCanvasElement {
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    return canvas;
  }

  private static drawBackground(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number
  ): void {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, width, height);
  }

  private static drawLegendItems(
    ctx: CanvasRenderingContext2D,
    allSheetData: CombinedSheetData[],
    dimensions: { width: number; height: number; rows: number }
  ): void {
    const itemSpacing = 15;
    const rowHeight = 20;
    const topPadding = 5;
    const maxItemWidth = 1100;
    const itemStartX = 20;

    ctx.font = "12px sans-serif";
    ctx.textAlign = "left";

    let currentX = itemStartX;
    let currentY = topPadding + 15;

    allSheetData.forEach((sheetData, index) => {
      const color = sheetData.color || "#9b59b6";
      const label = sheetData.sheetName;
      const textWidth = ctx.measureText(label).width;
      const itemWidth = 16 + textWidth + itemSpacing;

      if (
        currentX + itemWidth > maxItemWidth &&
        index < allSheetData.length - 1
      ) {
        currentX = itemStartX;
        currentY += rowHeight;
      }

      // Draw color indicator
      ctx.fillStyle = color;
      ctx.fillRect(currentX, currentY - 6, 12, 12);

      // Draw label text
      ctx.fillStyle = "#333";
      ctx.fillText(label, currentX + 16, currentY + 4);

      currentX += itemWidth;
    });
  }
}

/**
 * Main orchestrator for Excel export functionality
 */
export class ExcelExporter {
  private sheetNameManager = new SheetNameManager();

  async export(
    graphDataMap: Map<string, () => GraphExportData | null>,
    groupId: string,
    logs: LogEntry[] = [],
    pidData?: PidData
  ): Promise<void> {
    try {
      const filteredMap = this.filterValidSeries(graphDataMap);
      const workbook = XLSX.utils.book_new();
      const exportTimestamp = DateFormatter.getExportTimestamp();

      const allSheetData: CombinedSheetData[] = [];

      // Process each series
      filteredMap.forEach((getDataFn) => {
        const exportData = getDataFn();
        if (!exportData?.data?.newData) return;

        const series = exportData.data;
        const seriesTitle = series.title || "Series";
        
        // Ensure newData is not null before proceeding
        if (!series.newData) return;

        const sheetName = this.sheetNameManager.generate(
          exportData.config.title,
          seriesTitle,
          exportData.unit
        );

        const targetLines: GraphLine[] = [
          ...(exportData.config.lines || []),
          ...(series.lines || []),
        ];

        // Create data sheet
        const dataSheetBuilder = new DataSheetBuilder(
          {
            graphTitle: exportData.config.title,
            lineTitle: seriesTitle,
            series: series.newData,
            color: series.color,
            unit: exportData.unit,
            renderValue: exportData.renderValue,
            config: exportData.config,
            targetLines,
          },
          seriesTitle,
          exportData.unit
        );

        const worksheet = dataSheetBuilder.build();
        XLSX.utils.book_append_sheet(workbook, worksheet, sheetName);

        // Collect data for analysis sheet
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
      });

      if (allSheetData.length === 0) {
        alert("No data available to export from any graphs in this group");
        return;
      }

      // Create analysis sheet
      const analysisSheetBuilder = new AnalysisSheetBuilder(
        allSheetData,
        groupId,
        logs,
        pidData
      );
      const analysisSheet = await analysisSheetBuilder.build();
      XLSX.utils.book_append_sheet(workbook, analysisSheet, "Analysis");

      // Convert to ExcelJS for image support
      await this.addChartImages(workbook, allSheetData, groupId);
    } catch (error) {
      alert(
        `Error exporting data to Excel: ${
          error instanceof Error ? error.message : "Unknown error"
        }. Please try again.`
      );
    }
  }

  private filterValidSeries(
    graphDataMap: Map<string, () => GraphExportData | null>
  ): Map<string, () => GraphExportData | null> {
    const filteredMap = new Map<string, () => GraphExportData | null>();
    graphDataMap.forEach((getDataFn, seriesId) => {
      if (seriesId.includes("-series-")) {
        filteredMap.set(seriesId, getDataFn);
      }
    });
    return filteredMap;
  }

  private async addChartImages(
    workbook: XLSX.WorkBook,
    allSheetData: CombinedSheetData[],
    groupId: string
  ): Promise<void> {
    const xlsxBuffer = XLSX.write(workbook, {
      type: "buffer",
      bookType: "xlsx",
    });

    let excelJSWorkbook: ExcelJS.Workbook;
    try {
      excelJSWorkbook = new ExcelJS.Workbook();
      await excelJSWorkbook.xlsx.load(xlsxBuffer);
    } catch (error) {
      console.error("Failed to load XLSX buffer into ExcelJS", error);
      alert(
        "Export failed while preparing the Excel file. The generated workbook data was invalid or could not be processed."
      );
      return;
    }

    const analysisWorksheet = excelJSWorkbook.getWorksheet("Analysis");
    if (!analysisWorksheet) return;

    const sortedTimestamps = Array.from(
      new Set(allSheetData.flatMap((d) => d.timestamps))
    ).sort((a, b) => a - b);

    const startTime = sortedTimestamps[0];
    const endTime = sortedTimestamps[sortedTimestamps.length - 1];
    const timeRangeTitle = DateFormatter.formatTimeRange(startTime, endTime);

    const chartImage = await ChartImageGenerator.generate(
      allSheetData,
      groupId,
      timeRangeTitle,
      sortedTimestamps,
      startTime
    );

    if (chartImage) {
      const chartImageId = excelJSWorkbook.addImage({
        base64: chartImage,
        extension: "png",
      });

      const lastRow = analysisWorksheet.rowCount;
      analysisWorksheet.addImage(chartImageId, {
        tl: { col: 0, row: lastRow + 2 },
        ext: { width: 1200, height: 600 },
      });

      const legendImage = LegendImageGenerator.generate(allSheetData);
      if (legendImage) {
        const legendImageId = excelJSWorkbook.addImage({
          base64: legendImage,
          extension: "png",
        });

        analysisWorksheet.addImage(legendImageId, {
          tl: { col: 0, row: lastRow + 2 + 32 },
          ext: { width: 1200, height: 50 },
        });
      }
    }

    // Write final file with ExcelJS
    const buffer = await excelJSWorkbook.xlsx.writeBuffer();
    this.triggerDownload(buffer, groupId, DateFormatter.getExportTimestamp());
  }

  private triggerDownload(
    buffer: ArrayBuffer,
    groupId: string,
    exportTimestamp: string
  ): void {
    const filename = `${groupId
      .toLowerCase()
      .replace(/\s+/g, "_")}_export_${exportTimestamp}.xlsx`;

    const blob = new Blob([buffer], {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    link.click();
    window.URL.revokeObjectURL(url);
  }
}

/**
 * Convenience function to maintain backward compatibility with existing code
 */
export async function exportGraphsToExcel(
  graphDataMap: Map<string, () => GraphExportData | null>,
  groupId: string,
  logs: LogEntry[] = [],
  pidData?: PidData
): Promise<void> {
  const exporter = new ExcelExporter();
  await exporter.export(graphDataMap, groupId, logs, pidData);
}
