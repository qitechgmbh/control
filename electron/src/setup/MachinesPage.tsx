import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { Value } from "@/components/Value";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import {
  getVendorProperties,
  getMachineProperties,
} from "@/machines/properties";
import { IconText } from "@/components/IconText";
import { MachinesEventData, useMainNamespace } from "@/client/mainNamespace";
import { API_BASE_URL, extractError } from "@/client/useClient";
import { toastHttpNotOk, toastZodError } from "@/components/Toast";
import { z } from "zod";
import { TouchButton } from "@/components/touch/TouchButton";

const machineApiToggleRequestSchema = z.object({ enabled: z.boolean() });
const machineApiToggleResponseSchema = z.object({
  enabled: z.boolean(),
  ip_addresses: z.array(z.string()),
});

export const columns: ColumnDef<
  NonNullable<MachinesEventData>["machines"][number]
>[] = [
  {
    accessorKey: "qitech_machine",
    header: "Machine",
    cell: (row) => {
      const machine_identification =
        row.row.original?.machine_identification_unique.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      const machinePreset = getMachineProperties(machine_identification);
      return machinePreset?.name + " " + machinePreset?.version;
    },
  },
  {
    accessorKey: "qitech_vendor",
    header: "Vendor",
    cell: (row) => {
      const machine_identification =
        row.row.original?.machine_identification_unique.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      return (
        getVendorProperties(machine_identification.vendor)?.name ?? "UNKNOWN"
      );
    },
  },
  {
    accessorKey: "qitech_serial",
    header: "Serial",
    cell: (row) => {
      const serial = row.row.original?.machine_identification_unique.serial;
      if (!serial) {
        return "—";
      }
      return <Value value={serial} />;
    },
  },
  {
    accessorKey: "error",
    header: "Error",
    cell: (row) => {
      const error = row.row.original.error;
      if (!error) {
        return <IconText icon="lu:Check" variant="success"></IconText>;
      }
      return (
        <IconText icon="lu:TriangleAlert" variant="error">
          {error}
        </IconText>
      );
    },
  },
];

export function MachinesPage() {
  const { machines } = useMainNamespace();

  const [machineApiEnabled, setMachineApiEnabled] = useState<
    boolean | undefined
  >(machines?.data?.machine_api_enabled);
  const [ipAddresses, setIpAddresses] = useState<string[]>([]);
  const [updating, setUpdating] = useState(false);

  useEffect(() => {
    if (machines?.data?.machine_api_enabled !== undefined) {
      setMachineApiEnabled(machines.data.machine_api_enabled);
    }
  }, [machines?.data?.machine_api_enabled]);

  // Fetch IP addresses on mount and when API is enabled
  useEffect(() => {
    const fetchIpAddresses = async () => {
      try {
        const response = await fetch(
          `${API_BASE_URL}/api/v1/machine/api/enabled`,
        );
        if (response.ok) {
          const body = await response.json();
          const parsedResponse = machineApiToggleResponseSchema.safeParse(body);
          if (parsedResponse.success) {
            setIpAddresses(parsedResponse.data.ip_addresses);
          }
        }
      } catch (error) {
        // Silently fail - IP addresses are not critical
        console.error("Failed to fetch IP addresses:", error);
      }
    };

    if (machineApiEnabled) {
      fetchIpAddresses();
    }
  }, [machineApiEnabled]);

  const handleToggle = useCallback(
    async (next: boolean) => {
      if (updating) {
        return;
      }

      const parsedBody = machineApiToggleRequestSchema.safeParse({
        enabled: next,
      });
      if (!parsedBody.success) {
        toastZodError(parsedBody.error, "API Anfrage falsch formatiert");
        return;
      }

      setUpdating(true);
      try {
        const response = await fetch(
          `${API_BASE_URL}/api/v1/machine/api/enabled`,
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(parsedBody.data),
          },
        );

        if (
          !response.headers.get("content-type")?.includes("application/json")
        ) {
          const errorText = await response.text();
          toastHttpNotOk(response.status, errorText);
          return;
        }

        const body = await response.json();
        if (!response.ok) {
          toastHttpNotOk(response.status, extractError(response.status, body));
          return;
        }

        const parsedResponse = machineApiToggleResponseSchema.safeParse(body);
        if (!parsedResponse.success) {
          toastZodError(parsedResponse.error, "API Antwort falsch formatiert");
          return;
        }

        setMachineApiEnabled(parsedResponse.data.enabled);
        setIpAddresses(parsedResponse.data.ip_addresses);
      } catch (error) {
        const message =
          error instanceof Error ? error.message : String(error ?? "");
        toastHttpNotOk(500, message);
      } finally {
        setUpdating(false);
      }
    },
    [updating],
  );

  const data = useMemo(() => {
    return machines?.data?.machines || [];
  }, [machines]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <SectionTitle title="Machines">
        <RefreshIndicator ts={machines?.ts} />
      </SectionTitle>
      <div className="bg-background mb-6 rounded-lg border border-zinc-200 p-4 shadow-sm">
        <div className="flex flex-col gap-4">
          <div className="flex flex-row items-start justify-between gap-4">
            <div className="flex-1">
              <div className="text-sm font-semibold tracking-wide text-zinc-500 uppercase">
                Host Machine API
              </div>
              <div className="text-foreground text-base font-medium">
                Expose machine read endpoints
              </div>
              <p className="text-muted-foreground text-sm">
                Allow other hosts to query state and live values via HTTP.
              </p>
            </div>

            <div className="flex flex-col gap-2">
              <div className="text-sm font-medium text-zinc-700">
                API Status
              </div>
              <div className="flex gap-2">
                <TouchButton
                  variant={machineApiEnabled ? "default" : "outline"}
                  onClick={() => handleToggle(true)}
                  disabled={
                    machineApiEnabled === undefined ||
                    updating ||
                    machineApiEnabled === true
                  }
                  isLoading={updating && machineApiEnabled !== true}
                >
                  Enabled
                </TouchButton>
                <TouchButton
                  variant={!machineApiEnabled ? "default" : "outline"}
                  onClick={() => handleToggle(false)}
                  disabled={
                    machineApiEnabled === undefined ||
                    updating ||
                    machineApiEnabled === false
                  }
                  isLoading={updating && machineApiEnabled !== false}
                >
                  Disabled
                </TouchButton>
              </div>
            </div>
          </div>

          {machineApiEnabled && ipAddresses.length > 0 && (
            <div className="border-t border-zinc-200 pt-4">
              <div className="mb-2 text-sm font-medium text-zinc-700">
                Available at:
              </div>
              <div className="flex flex-col gap-1">
                {ipAddresses.map((ip) => (
                  <div key={ip} className="font-mono text-sm text-zinc-600">
                    http://{ip}:3001/api
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
