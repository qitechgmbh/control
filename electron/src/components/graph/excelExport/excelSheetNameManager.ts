import { renderUnitSymbol, Unit } from "@/control/units";
import { IExportConfig } from "./excelExportConfig";

/**
 * Manages unique sheet name generation for Excel workbooks
 * Now uses IExportConfig for unit mappings
 */
export class SheetNameManager {
  private usedNames = new Set<string>();

  constructor(private config: IExportConfig) {}

  generate(
    graphTitle: string,
    seriesTitle: string,
    unit: Unit | undefined,
  ): string {
    const unitSymbol = renderUnitSymbol(unit) || "";
    let sheetName = "";

    // Use unit-based name for generic series, otherwise use series title
    if (/^Series \d+$/i.test(seriesTitle)) {
      const friendlyUnitName = this.config.getUnitFriendlyName(unitSymbol);
      sheetName = friendlyUnitName || seriesTitle;
    } else {
      const friendlyUnitName = this.config.getUnitFriendlyName(unitSymbol);
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
    return (
      name
        .replace(/[\\/?*$:[\]]/g, "_")
        .substring(0, 31)
        .trim() || "Sheet"
    );
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
