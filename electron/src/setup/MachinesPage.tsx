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
import React, { useMemo } from "react";
import {
  getVendorProperties,
  getMachineProperties,
} from "@/machines/properties";
import { IconText } from "@/components/IconText";
import { MachinesEventData, useMainNamespace } from "@/client/mainNamespace";
import { setReadOnlyApiEnabled } from "@/client/readOnlyApi";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { useState } from "react";
import { toast } from "sonner";

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
  const { machines, readOnlyApiStatus } = useMainNamespace();
  const [isLoading, setIsLoading] = useState(false);

  const data = useMemo(() => {
    return machines?.data?.machines || [];
  }, [machines]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  const handleReadOnlyApiToggle = async (enabled: boolean) => {
    setIsLoading(true);
    try {
      await setReadOnlyApiEnabled(enabled);
      toast.success(
        enabled
          ? "Read-only API enabled successfully"
          : "Read-only API disabled successfully",
      );
    } catch (error) {
      console.error("Failed to set read-only API:", error);
      toast.error("Failed to update read-only API setting");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Page>
      <SectionTitle title="Machines">
        <RefreshIndicator ts={machines?.ts} />
      </SectionTitle>
      <MyTable table={table} key={data.toString()} />

      <SectionTitle title="API Configuration" />
      <ControlCard title="Read-Only API">
        <Label label="Enable Remote Monitoring">
          <SelectionGroupBoolean
            value={readOnlyApiStatus?.data?.enabled ?? false}
            disabled={isLoading}
            loading={isLoading}
            optionFalse={{
              children: "Disabled",
              icon: "lu:Lock",
            }}
            optionTrue={{
              children: "Enabled",
              icon: "lu:LockOpen",
            }}
            onChange={handleReadOnlyApiToggle}
          />
        </Label>
        <p className="text-sm text-gray-600">
          When enabled, external applications can query machine state and live
          data through the read-only API endpoint (/api/v1/machine/query). You
          must specify which fields to retrieve (e.g.,
          "live_values.temperature", "state.mode_state"). Mutations are not
          allowed through this endpoint.
        </p>
        <div className="mt-2 rounded bg-gray-50 p-3">
          <p className="text-xs font-semibold text-gray-700">Example Usage:</p>
          <code className="mt-1 block text-xs text-gray-600">
            POST /api/v1/machine/query
            <br />
            {`{ "machine_identification_unique": {...}, "fields": ["live_values.spool_rpm", "state.mode_state"] }`}
          </code>
        </div>
      </ControlCard>
    </Page>
  );
}
