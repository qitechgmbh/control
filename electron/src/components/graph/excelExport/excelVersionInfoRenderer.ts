import { IEnvironmentInfoProvider } from "./excelExportConfig";

/**
 * Handles version information retrieval and rendering
 * Follows Single Responsibility Principle - only manages version info
 * Now uses IEnvironmentInfoProvider for better testability
 */
export class VersionInfoRenderer {
  private versionInfo: string = "";
  private commitInfo: string = "";

  constructor(private envProvider: IEnvironmentInfoProvider) {}

  async fetchVersionInfo(): Promise<void> {
    try {
      const info = await this.envProvider.getVersionInfo();
      this.versionInfo = info.version ?? "";
      this.commitInfo = info.commit ?? "";
    } catch (error) {
      console.warn("Failed to fetch environment info", error);
    }
  }

  renderOnCanvas(ctx: CanvasRenderingContext2D, canvasWidth: number): void {
    if (!this.versionInfo && !this.commitInfo) return;

    const versionText = this.formatVersionText();

    ctx.save();
    ctx.font = "12px sans-serif";
    ctx.fillStyle = "#666";
    ctx.textAlign = "center";
    ctx.fillText(versionText, canvasWidth / 2, 20);
    ctx.restore();
  }

  private formatVersionText(): string {
    const parts: string[] = [];
    if (this.versionInfo) {
      parts.push(`Version: ${this.versionInfo}`);
    }
    if (this.commitInfo) {
      parts.push(`Commit: ${this.commitInfo}`);
    }
    return parts.join(" | ");
  }

  getVersionInfo(): string {
    return this.versionInfo;
  }

  getCommitInfo(): string {
    return this.commitInfo;
  }
}
