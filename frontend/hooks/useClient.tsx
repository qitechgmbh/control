import { useState } from "react";
import { z } from "zod";

const GetEthercat = z.object({
  devices: z.array(
    z.object({
      name: z.string(),
      address: z.string(),
    })
  ),
});

export type GetEthercat = z.infer<typeof GetEthercat>;

type Client = {
  getEthercat: () => Promise<GetEthercat>;
};

const baseUrl = "http://localhost:3001";

export const getClient = () => {
  const client: Client = {
    getEthercat: async () => {
      const response = await fetch(`${baseUrl}/api/v1/ethercat`);
      const data = await response.json();
      return GetEthercat.parse(data);
    },
  };
  return client;
};

export const useClient = () => {
  const [client] = useState<Client>(getClient());
  return client;
};
