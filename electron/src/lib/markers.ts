import { MachineIdentificationUnique } from "@/machines/types";

const MARKERS_URL = "http://localhost:3001/api/v1/machine/markers";

/** A graph marker stored in the database, shared by all graphs of a machine. */
export type Marker = {
  id: string;
  /** Unix ms. */
  timestamp: number;
  name: string;
  color: string;
};

export type NewMarker = {
  timestamp: number;
  name: string;
  color?: string;
};

async function post<T>(path: string, body: unknown): Promise<T> {
  const response = await fetch(`${MARKERS_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    let message = `HTTP ${response.status}`;
    try {
      const json = await response.json();
      if (json?.error) message = json.error;
    } catch {
      // keep the status message
    }
    throw new Error(message);
  }
  return response.json();
}

export function fetchMarkers(
  machine: MachineIdentificationUnique,
): Promise<Marker[]> {
  return post("", { machine_identification_unique: machine });
}

export function createMarker(
  machine: MachineIdentificationUnique,
  marker: NewMarker,
): Promise<Marker> {
  return post("/add", {
    machine_identification_unique: machine,
    timestamp: Math.round(marker.timestamp),
    name: marker.name,
    color: marker.color,
  });
}

export async function deleteMarker(
  machine: MachineIdentificationUnique,
  id: string,
): Promise<void> {
  await post("/remove", { machine_identification_unique: machine, id });
}

export async function deleteAllMarkers(
  machine: MachineIdentificationUnique,
): Promise<void> {
  await post("/clear", { machine_identification_unique: machine });
}
