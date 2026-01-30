import * as XLSX from "xlsx";
import ExcelJS from "exceljs";
import { seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol } from "@/control/units";
import { GraphLine } from "../types";
import { LogEntry } from "@/stores/logsStore";
import {
  ExportConfig,
  IExportConfig,
  WindowEnvironmentProvider,
} from "./excelExportConfig";
import { IValueFormatter, ValueFormatter } from "./excelFormatters";
import { IPidDataProvider } from "./excelUtils";
import {
  CombinedSheetData,
  GraphExportData,
  PidData,
} from "./excelExportTypes";
import { DateFormatter } from "./excelDateFormatter";
import { SheetNameManager } from "./excelSheetNameManager";
import { DataSheetBuilder } from "./excelDataSheetBuilder";
import { AnalysisSheetBuilder } from "./excelAnalysisSheetBuilder";
import { VersionInfoRenderer } from "./excelVersionInfoRenderer";
import { ChartImageGenerator } from "./excelChartImageGenerator";
import { LegendImageGenerator } from "./excelLegendImageGenerator";

export type { GraphExportData, PidSettings, PidData } from "./excelExportTypes";

/**
 * Main orchestrator for Excel export functionality
 */
export class ExcelExporter {
  private config: IExportConfig;
  private formatter: IValueFormatter;
  private sheetNameManager: SheetNameManager;
  private pidDataProvider?: IPidDataProvider;

  constructor(
    config?: IExportConfig,
    formatter?: IValueFormatter,
    pidDataProvider?: IPidDataProvider,
  ) {
    this.config = config || new ExportConfig();
    this.formatter = formatter || new ValueFormatter();
    this.sheetNameManager = new SheetNameManager(this.config);
    this.pidDataProvider = pidDataProvider;
  }

  async export(
    graphDataMap: Map<string, () => GraphExportData | null>,
    groupId: string,
    logs: LogEntry[] = [],
    pidData?: PidData,
  ): Promise<void> {
    try {
      // If PID data provider is available and no PID data provided, fetch it
      if (!pidData && this.pidDataProvider) {
        pidData = (await this.pidDataProvider.fetchPidSettings()) || undefined;
      }

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
          exportData.unit,
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
          exportData.unit,
          this.formatter,
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
        pidData,
        this.config,
        this.formatter,
      );
      const analysisSheet = await analysisSheetBuilder.build();
      XLSX.utils.book_append_sheet(workbook, analysisSheet, "Analysis");

      // Convert to ExcelJS for image support
      await this.addChartImages(workbook, allSheetData, groupId);
    } catch (error) {
      alert(
        `Error exporting data to Excel: ${
          error instanceof Error ? error.message : "Unknown error"
        }. Please try again.`,
      );
    }
  }

  private filterValidSeries(
    graphDataMap: Map<string, () => GraphExportData | null>,
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
    groupId: string,
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
        "Export failed while preparing the Excel file. The generated workbook data was invalid or could not be processed.",
      );
      return;
    }

    const analysisWorksheet = excelJSWorkbook.getWorksheet("Analysis");
    if (!analysisWorksheet) return;

    const sortedTimestamps = Array.from(
      new Set(allSheetData.flatMap((d) => d.timestamps)),
    ).sort((a, b) => a - b);

    const startTime = sortedTimestamps[0];
    const endTime = sortedTimestamps[sortedTimestamps.length - 1];
    const timeRangeTitle = this.formatter.formatTimeRange(startTime, endTime);

    // Fetch version info for chart rendering
    const envProvider = new WindowEnvironmentProvider();
    const versionRenderer = new VersionInfoRenderer(envProvider);
    await versionRenderer.fetchVersionInfo();

    const chartImage = await ChartImageGenerator.generate(
      allSheetData,
      groupId,
      timeRangeTitle,
      sortedTimestamps,
      startTime,
      versionRenderer,
      this.config,
    );

    if (chartImage) {
      const dimensions = this.config.getChartDimensions();

      const chartImageId = excelJSWorkbook.addImage({
        base64: chartImage,
        extension: "png",
      });

      const lastRow = analysisWorksheet.rowCount;
      analysisWorksheet.addImage(chartImageId, {
        tl: { col: 0, row: lastRow + 2 },
        ext: { width: dimensions.width, height: dimensions.height },
      });

      const legendImage = LegendImageGenerator.generate(
        allSheetData,
        this.config,
      );
      if (legendImage) {
        const legendImageId = excelJSWorkbook.addImage({
          base64: legendImage,
          extension: "png",
        });

        analysisWorksheet.addImage(legendImageId, {
          tl: { col: 0, row: lastRow + 2 + 32 },
          ext: { width: dimensions.width, height: 50 },
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
    exportTimestamp: string,
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
  pidData?: PidData,
): Promise<void> {
  const exporter = new ExcelExporter();
  await exporter.export(graphDataMap, groupId, logs, pidData);
}
