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
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
