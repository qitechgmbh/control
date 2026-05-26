import * as XLSX from "xlsx";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import { GraphConfig, GraphLine } from "../types";
import { IValueFormatter, ValueFormatter } from "./excelFormatters";
import { StatisticsCalculator } from "./excelStatisticsCalculator";
import { ExcelCellSanitizer } from "./excelUtils";

/**
 * Creates individual data sheets for each series
 */
export class DataSheetBuilder {
  private formatter: IValueFormatter;

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
    private unit: Unit | undefined,
    formatter?: IValueFormatter,
  ) {
    this.formatter = formatter || new ValueFormatter();
  }

  build(): XLSX.WorkSheet {
    const [timestamps, values] = seriesToUPlotData(this.graphLine.series.long);
    const unitSymbol = renderUnitSymbol(this.unit) || "";

    const sheetData: any[][] = [];

    // Build header
    const col1Header = unitSymbol
      ? `${unitSymbol} ${this.seriesTitle}`
      : this.seriesTitle;

    sheetData.push(["Timestamp", col1Header, "", "", "Statistic", "Value"]);

    // Build stats section
    const statsRows = this.buildStatsRows(timestamps, values, unitSymbol);

    // Combine data and stats rows
    const maxRows = Math.max(timestamps.length, statsRows.length);
    for (let i = 0; i < maxRows; i++) {
      const row = this.buildDataRow(i, timestamps, values, statsRows);
      sheetData.push(row);
    }

    const sanitizedSheetData = sheetData.map((row) =>
      ExcelCellSanitizer.sanitizeRow(row),
    );

    // Convert to worksheet
    const worksheet = XLSX.utils.aoa_to_sheet(sanitizedSheetData);
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
    unitSymbol: string,
  ): string[][] {
    const statsRows: string[][] = [];

    statsRows.push(["Graph", this.graphLine.graphTitle]);
    statsRows.push(["Line Name", this.graphLine.lineTitle]);
    statsRows.push(["Line Color", this.graphLine.color || "Default"]);
    statsRows.push(["Generated", this.formatter.formatDate(new Date())]);
    statsRows.push(["", ""]);
    statsRows.push(["Total Data Points", timestamps.length.toString()]);

    if (timestamps.length > 0) {
      const firstDate = new Date(timestamps[0]);
      const lastDate = new Date(timestamps[timestamps.length - 1]);

      statsRows.push([
        "Time Range Start",
        this.formatter.formatDate(firstDate),
      ]);
      statsRows.push(["Time Range End", this.formatter.formatDate(lastDate)]);

      const duration = timestamps[timestamps.length - 1] - timestamps[0];
      const durationHours = this.formatter.formatDuration(duration);
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
    statsRows: string[][],
  ): any[] {
    const row: any[] = ["", "", "", ""];

    // Add timestamp and value data
    if (index < timestamps.length) {
      const date = new Date(timestamps[index]);
      row[0] = this.formatter.formatDate(date);
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
      : this.formatter.formatNumber(value);
  }
}
