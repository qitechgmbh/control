import * as XLSX from "xlsx";
import { LogEntry } from "@/stores/logsStore";
import { ExportConfig, IExportConfig } from "./excelExportConfig";
import {
  IValueFormatter,
  ValueFormatter,
  TimestampConverter,
  ArrayUtils,
} from "./excelFormatters";
import { MetadataProviderFactory } from "./excelMetadata";
import { ChartAxisCalculator, ExcelCellSanitizer } from "./excelUtils";
import { CommentManager } from "./excelCommentManager";
import { CombinedSheetData, PidData } from "./excelExportTypes";

/**
 * Creates the combined analysis sheet with all series data
 */
export class AnalysisSheetBuilder {
  private config: IExportConfig;
  private formatter: IValueFormatter;

  constructor(
    private allSheetData: CombinedSheetData[],
    private groupId: string,
    private logs: LogEntry[],
    private pidData?: PidData,
    config?: IExportConfig,
    formatter?: IValueFormatter,
  ) {
    this.config = config || new ExportConfig();
    this.formatter = formatter || new ValueFormatter();
  }

  async build(): Promise<XLSX.WorkSheet> {
    // Get sorted timestamps
    const sortedTimestamps = this.getSortedTimestamps();
    const startTime = sortedTimestamps[0];
    const endTime = sortedTimestamps[sortedTimestamps.length - 1];

    // Filter relevant comments
    const relevantComments = CommentManager.filterRelevant(
      this.logs,
      startTime,
      endTime,
    );

    // Build data by timestamp map
    const dataByTimestamp = this.buildDataByTimestampMap();

    // Build columns
    const columns = this.buildColumns();

    // Create sheet data array
    const sheetData: any[][] = [];

    // Add title row
    const timeRangeTitle = this.formatter.formatTimeRange(startTime, endTime);
    sheetData.push(
      ArrayUtils.createRow(
        [`${this.groupId} - ${timeRangeTitle}`],
        columns.length,
      ),
    );
    sheetData.push(ArrayUtils.createEmptyArray(columns.length));

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
        relevantComments,
      );

      const secondsFromStart = TimestampConverter.toSecondsFromStart(
        timestamp,
        startTime,
      );
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
      timeRangeTitle,
    );

    const sanitizedSheetData = sheetData.map((row) =>
      ExcelCellSanitizer.sanitizeRow(row),
    );

    // Convert to worksheet
    const worksheet = XLSX.utils.aoa_to_sheet(sanitizedSheetData);

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
          (line) => line.type === "target",
        );
        if (targetLine) {
          targetValues.push(this.formatter.formatNumber(targetLine.value));
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
      sheetData.push(ArrayUtils.createEmptyArray(columnCount));
    }
  }

  private buildDataRow(
    timestamp: number,
    startTime: number,
    dataByTimestamp: Map<number, Map<string, number>>,
    relevantComments: LogEntry[],
  ): any[] {
    const row: any[] = [];

    // Calculate seconds from start using TimestampConverter
    const secondsFromStart = TimestampConverter.toSecondsFromStart(
      timestamp,
      startTime,
    );
    row.push(secondsFromStart);

    // Add data for each column
    this.allSheetData.forEach((sheetDataEntry) => {
      const tsData = dataByTimestamp.get(timestamp);
      if (tsData && tsData.has(sheetDataEntry.sheetName)) {
        row.push(
          this.sanitizeNumber(tsData.get(sheetDataEntry.sheetName) ?? NaN),
        );
      } else {
        row.push("");
      }
    });

    // Check for comments at this timestamp
    const comment = CommentManager.findAtTimestamp(relevantComments, timestamp);
    row.push(comment ? comment.message : "");

    return row;
  }

  private sanitizeNumber(value: number): number | "" {
    return Number.isFinite(value) ? value : "";
  }

  private async addMetadata(
    sheetData: any[][],
    columnCount: number,
    relevantComments: LogEntry[],
  ): Promise<void> {
    sheetData.push(ArrayUtils.createEmptyArray(columnCount));
    sheetData.push(ArrayUtils.createEmptyArray(columnCount));

    // Use MetadataProvider to build metadata
    const metadataProvider = MetadataProviderFactory.createForExport({
      softwareName: this.config.getSoftwareName(),
      exportDate: this.formatter.formatDate(new Date()),
      pidData: this.pidData,
      commentCount: relevantComments.length,
    });

    sheetData.push(...metadataProvider.buildRows(columnCount));
  }

  private addChartInstructions(
    sheetData: any[][],
    columnCount: number,
    dataStartRow: number,
    maxSeconds: number,
    timeRangeTitle: string,
  ): void {
    // Calculate optimal Y-axis range from all data
    const allValues = this.allSheetData.flatMap((sheet) => sheet.values);
    const yAxisRange = ChartAxisCalculator.calculateOptimalRange(allValues);
    const yAxisInstruction = ChartAxisCalculator.formatRangeInstruction(
      yAxisRange.min,
      yAxisRange.max,
    );

    sheetData.push(ArrayUtils.createEmptyArray(columnCount));
    sheetData.push(ArrayUtils.createEmptyArray(columnCount));
    sheetData.push(ArrayUtils.createRow(["Chart Instructions"], columnCount));
    sheetData.push(
      ArrayUtils.createRow(
        [`1. Select all data from row ${dataStartRow} to the last data row`],
        columnCount,
      ),
    );
    sheetData.push(
      ArrayUtils.createRow(
        ["2. Insert > Chart > Scatter Chart with Straight Lines and Markers"],
        columnCount,
      ),
    );
    sheetData.push(
      ArrayUtils.createRow(
        ["3. X-axis: Time (seconds), Y-axis: All measurement columns"],
        columnCount,
      ),
    );
    sheetData.push(
      ArrayUtils.createRow(
        [`4. Set X-axis range: 0 to ${maxSeconds}`],
        columnCount,
      ),
    );
    sheetData.push(ArrayUtils.createRow([yAxisInstruction], columnCount));
    sheetData.push(
      ArrayUtils.createRow(["6. Position legend at bottom"], columnCount),
    );
    sheetData.push(
      ArrayUtils.createRow(
        [`7. Chart Title: ${this.groupId} - ${timeRangeTitle}`],
        columnCount,
      ),
    );
  }

  private configureWorksheet(
    worksheet: XLSX.WorkSheet,
    columnCount: number,
  ): void {
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
