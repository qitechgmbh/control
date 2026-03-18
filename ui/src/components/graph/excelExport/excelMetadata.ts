/**
 * Metadata provider system following Open/Closed Principle
 * New metadata sections can be added without modifying existing code
 */

export interface IMetadataSection {
  getTitle(): string;
  getRows(): Array<{ key: string; value: string }>;
}

/**
 * Base metadata section implementation
 */
export abstract class MetadataSection implements IMetadataSection {
  abstract getTitle(): string;
  abstract getRows(): Array<{ key: string; value: string }>;
}

/**
 * Export information section
 */
export class ExportInfoSection extends MetadataSection {
  constructor(
    private softwareName: string,
    private exportDate: string,
  ) {
    super();
  }

  getTitle(): string {
    return "Export Information";
  }

  getRows(): Array<{ key: string; value: string }> {
    return [
      { key: "Software", value: this.softwareName },
      { key: "Export Date", value: this.exportDate },
    ];
  }
}

/**
 * PID settings section for temperature controllers
 */
export class TemperaturePidSection extends MetadataSection {
  constructor(
    private pidSettings: Record<string, { kp: number; ki: number; kd: number }>,
  ) {
    super();
  }

  getTitle(): string {
    return "Temperature Controllers";
  }

  getRows(): Array<{ key: string; value: string }> {
    const rows: Array<{ key: string; value: string }> = [];

    Object.entries(this.pidSettings).forEach(([zone, settings]) => {
      rows.push(
        { key: `  ${zone} - Kp`, value: settings.kp.toFixed(3) },
        { key: `  ${zone} - Ki`, value: settings.ki.toFixed(3) },
        { key: `  ${zone} - Kd`, value: settings.kd.toFixed(3) },
      );
    });

    return rows;
  }
}

/**
 * PID settings section for pressure controller
 */
export class PressurePidSection extends MetadataSection {
  constructor(private pidSettings: { kp: number; ki: number; kd: number }) {
    super();
  }

  getTitle(): string {
    return "Pressure Controller";
  }

  getRows(): Array<{ key: string; value: string }> {
    return [
      { key: "  Kp", value: this.pidSettings.kp.toFixed(3) },
      { key: "  Ki", value: this.pidSettings.ki.toFixed(3) },
      { key: "  Kd", value: this.pidSettings.kd.toFixed(3) },
    ];
  }
}

/**
 * Comment statistics section
 */
export class CommentStatsSection extends MetadataSection {
  constructor(private commentCount: number) {
    super();
  }

  getTitle(): string {
    return "Comment Statistics";
  }

  getRows(): Array<{ key: string; value: string }> {
    return [{ key: "Total Comments", value: this.commentCount.toString() }];
  }
}

/**
 * Metadata provider that aggregates multiple sections
 * Follows Composite Pattern
 */
export class MetadataProvider {
  private sections: IMetadataSection[] = [];

  addSection(section: IMetadataSection): this {
    this.sections.push(section);
    return this;
  }

  getSections(): IMetadataSection[] {
    return this.sections;
  }

  /**
   * Build metadata rows for Excel sheet
   */
  buildRows(columnCount: number): string[][] {
    const rows: string[][] = [];

    this.sections.forEach((section, index) => {
      // Add empty row before each section except the first
      if (index > 0) {
        rows.push(Array(columnCount).fill(""));
      }

      // Add section title
      rows.push([section.getTitle(), ...Array(columnCount - 1).fill("")]);

      // Add section rows
      section.getRows().forEach((row) => {
        rows.push([row.key, row.value, ...Array(columnCount - 2).fill("")]);
      });
    });

    return rows;
  }
}

/**
 * Factory for creating metadata providers
 */
export class MetadataProviderFactory {
  static createForExport(params: {
    softwareName: string;
    exportDate: string;
    pidData?: {
      temperature?: Record<string, { kp: number; ki: number; kd: number }>;
      pressure?: { kp: number; ki: number; kd: number };
    };
    commentCount?: number;
  }): MetadataProvider {
    const provider = new MetadataProvider();

    // Always add export info
    provider.addSection(
      new ExportInfoSection(params.softwareName, params.exportDate),
    );

    // Add PID sections if available
    if (params.pidData?.temperature) {
      provider.addSection(
        new TemperaturePidSection(params.pidData.temperature),
      );
    }

    if (params.pidData?.pressure) {
      provider.addSection(new PressurePidSection(params.pidData.pressure));
    }

    // Add comment stats if provided
    if (params.commentCount !== undefined) {
      provider.addSection(new CommentStatsSection(params.commentCount));
    }

    return provider;
  }
}
