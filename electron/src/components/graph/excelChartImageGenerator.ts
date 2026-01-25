import uPlot from "uplot";
import { ExportConfig, IExportConfig } from "./excelExportConfig";
import { TimestampConverter } from "./excelFormatters";
import { CanvasUtils } from "./excelUtils";
import { CombinedSheetData } from "./excelExportTypes";
import { VersionInfoRenderer } from "./excelVersionInfoRenderer";

/**
 * Generates chart images using uPlot
 */
export class ChartImageGenerator {
  static async generate(
    allSheetData: CombinedSheetData[],
    groupId: string,
    timeRangeTitle: string,
    sortedTimestamps: number[],
    startTime: number,
    versionRenderer?: VersionInfoRenderer,
    config?: IExportConfig,
  ): Promise<string | null> {
    const exportConfig = config || new ExportConfig();
    let container: HTMLDivElement | null = null;
    let plot: uPlot | null = null;

    try {
      const dimensions = exportConfig.getChartDimensions();
      container = CanvasUtils.createOffscreenContainer(
        dimensions.width,
        dimensions.height,
      );
      document.body.appendChild(container);

      const chartData = this.prepareChartData(
        allSheetData,
        sortedTimestamps,
        startTime,
      );
      const series = this.buildSeriesConfig(allSheetData, exportConfig);
      const opts = this.buildPlotOptions(
        groupId,
        timeRangeTitle,
        series,
        exportConfig,
      );

      plot = new uPlot(opts, chartData as uPlot.AlignedData, container);

      await this.waitForRender(container);

      const canvas = container.querySelector("canvas");
      if (!canvas) return null;

      // Render version info centered at top of chart
      if (versionRenderer) {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          versionRenderer.renderOnCanvas(ctx, canvas.width);
        }
      }

      const imageData = canvas.toDataURL("image/png");
      return imageData.split(",")[1];
    } catch (error) {
      console.error("Error generating chart image:", error);
      return null;
    } finally {
      if (plot) plot.destroy();
      if (container && document.body.contains(container)) {
        document.body.removeChild(container);
      }
    }
  }

  private static prepareChartData(
    allSheetData: CombinedSheetData[],
    sortedTimestamps: number[],
    startTime: number,
  ): number[][] {
    const chartData: number[][] = [
      TimestampConverter.arrayToSecondsFromStart(sortedTimestamps, startTime),
    ];

    const timestampIndexMap = new Map<number, number>();
    sortedTimestamps.forEach((ts, idx) => {
      timestampIndexMap.set(ts, idx);
    });

    allSheetData.forEach((sheetData) => {
      const values = new Array(sortedTimestamps.length).fill(null);

      sheetData.timestamps.forEach((ts, idx) => {
        const timeIndex = timestampIndexMap.get(ts);
        if (timeIndex !== undefined) {
          values[timeIndex] = sheetData.values[idx];
        }
      });

      chartData.push(values);
    });

    return chartData;
  }

  private static buildSeriesConfig(
    allSheetData: CombinedSheetData[],
    config?: IExportConfig,
  ): uPlot.Series[] {
    const exportConfig = config || new ExportConfig();
    const series: uPlot.Series[] = [{ label: "Time (s)" }];

    allSheetData.forEach((sheetData) => {
      const color = sheetData.color || exportConfig.getDefaultChartColor();
      series.push({
        label: sheetData.sheetName,
        stroke: color,
        width: 2,
        points: {
          show: true,
          size: 3,
          width: 1,
        },
      });
    });

    return series;
  }

  private static buildPlotOptions(
    groupId: string,
    timeRangeTitle: string,
    series: uPlot.Series[],
    config?: IExportConfig,
  ): uPlot.Options {
    const exportConfig = config || new ExportConfig();
    const dimensions = exportConfig.getChartDimensions();

    return {
      title: `${groupId} - ${timeRangeTitle}`,
      width: dimensions.width,
      height: dimensions.height,
      series,
      scales: {
        x: { time: false },
      },
      axes: [
        {
          label: "Time (seconds)",
          stroke: "#333",
          grid: { stroke: "#e0e0e0", width: 1 },
        },
        {
          label: "Values",
          stroke: "#333",
          grid: { stroke: "#e0e0e0", width: 1 },
        },
      ],
      legend: { show: false },
      cursor: { show: false },
    };
  }

  private static async waitForRender(container: HTMLDivElement): Promise<void> {
    return new Promise<void>((resolve) => {
      const checkCanvas = () => {
        const canvas = container.querySelector("canvas");
        if (canvas && canvas.width > 0 && canvas.height > 0) {
          requestAnimationFrame(() => resolve());
        } else {
          requestAnimationFrame(checkCanvas);
        }
      };
      checkCanvas();

      // Fallback timeout
      setTimeout(() => resolve(), 500);
    });
  }
}
