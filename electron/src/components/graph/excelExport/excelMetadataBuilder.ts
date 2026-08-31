import { IExportConfig } from "./excelExportConfig";
import { IValueFormatter } from "./excelFormatters";
import { MetadataProvider, MetadataProviderFactory } from "./excelMetadata";
import { PidData } from "./excelExportTypes";

/**
 * Builds metadata sections for Excel sheets
 * Now uses MetadataProvider for better separation of concerns
 * @deprecated Use MetadataProvider directly for new code
 */
export class MetadataBuilder {
  private metadataProvider: MetadataProvider;
  private config: IExportConfig;
  private formatter: IValueFormatter;

  constructor(config: IExportConfig, formatter: IValueFormatter) {
    this.config = config;
    this.formatter = formatter;
    this.metadataProvider = new MetadataProvider();
  }

  addExportInfo(columnCount: number): this {
    // This method is kept for backward compatibility
    // Actual data is added when building final rows
    return this;
  }

  addPidSettings(pidData: PidData | undefined, columnCount: number): this {
    // This method is kept for backward compatibility
    // Actual data is added when building final rows
    return this;
  }

  /**
   * Build metadata rows using MetadataProvider
   */
  getRows(
    columnCount: number,
    pidData?: PidData,
    commentCount?: number,
  ): string[][] {
    // Create metadata provider with all sections
    const provider = MetadataProviderFactory.createForExport({
      softwareName: this.config.getSoftwareName(),
      exportDate: this.formatter.formatDate(new Date()),
      pidData: pidData,
      commentCount: commentCount,
    });

    return provider.buildRows(columnCount);
  }
}
