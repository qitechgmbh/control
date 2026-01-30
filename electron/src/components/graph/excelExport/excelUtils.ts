/**
 * Additional utility classes for Excel export
 * Following SOLID principles and DRY
 */

/**
 * Utility for creating offscreen canvas containers
 * Eliminates duplicate container creation code
 */
export class CanvasUtils {
  static createOffscreenContainer(
    width: number,
    height: number,
  ): HTMLDivElement {
    const container = document.createElement("div");
    container.style.width = `${width}px`;
    container.style.height = `${height}px`;
    container.style.position = "absolute";
    container.style.left = "-9999px";
    return container;
  }
}

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
 * Interface for fetching machine PID settings
 */
export interface IPidDataProvider {
  fetchPidSettings(): Promise<{
    temperature?: Record<string, { kp: number; ki: number; kd: number }>;
    pressure?: { kp: number; ki: number; kd: number };
  } | null>;
}

/**
 * Provider that fetches PID settings from machine API
 * Implements Dependency Inversion Principle
 */
export class MachinePidDataProvider implements IPidDataProvider {
  constructor(
    private baseUrl: string = "http://10.10.10.1:3001",
    private machineSlug?: string,
    private machineSerial?: number,
  ) {}

  async fetchPidSettings(): Promise<{
    temperature?: Record<string, { kp: number; ki: number; kd: number }>;
    pressure?: { kp: number; ki: number; kd: number };
  } | null> {
    try {
      // If machine slug/serial provided, fetch from specific machine
      if (this.machineSlug && this.machineSerial !== undefined) {
        const response = await fetch(
          `${this.baseUrl}/api/v2/machine/${this.machineSlug}/${this.machineSerial}`,
        );

        if (!response.ok) {
          console.warn("Failed to fetch machine data for PID settings");
          return null;
        }

        const data = await response.json();

        // Extract PID settings from machine data
        // This is a placeholder - actual implementation depends on machine API structure
        return this.extractPidFromMachineData(data);
      }

      // Otherwise, return null (PID data should be passed in)
      return null;
    } catch (error) {
      console.error("Error fetching PID settings from machine:", error);
      return null;
    }
  }

  private extractPidFromMachineData(machineData: any): {
    temperature?: Record<string, { kp: number; ki: number; kd: number }>;
    pressure?: { kp: number; ki: number; kd: number };
  } | null {
    // This is a placeholder implementation
    // Actual extraction depends on machine API structure
    // TODO: Implement based on actual machine data structure

    const pidSettings: any = {};

    // Example: Look for PID-related fields in machine data
    if (machineData.temperature_controllers) {
      pidSettings.temperature = machineData.temperature_controllers;
    }

    if (machineData.pressure_controller) {
      pidSettings.pressure = machineData.pressure_controller;
    }

    return Object.keys(pidSettings).length > 0 ? pidSettings : null;
  }
}

/**
 * Mock provider for testing
 */
export class MockPidDataProvider implements IPidDataProvider {
  constructor(
    private mockData: {
      temperature?: Record<string, { kp: number; ki: number; kd: number }>;
      pressure?: { kp: number; ki: number; kd: number };
    } | null,
  ) {}

  async fetchPidSettings(): Promise<{
    temperature?: Record<string, { kp: number; ki: number; kd: number }>;
    pressure?: { kp: number; ki: number; kd: number };
  } | null> {
    return Promise.resolve(this.mockData);
  }
}
