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
import { getVendorPreset, getMachinePreset } from "@/machines/types";
import { IconText } from "@/components/IconText";
import {
  EthercatSetupEventData,
  useMainNamespace,
} from "@/client/mainNamespace";

export const columns: ColumnDef<
  NonNullable<EthercatSetupEventData["Done"]>["machines"][number]
>[] = [
  {
    accessorKey: "qitech_machine",
    header: "Machine",
    cell: (row) => {
      const machine_identification_unique =
        row.row.original?.machine_identification_unique;
      if (!machine_identification_unique) {
        return "—";
      }
      const machinePreset = getMachinePreset(machine_identification_unique);
      return machinePreset?.name + " " + machinePreset?.version;
    },
  },
  {
    accessorKey: "qitech_vendor",
    header: "Vendor",
    cell: (row) => {
      const machine_identification_unique =
        row.row.original?.machine_identification_unique;
      if (!machine_identification_unique) {
        return "—";
      }
      return (
        getVendorPreset(machine_identification_unique.vendor)?.name ?? "UNKNOWN"
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
  const { ethercatSetup } = useMainNamespace();

  const data = useMemo(() => {
    return ethercatSetup?.data?.Done?.machines || [];
  }, [ethercatSetup]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <SectionTitle title="Machines">
        <RefreshIndicator ts={ethercatSetup?.ts} />
      </SectionTitle>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
