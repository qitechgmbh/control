"use client";

import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { Button } from "@/components/ui/button";

import { Bool, EthercatVendorId, Hex, Unit } from "@/components/Value";
import {
  EthercatDevicesEvent,
  useSocketioEthercatDevicesEvent,
} from "@/hooks/useSocketio";

import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";

export const columns: ColumnDef<EthercatDevicesEvent["devices"][0]>[] = [
  {
    accessorKey: "address",
    header: "Adresse",
    cell: (row) => <Hex value={row.row.original.address} />,
  },
  {
    accessorKey: "alias_address",
    header: "Alias Adresse",
    cell: (row) => <Hex value={row.row.original.alias_address} />,
  },
  {
    accessorKey: "name",
    header: "Name",
    cell: (row) => <div>{row.row.original.name}</div>,
  },

  {
    accessorKey: "vendor_revision",
    header: "Vendor",
    cell: (row) => <EthercatVendorId value={row.row.original.vendor_id} />,
  },
  {
    accessorKey: "vendor_product_id",
    header: "Product ID",
    cell: (row) => <Hex value={row.row.original.product_id} />,
  },
  {
    accessorKey: "vendor_revision",
    header: "product Revision",
    cell: (row) => <Hex value={row.row.original.revision} />,
  },
  {
    accessorKey: "vendor_serial",
    header: "Serial",
    cell: (row) => <Hex value={row.row.original.serial} />,
  },
  {
    accessorKey: "dc_support",
    header: "Distributed Clocks",
    cell: (row) => <Bool value={row.row.original.dc_support} />,
  },
  {
    accessorKey: "propagation_delay",
    header: "Delay",
    cell: (row) => (
      <Unit value={row.row.original.propagation_delay} unit="ns" />
    ),
  },
  {
    accessorKey: "details",
    header: "Details",
    cell: (row) => (
      <Button>
        {/* <Link href={`ethercat/${row.row.original.address.toString(16)}`}>
          Details
        </Link> */}
      </Button>
    ),
  },
];

export default function EthercatPage() {
  const deviceMessage = useSocketioEthercatDevicesEvent();

  const data = deviceMessage.data?.devices || [];

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page title="EtherCAT">
      <SectionTitle title="Geräte">
        <RefreshIndicator messageResponse={deviceMessage} />
      </SectionTitle>
      <MyTable table={table} />
    </Page>
  );
}
