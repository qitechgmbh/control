import * as XLSX from "xlsx";
import { seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol } from "@/control/units";
import { GraphLine } from "../types";
import { LogEntry } from "@/stores/logsStore";
import { ExportConfig, IExportConfig } from "./exportConfig";
import { IValueFormatter, ValueFormatter } from "./formatters";
import { ExcelCellSanitizer } from "./utils";
import { CombinedSheetData, GraphExportData, PidData } from "./types";
import { DateFormatter } from "./DateFormatter";
import { SheetNameManager } from "./SheetNameManager";
import { DataSheetBuilder } from "./DataSheetBuilder";
import { AnalysisSheetBuilder } from "./AnalysisSheetBuilder";
import { saveFile, SaveFileResult } from "@/helpers/file_export_helpers";

// The renderer runs with contextIsolation enabled, so the Node `Buffer`
// global isn't available here — encode to base64 with browser APIs instead.
function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

export type { GraphExportData, PidSettings, PidData } from "./types";

/**
 * Main orchestrator for Excel export functionality
 */
export class ExcelExporter {
  private config: IExportConfig;
  private formatter: IValueFormatter;
  private sheetNameManager: SheetNameManager;

  constructor(config?: IExportConfig, formatter?: IValueFormatter) {
    this.config = config || new ExportConfig();
    this.formatter = formatter || new ValueFormatter();
    this.sheetNameManager = new SheetNameManager(this.config);
  }

  async export(
    graphDataMap: Map<string, () => GraphExportData | null>,
    groupId: string,
    logs: LogEntry[] = [],
    pidData?: PidData,
  ): Promise<SaveFileResult | undefined> {
    try {
      const filteredMap = this.filterValidSeries(graphDataMap);
      const workbook = XLSX.utils.book_new();

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
        ExcelCellSanitizer.sanitizeWorksheet(worksheet);
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
      ExcelCellSanitizer.sanitizeWorksheet(analysisSheet);
      XLSX.utils.book_append_sheet(workbook, analysisSheet, "Analysis");

      const xlsxBuffer = XLSX.write(workbook, {
        type: "buffer",
        bookType: "xlsx",
      });

      return await this.saveWorkbook(
        xlsxBuffer,
        groupId,
        DateFormatter.getExportTimestamp(),
      );
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

  private async saveWorkbook(
    buffer: Uint8Array,
    groupId: string,
    exportTimestamp: string,
  ): Promise<SaveFileResult> {
    const filename = `${groupId
      .toLowerCase()
      .replace(/\s+/g, "_")}_export_${exportTimestamp}.xlsx`;

    // Errors are surfaced via the returned result (ExportResultDialog),
    // not an alert — the caller already renders that dialog on failure.
    return await saveFile({
      suggestedName: filename,
      filters: [{ name: "Excel Files", extensions: ["xlsx"] }],
      content: bytesToBase64(new Uint8Array(buffer)),
      encoding: "base64",
    });
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
): Promise<SaveFileResult | undefined> {
  const exporter = new ExcelExporter();
  return await exporter.export(graphDataMap, groupId, logs, pidData);
}
