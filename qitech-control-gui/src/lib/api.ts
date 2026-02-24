/**
 * REST API client
 *
 * Thin wrapper — no react-query or custom hooks needed for simple mutations.
 * SolidJS's createResource() handles data fetching with built-in loading/error states.
 */

export const BASE_URL = "http://localhost:3001";

export type MachineIdentificationUnique = {
  machine_identification: { vendor: number; machine: number };
  serial: number;
};

export async function mutateMachine(
  machine_identification_unique: MachineIdentificationUnique,
  data: unknown,
): Promise<void> {
  const res = await fetch(`${BASE_URL}/api/v1/machine/mutate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ machine_identification_unique, data }),
  });
  if (!res.ok) {
    throw new Error(`Mutation failed: ${res.status} ${res.statusText}`);
  }
}
