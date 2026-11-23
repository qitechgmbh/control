import * as XLSX from "xlsx";
/**
 * Additional utility classes for Excel export
 * Following SOLID principles and DRY
 */

/**
 * Utility for calculating optimal Y-axis range based on data
 * Can be extended for more sophisticated scaling algorithms
 */
export class ChartAxisCalculator {
  /**
   * Calculate optimal Y-axis range with padding
   */
  static calculateOptimalRange(
    values: number[],
    paddingPercent: number = 10,
  ): { min: number; max: number } {
    if (values.length === 0) {
      return { min: 0, max: 1000 }; // fallback
    }

    const dataMin = Math.min(...values);
    const dataMax = Math.max(...values);
    const range = dataMax - dataMin;

    // Add padding
    const padding = range * (paddingPercent / 100);

    return {
      min: Math.floor(dataMin - padding),
      max: Math.ceil(dataMax + padding),
    };
  }

  /**
   * Format Y-axis range instruction for Excel
   */
  static formatRangeInstruction(min: number, max: number): string {
    return `5. Set Y-axis range: ${min} to ${max}`;
  }
}

/**
 * Sanitizes cell values to avoid invalid XML in XLSX exports.
 */
export class ExcelCellSanitizer {
  /* eslint-disable no-control-regex */
  private static invalidXmlChars =
    /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F\uD800-\uDFFF\uFFFE\uFFFF]/g;
  /* eslint-enable no-control-regex */

  static sanitizeCell(value: unknown): string | number {
    if (typeof value === "number") {
      return Number.isFinite(value) ? value : "";
    }

    if (typeof value === "boolean") {
      return value ? 1 : 0;
    }

    if (value instanceof Date) {
      return value.toISOString();
    }

    if (typeof value === "string") {
      return value.replace(this.invalidXmlChars, "");
    }

    if (value === null || value === undefined) {
      return "";
    }

    return String(value).replace(this.invalidXmlChars, "");
  }

  static sanitizeRow(row: unknown[]): Array<string | number> {
    return row.map((value) => this.sanitizeCell(value));
  }

  static sanitizeWorksheet(worksheet: XLSX.WorkSheet): void {
    Object.keys(worksheet).forEach((key) => {
      if (key.startsWith("!")) return;
      const cell = worksheet[key] as XLSX.CellObject | undefined;
      if (!cell) return;

      const sanitized = this.sanitizeCell(cell.v);
      if (sanitized === "") {
        delete worksheet[key];
        return;
      }

      cell.v = sanitized as any;
    });
  }
}
