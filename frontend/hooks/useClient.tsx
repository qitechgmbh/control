import { useState } from "react";
import { z } from "zod";

const xRequestSchema = z.object({
  address: z.number(),
});

const xResponseSchema = z.object({
  x2000: z.number(),
});

export type XResponse = z.infer<typeof xResponseSchema>;

export type XRequest = z.infer<typeof xRequestSchema>;

type Client = {
  x: (req: XRequest) => Promise<XResponse>;
};

const baseUrl = "http://localhost:3001";

export const getClient = () => {
  const client: Client = {
    x: async (req: XRequest) => {
      const response = await fetch(`${baseUrl}/api/v1/x`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(xRequestSchema.parse(req)),
      });
      const data = await response.json();
      return xResponseSchema.parse(data);
    },
  };
  return client;
};

export const useClient = () => {
  const [client] = useState<Client>(getClient());
  return client;
};
