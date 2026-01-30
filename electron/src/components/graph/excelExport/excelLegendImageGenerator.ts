import { ExportConfig, IExportConfig } from "./excelExportConfig";
import { CombinedSheetData } from "./excelExportTypes";

/**
 * Generates legend images for charts
 */
export class LegendImageGenerator {
  static generate(
    allSheetData: CombinedSheetData[],
    config?: IExportConfig,
  ): string | null {
    const exportConfig = config || new ExportConfig();

    try {
      const dimensions = this.calculateDimensions(allSheetData, exportConfig);
      const canvas = this.createCanvas(dimensions.width, dimensions.height);
      const ctx = canvas.getContext("2d");

      if (!ctx) return null;

      this.drawBackground(ctx, dimensions.width, dimensions.height);
      this.drawLegendItems(ctx, allSheetData, dimensions, exportConfig);

      const imageData = canvas.toDataURL("image/png");
      return imageData.split(",")[1];
    } catch (error) {
      console.error("Error generating legend image:", error);
      return null;
    }
  }

  private static calculateDimensions(
    allSheetData: CombinedSheetData[],
    config: IExportConfig,
  ): {
    width: number;
    height: number;
    rows: number;
  } {
    const chartDimensions = config.getChartDimensions();
    const legendDimensions = config.getLegendDimensions();
    const canvasWidth = chartDimensions.width;
    const itemSpacing = legendDimensions.itemSpacing;
    const rowHeight = 20;
    const topPadding = 5;
    const bottomPadding = 5;
    const maxItemWidth = canvasWidth - 100;
    const itemStartX = 20;

    const tempCanvas = document.createElement("canvas");
    const tempCtx = tempCanvas.getContext("2d");
    if (!tempCtx) return { width: canvasWidth, height: 50, rows: 1 };

    tempCtx.font = "12px sans-serif";

    let legendX = itemStartX;
    let rowCount = 1;

    allSheetData.forEach((sheetData, index) => {
      const textWidth = tempCtx.measureText(sheetData.sheetName).width;
      const itemWidth = 16 + textWidth + itemSpacing;

      if (
        legendX + itemWidth > maxItemWidth &&
        index < allSheetData.length - 1
      ) {
        legendX = itemStartX;
        rowCount++;
      }

      legendX += itemWidth;
    });

    const height =
      topPadding + rowHeight + (rowCount - 1) * rowHeight + bottomPadding;

    return { width: canvasWidth, height, rows: rowCount };
  }

  private static createCanvas(
    width: number,
    height: number,
  ): HTMLCanvasElement {
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    return canvas;
  }

  private static drawBackground(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
  ): void {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, width, height);
  }

  private static drawLegendItems(
    ctx: CanvasRenderingContext2D,
    allSheetData: CombinedSheetData[],
    dimensions: { width: number; height: number; rows: number },
    config: IExportConfig,
  ): void {
    const legendDimensions = config.getLegendDimensions();
    const itemSpacing = legendDimensions.itemSpacing;
    const rowHeight = 20;
    const topPadding = 5;
    const maxItemWidth = dimensions.width - 100;
    const itemStartX = 20;

    ctx.font = "12px sans-serif";
    ctx.textAlign = "left";

    let currentX = itemStartX;
    let currentY = topPadding + 15;

    allSheetData.forEach((sheetData, index) => {
      const color = sheetData.color || config.getDefaultChartColor();
      const label = sheetData.sheetName;
      const textWidth = ctx.measureText(label).width;
      const itemWidth = 16 + textWidth + itemSpacing;

      if (
        currentX + itemWidth > maxItemWidth &&
        index < allSheetData.length - 1
      ) {
        currentX = itemStartX;
        currentY += rowHeight;
      }

      // Draw color indicator
      ctx.fillStyle = color;
      ctx.fillRect(currentX, currentY - 6, 12, 12);

      // Draw label text
      ctx.fillStyle = "#333";
      ctx.fillText(label, currentX + 16, currentY + 4);

      currentX += itemWidth;
    });
  }
}
