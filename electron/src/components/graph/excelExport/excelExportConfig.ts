/**
 * Configuration system for Excel Export
 * Centralizes all hardcoded values and provides type-safe access
 * Follows Dependency Inversion Principle - depends on abstractions
 */

export interface IExportConfig {
  getSoftwareName(): string;
  getDefaultPrecision(): number;
  getUnitFriendlyName(unit: string): string | undefined;
  getDefaultChartColor(): string;
  getDateLocale(): string;
  getYAxisRange(): { min: number; max: number } | null; // null means auto-scale
}

/**
 * Default implementation that can be extended or replaced
 */
export class ExportConfig implements IExportConfig {
  private readonly config = {
    softwareName: "QiTech Control",
    defaultPrecision: 3,
    unitFriendlyNames: {
      "°C": "Temp",
      W: "Watt",
      A: "Ampere",
      bar: "Bar",
      rpm: "Rpm",
      "1/min": "Rpm",
      mm: "mm",
      "%": "Percent",
    } as Record<string, string>,
    defaultChartColor: "#9b59b6",
    dateLocale: "de-DE",
    // null means auto-scale based on data
    yAxisRange: null as { min: number; max: number } | null,
  };

  getSoftwareName(): string {
    return this.config.softwareName;
  }

  getDefaultPrecision(): number {
    return this.config.defaultPrecision;
  }

  getUnitFriendlyName(unit: string): string | undefined {
    return this.config.unitFriendlyNames[unit];
  }

  getDefaultChartColor(): string {
    return this.config.defaultChartColor;
  }

  getDateLocale(): string {
    return this.config.dateLocale;
  }

  getYAxisRange(): { min: number; max: number } | null {
    return this.config.yAxisRange;
  }

  /**
   * Allow runtime configuration updates
   */
  setYAxisRange(min: number, max: number): void {
    this.config.yAxisRange = { min, max };
  }

  /**
   * Add new unit mapping at runtime
   */
  addUnitFriendlyName(unit: string, friendlyName: string): void {
    this.config.unitFriendlyNames[unit] = friendlyName;
  }
}

/**
 * Machine-aware configuration that fetches data from machine context
 * This can be extended to fetch PID settings, machine-specific units, etc.
 */
export class MachineAwareExportConfig extends ExportConfig {
  constructor(private machineContext?: any) {
    super();
  }

  /**
   * Override to get software name from machine context if available
   */
  override getSoftwareName(): string {
    // Could fetch from machine context in future
    return this.machineContext?.softwareName ?? super.getSoftwareName();
  }
}

/**
 * Factory for creating configuration instances
 * Follows Abstract Factory Pattern
 */
export class ExportConfigFactory {
  static create(machineContext?: any): IExportConfig {
    if (machineContext) {
      return new MachineAwareExportConfig(machineContext);
    }
    return new ExportConfig();
  }
}
