import { ValueFormatter } from "./excelFormatters";

/**
 * Utility class for date/time formatting and manipulation
 * Now uses ValueFormatter internally
 */
export class DateFormatter {
  private static formatter = new ValueFormatter();

  static format(date: Date): string {
    return this.formatter.formatDate(date);
  }

  static getExportTimestamp(): string {
    return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
  }

  static formatTimeRange(startTime: number, endTime: number): string {
    return this.formatter.formatTimeRange(startTime, endTime);
  }
}
