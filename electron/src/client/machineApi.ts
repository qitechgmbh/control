import { API_BASE_URL } from "@/client/useClient";
import { z } from "zod";

const machineApiToggleRequestSchema = z.object({ enabled: z.boolean() });
const machineApiToggleResponseSchema = z.object({
  enabled: z.boolean(),
  ip_addresses: z.array(z.string()),
});

export const MachineApi = {
  async fetchApiStatus() {
    const response = await fetch(`${API_BASE_URL}/api/v1/machine/api/enabled`);
    if (!response.ok) throw new Error("Failed to fetch API status");
    const body = await response.json();
    const parsed = machineApiToggleResponseSchema.safeParse(body);
    if (!parsed.success) throw new Error("Invalid API response format");
    return parsed.data;
  },

  async setApiEnabled(enabled: boolean) {
    const parsedBody = machineApiToggleRequestSchema.safeParse({ enabled });
    if (!parsedBody.success) throw new Error("Invalid request format");
    const response = await fetch(`${API_BASE_URL}/api/v1/machine/api/enabled`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(parsedBody.data),
    });
    if (!response.headers.get("content-type")?.includes("application/json")) {
      throw new Error(await response.text());
    }
    const body = await response.json();
    if (!response.ok) throw new Error(body);
    const parsed = machineApiToggleResponseSchema.safeParse(body);
    if (!parsed.success) throw new Error("Invalid API response format");
    return parsed.data;
  },
};
