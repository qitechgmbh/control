import { z } from "zod";

// Schema for read-only API status event
export const readOnlyApiStatusSchema = z.object({
  enabled: z.boolean(),
});

export type ReadOnlyApiStatus = z.infer<typeof readOnlyApiStatusSchema>;

// API request/response types
export interface MachineQueryRequest {
  machine_identification_unique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  };
  /**
   * Fields to query from the machine. Use dot notation.
   * Examples:
   * - "live_values.temperature" - get only temperature
   * - "live_values.spool_rpm" - get only spool RPM
   * - "state.mode_state" - get only mode state
   * - "live_values" - get all live values
   * - "state" - get all state
   * - "*" - get everything
   */
  fields: string[];
}

export interface MachineQueryResponse {
  success: boolean;
  error?: string;
  data?: any; // The filtered data based on requested fields
}

export interface ReadOnlyApiConfigRequest {
  enabled: boolean;
}

export interface ReadOnlyApiStatusResponse {
  enabled: boolean;
}

export interface MutationResponse {
  success: boolean;
  error?: string;
}

// API functions
export async function setReadOnlyApiEnabled(enabled: boolean): Promise<void> {
  const response = await fetch("/api/v1/read_only_api/config", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ enabled } as ReadOnlyApiConfigRequest),
  });

  if (!response.ok) {
    throw new Error(`Failed to set read-only API: ${response.statusText}`);
  }

  const data: MutationResponse = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Failed to set read-only API");
  }
}

export async function getReadOnlyApiStatus(): Promise<boolean> {
  const response = await fetch("/api/v1/read_only_api/status");

  if (!response.ok) {
    throw new Error(
      `Failed to get read-only API status: ${response.statusText}`,
    );
  }

  const data: ReadOnlyApiStatusResponse = await response.json();
  return data.enabled;
}

/**
 * Query machine data through the read-only API
 * @param machineIdentification - The machine to query
 * @param fields - Array of field paths to retrieve (e.g., ["live_values.temperature", "state.mode_state"])
 * @returns The filtered machine data
 */
export async function queryMachineData(
  machineIdentification: MachineQueryRequest["machine_identification_unique"],
  fields: string[],
): Promise<any> {
  const response = await fetch("/api/v1/machine/query", {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      machine_identification_unique: machineIdentification,
      fields,
    } as MachineQueryRequest),
  });

  if (!response.ok) {
    throw new Error(`Failed to query machine data: ${response.statusText}`);
  }

  const data: MachineQueryResponse = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Failed to query machine data");
  }

  return data.data;
}
