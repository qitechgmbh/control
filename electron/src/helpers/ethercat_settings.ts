import { baseUrl } from "@/client/useClient";
import { z } from "zod";

export const ethercatSettingSchema = z.object({
  enabled: z.boolean(),
});

export type EthercatSetting = z.infer<typeof ethercatSettingSchema>;

const path = `${baseUrl}/api/v2/settings/ethercat`;

export async function getEthercatSetting(): Promise<EthercatSetting> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to read EtherCAT setting (${response.status})`);
  }
  return ethercatSettingSchema.parse(await response.json());
}

export async function setEthercatSetting(
  enabled: boolean,
): Promise<EthercatSetting> {
  const response = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ enabled }),
  });
  if (!response.ok) {
    throw new Error(`Failed to save EtherCAT setting (${response.status})`);
  }
  return ethercatSettingSchema.parse(await response.json());
}
