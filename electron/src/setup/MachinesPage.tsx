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
import { Toggle } from "@/components/ui/toggle";
import { API_BASE_URL, extractError } from "@/client/useClient";
import { toastHttpNotOk, toastZodError } from "@/components/Toast";
import { z } from "zod";

const machineApiToggleRequestSchema = z.object({ enabled: z.boolean() });
const machineApiToggleResponseSchema = z.object({ enabled: z.boolean() });

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
  const [updating, setUpdating] = useState(false);

  useEffect(() => {
    if (machines?.data?.machine_api_enabled !== undefined) {
      setMachineApiEnabled(machines.data.machine_api_enabled);
    }
  }, [machines?.data?.machine_api_enabled]);

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
        const response = await fetch(`${API_BASE_URL}/api/v1/machine/api/enabled`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(parsedBody.data),
        });

        if (!response.headers.get("content-type")?.includes("application/json")) {
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
      <div className="mb-6 rounded-lg border border-zinc-200 bg-background p-4 shadow-sm">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <div className="text-sm font-semibold uppercase tracking-wide text-zinc-500">
              Host Machine API
            </div>
            <div className="text-base font-medium text-foreground">
              Expose machine read endpoints
            </div>
            <p className="text-sm text-muted-foreground">
              Allow other hosts to query state and live values via HTTP.
            </p>
          </div>
          <Toggle
            pressed={machineApiEnabled ?? false}
            disabled={machineApiEnabled === undefined || updating}
            onPressedChange={handleToggle}
            aria-label="Expose machine read API"
            className="h-10 min-w-[6.5rem] px-4"
          >
            {updating
              ? "Saving..."
              : machineApiEnabled
                ? "Enabled"
                : "Disabled"}
          </Toggle>
        </div>
      </div>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
