import { toastHttpNotOk, toastZodError } from "@/components/Toast";
import {
  deviceHardwareIdentificationEthercatSchema,
  deviceMachineIdentification,
  machineIdentificationUnique,
} from "@/machines/types";
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
  hardware_identification_ethercat: deviceHardwareIdentificationEthercatSchema,
  device_machine_identification: deviceMachineIdentification,
});

function machineMutateRequestSchema<T extends z.ZodTypeAny>(dataSchema: T) {
  return z.object({
    machine_identification_unique: machineIdentificationUnique,
    data: dataSchema,
  });
}

export type MachineMutateRequestSchema<T extends z.ZodTypeAny> = z.infer<
  ReturnType<typeof machineMutateRequestSchema<T>>
>;

export type MutationResponseSchema = z.infer<typeof mutationResponseSchema>;

export type WriteMachineIdentificationRequest = z.infer<
  typeof writeMachineIdentification
>;

type Client = {
  writeMachineDeviceIdentification: (
    req: WriteMachineIdentificationRequest,
  ) => Promise<MutationResponseSchema>;
  machineMutate: <T extends z.ZodTypeAny>(
    req: MachineMutateRequestSchema<T>,
    dataSchema: T,
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
    machineMutate: async <T extends z.ZodTypeAny>(
      req: MachineMutateRequestSchema<T>,
      dataSchema: T,
    ) => {
      return client._request({
        path: "/api/v1/machine/mutate",
        body: req,
        bodySchema: machineMutateRequestSchema(dataSchema),
      });
    },
    _request: async ({
      path,
      method = "POST",
      headers = {},
      body = {},
      bodySchema = z.object({}),
    }) => {
      // check request body
      const bodyParsed = bodySchema.safeParse(body);
      if (!bodyParsed.success) {
        toastZodError(bodyParsed.error, "API Anfrage falsch formatiert");
        return {
          success: false,
          error: "Invalid request body",
        };
      }

      // send request
      const response = await fetch(`${baseUrl}${path}`, {
        method,
        headers: {
          "Content-Type": "application/json",
          ...headers,
        },
        body: JSON.stringify(bodyParsed.data),
      });

      // check response content type
      if (!response.headers.get("content-type")?.includes("application/json")) {
        const error = await response.text();
        toastHttpNotOk(response.status, error);
        return {
          success: false,
          error,
        };
      }

      // check i response has error
      const data = await response.json();
      if (!response.ok) {
        const error = extractError(response.status, data);
        toastHttpNotOk(response.status, error);
        return {
          success: false,
          error: error,
        };
      }

      // check response body
      const dataParsed = mutationResponseSchema.safeParse(data);
      if (!dataParsed.success) {
        toastZodError(dataParsed.error, "API Antwort falsch formatiert");
        return {
          success: false,
          error: dataParsed.error.message,
        };
      }

      // yay, success
      return dataParsed.data;
    },
  };
  return client;
};

export const useClient = () => {
  const [client] = useState<Client>(getClient());
  return client;
};

export const useMachineMutate = <T extends z.ZodTypeAny>(
  dataSchema: T,
): {
  request: (
    req: MachineMutateRequestSchema<T>,
  ) => Promise<MutationResponseSchema>;
  isLoading: boolean;
  response: MutationResponseSchema | undefined;
} => {
  const client = useClient();
  const [res, setRes] = useState<MutationResponseSchema | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  return {
    request: async (req: MachineMutateRequestSchema<T>) => {
      setLoading(true);
      const res = await client.machineMutate(req, dataSchema);
      setRes(res);
      setLoading(false);
      return res;
    },
    isLoading: loading,
    response: res,
  };
};

export function extractError(status: number, body: any): string {
  // if body is an object,
  if (typeof body === "object") {
    if (body.error) {
      return body.error;
    }
    return JSON.stringify(body);
  }
  // if body is a string
  if (typeof body === "string") {
    return body;
  }

  return JSON.stringify(body);
}
