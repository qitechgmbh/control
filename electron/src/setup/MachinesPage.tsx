import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { Value } from "@/components/Value";
import {
  EthercatSetupEventMachineInfo,
  useSocketioEthercatSetupEvent,
} from "@/hooks/useSocketio";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useMemo } from "react";
import { get_vendor_name, getMachinePreset } from "@/machines/types";
import { IconText } from "@/components/IconText";

export const columns: ColumnDef<EthercatSetupEventMachineInfo>[] = [
  {
    accessorKey: "qitech_machine",
    header: "Maschine",
    cell: (row) => {
      const machine_identification = row.row.original?.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      return getMachinePreset(machine_identification)?.name ?? "UNKNOWN";
    },
  },
  {
    accessorKey: "qitech_vendor",
    header: "Hersteller",
    cell: (row) => {
      const machine_identification = row.row.original?.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      return get_vendor_name(machine_identification.vendor)?.name ?? "UNKNOWN";
    },
  },
  {
    accessorKey: "qitech_serial",
    header: "Seriennummer",
    cell: (row) => {
      const serial = row.row.original?.machine_identification.serial;
      if (!serial) {
        return "—";
      }
      return <Value value={serial} />;
    },
  },
  {
    accessorKey: "error",
    header: "Fehler",
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
  const deviceMessage = useSocketioEthercatSetupEvent();

  const data = useMemo(() => {
    return deviceMessage.data?.machine_infos || [];
  }, [deviceMessage]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page title="EtherCAT">
      <SectionTitle title="Maschinen">
        <RefreshIndicator messageResponse={deviceMessage} />
      </SectionTitle>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
