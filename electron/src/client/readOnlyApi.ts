import { z } from "zod";

// Schema for read-only API status event
export const readOnlyApiStatusSchema = z.object({
  enabled: z.boolean(),
  ip_addresses: z.array(z.string()),
});

export type ReadOnlyApiStatus = z.infer<typeof readOnlyApiStatusSchema>;

// API request/response types
export interface MachineEventRequest {
  machine_identification_unique: {
    machine_identification: {
      vendor: number;
      machine: number;
    };
    serial: number;
  };
  /**
   * Optional event field specification.
   * - undefined/null: returns all available events with all fields
   * - { LiveValues: ["field1", "field2"], State: null }: returns LiveValues with specific fields and all State fields
   * - { LiveValues: null, State: ["field1"] }: returns all LiveValues fields and specific State fields
   * - { LiveValues: [], State: null }: returns no LiveValues, all State fields
   * 
   * Each event type appears at most once in the response.
   */
  events?: {
    LiveValues?: string[];
    State?: string[];
  };
}

export interface MachineEventResponse {
  success: boolean;
  error?: string;
  data?: {
    State?: any;
    LiveValues?: any;
    [key: string]: any;
  };
}export interface ReadOnlyApiConfigRequest {
  enabled: boolean;
}

export interface ReadOnlyApiStatusResponse {
  enabled: boolean;
  ip_addresses: string[];
}

export interface MutationResponse {
  success: boolean;
  error?: string;
}

// Base URL for API requests
const API_BASE_URL = "http://localhost:3001";

// API functions
export async function setReadOnlyApiEnabled(enabled: boolean): Promise<void> {
  console.log(`[ReadOnlyAPI] Setting read-only API to: ${enabled}`);

  const response = await fetch(`${API_BASE_URL}/api/v1/read_only_api/config`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ enabled } as ReadOnlyApiConfigRequest),
  });

  console.log(
    `[ReadOnlyAPI] Response status: ${response.status} ${response.statusText}`,
  );

  if (!response.ok) {
    const errorText = await response.text();
    console.error(`[ReadOnlyAPI] HTTP Error Response:`, errorText);
    throw new Error(
      `Failed to set read-only API: ${response.statusText} - ${errorText}`,
    );
  }

  const data: MutationResponse = await response.json();
  console.log(`[ReadOnlyAPI] Response data:`, data);

  if (!data.success) {
    console.error(`[ReadOnlyAPI] API returned success=false:`, data.error);
    throw new Error(data.error || "Failed to set read-only API");
  }

  console.log(`[ReadOnlyAPI] Successfully set read-only API to: ${enabled}`);
}

export async function getReadOnlyApiStatus(): Promise<boolean> {
  const response = await fetch(`${API_BASE_URL}/api/v1/read_only_api/status`);

  if (!response.ok) {
    throw new Error(
      `Failed to get read-only API status: ${response.statusText}`,
    );
  }

  const data: ReadOnlyApiStatusResponse = await response.json();
  return data.enabled;
}

/**
 * Query machine events through the read-only API
 * @param machineIdentification - The machine to query
 * @param events - Optional event field specification. If omitted, returns all events with all fields.
 *                 Example: { LiveValues: ["temperature", "pressure"], State: null } returns specific LiveValues fields and all State fields
 * @returns Object with event types as keys (e.g., { "State": {...}, "LiveValues": {...} })
 */
export async function queryMachineData(
  machineIdentification: MachineEventRequest["machine_identification_unique"],
  events?: {
    LiveValues?: string[];
    State?: string[];
  },
): Promise<any> {
  const response = await fetch(`${API_BASE_URL}/api/v1/machine/event`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      machine_identification_unique: machineIdentification,
      events,
    } as MachineEventRequest),
  });

  if (!response.ok) {
    throw new Error(`Failed to query machine data: ${response.statusText}`);
  }

  const data: MachineEventResponse = await response.json();
  if (!data.success) {
    throw new Error(data.error || "Failed to query machine data");
  }

  return data.data;
}
