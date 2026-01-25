/**
 * Value formatting utilities following Strategy Pattern
 * Centralizes all formatting logic to eliminate duplication
 */

export interface IValueFormatter {
  formatNumber(value: number): string;
  formatDate(date: Date): string;
  formatTimeRange(startTime: number, endTime: number): string;
  formatDuration(milliseconds: number): string;
}

/**
 * Default formatter implementation
 */
export class ValueFormatter implements IValueFormatter {
  constructor(
    private precision: number = 3,
    private locale: string = "de-DE",
  ) {}

  formatNumber(value: number): string {
    if (value == null || isNaN(value)) {
      return "";
    }
    return value.toFixed(this.precision);
  }

  formatDate(date: Date): string {
    return date.toLocaleString(this.locale, {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  formatTimeRange(startTime: number, endTime: number): string {
    const startDate = new Date(startTime);
    const endDate = new Date(endTime);
    return `${this.formatDate(startDate)} bis ${this.formatDate(endDate)}`;
  }

  formatDuration(milliseconds: number): string {
    const hours = (milliseconds / (1000 * 60 * 60)).toFixed(2);
    return hours;
  }

  setPrecision(precision: number): void {
    this.precision = precision;
  }

  setLocale(locale: string): void {
    this.locale = locale;
  }
}

/**
 * Custom formatter that can use a render function
 */
export class CustomValueFormatter extends ValueFormatter {
  constructor(
    private customRenderFn?: (value: number) => string,
    precision?: number,
    locale?: string,
  ) {
    super(precision, locale);
  }

  override formatNumber(value: number): string {
    if (this.customRenderFn) {
      return this.customRenderFn(value);
    }
    return super.formatNumber(value);
  }
}

/**
 * Timestamp utilities to eliminate duplicate conversion logic
 */
export class TimestampConverter {
  /**
   * Convert timestamp to seconds from start
   */
  static toSecondsFromStart(timestamp: number, startTime: number): number {
    return Math.floor((timestamp - startTime) / 1000);
  }

  /**
   * Convert multiple timestamps to seconds from start
   */
  static arrayToSecondsFromStart(
    timestamps: number[],
    startTime: number,
  ): number[] {
    return timestamps.map((ts) => this.toSecondsFromStart(ts, startTime));
  }
}

/**
 * Array utilities to eliminate duplicate fill patterns
 */
export class ArrayUtils {
  /**
   * Create an array filled with empty strings
   */
  static createEmptyArray(count: number): string[] {
    return Array(count).fill("");
  }

  /**
   * Create a row with initial values and fill rest with empty strings
   */
  static createRow(initialValues: any[], totalColumns: number): any[] {
    const emptyCount = Math.max(0, totalColumns - initialValues.length);
    return [...initialValues, ...this.createEmptyArray(emptyCount)];
  }
}
