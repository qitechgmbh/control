"use client";

import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Bool, EthercatVendorId, Hex, Unit } from "@/components/Value";
import {
  EthercatDevicesEvent,
  useSocketioEthercatDevicesEvent,
} from "@/hooks/useSocketio";

import {
  ColumnDef,
  flexRender,
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
      <RefreshIndicator messageResponse={deviceMessage} />
      <Table className="w-full">
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead key={header.id}>
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </Page>
  );
}
