import { toastHttpNotOk, toastZodError } from "@/components/Toast";
import { useState } from "react";
import { z } from "zod";

const mutationResponseSchema = z.discriminatedUnion("success", [
  z.object({
    success: z.literal(true),
    error: z.null(),
  }),
  z.object({
    success: z.literal(false),
    error: z.string(),
  }),
]);

const writeMachineIdentification = z.object({
  subdevice_index: z.number(),
  machine_identification: z.object({
    vendor: z.number(),
    serial: z.number(),
    machine: z.number(),
  }),
  role: z.number(),
});

export type MutationResponseSchema = z.infer<typeof mutationResponseSchema>;

export type WriteMachineIdentificationRequest = z.infer<
  typeof writeMachineIdentification
>;

type Client = {
  writeMachineDeviceIdentification: (
    req: WriteMachineIdentificationRequest,
  ) => Promise<MutationResponseSchema>;
  _request: (options: {
    path: string;
    method?: string;
    headers?: Record<string, string>;
    body?: any;
    bodySchema?: z.ZodType<any, any>;
  }) => Promise<MutationResponseSchema>;
};

const baseUrl = "http://localhost:3001";

export const getClient = () => {
  const client: Client = {
    writeMachineDeviceIdentification: async (
      req: WriteMachineIdentificationRequest,
    ) => {
      return client._request({
        path: "/api/v1/write_machine_device_identification",
        body: req,
        bodySchema: writeMachineIdentification,
      });
    },
    _request: async ({
      path,
      method = "POST",
      headers = {},
      body = {},
      bodySchema = z.object({}),
    }) => {
      const bodyParsed = bodySchema.safeParse(body);
      if (!bodyParsed.success) {
        toastZodError(bodyParsed.error, "API Anfrage falsch formatiert");
        return {
          success: false,
          error: "Invalid request body",
        };
      }
      const response = await fetch(`${baseUrl}${path}`, {
        method,
        headers: {
          "Content-Type": "application/json",
          ...headers,
        },
        body: JSON.stringify(bodyParsed.data),
      });
      if (!response.ok) {
        toastHttpNotOk(response);
        return {
          success: false,
          error: "API Fehler",
        };
      }
      const data = await response.json();
      const dataParsed = mutationResponseSchema.safeParse(data);
      if (!dataParsed.success) {
        toastZodError(dataParsed.error, "API Antwort falsch formatiert");
        return {
          success: false,
          error: dataParsed.error.message,
        };
      }
      return dataParsed.data;
    },
  };
  return client;
};

export const useClient = () => {
  const [client] = useState<Client>(getClient());
  return client;
};
