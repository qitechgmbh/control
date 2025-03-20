import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { EthercatVendorId, Hex, Value } from "@/components/Value";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useMemo } from "react";
import { DeviceEepromDialog } from "./DeviceEepromDialog";
import { getMachinePreset } from "@/machines/types";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import { EthercatSetupEventData, useMainRoom } from "@/client/mainRoom";

export const columns: ColumnDef<EthercatSetupEventData["devices"][number]>[] = [
  {
    accessorKey: "subdevice_index",
    header: "Index",
    cell: (row) => <Value value={row.row.original.subdevice_index} />,
  },
  {
    accessorKey: "configured_address",
    header: "Adress",
    cell: (row) => <Hex value={row.row.original.configured_address} />,
  },
  {
    accessorKey: "name",
    header: "Device Name",
    cell: (row) => <div>{row.row.original.name}</div>,
  },
  {
    accessorKey: "vendor_id",
    header: "Vendor",
    cell: (row) => <EthercatVendorId value={row.row.original.vendor_id} />,
  },
  {
    accessorKey: "product_id",
    header: "Product ID",
    cell: (row) => <Hex value={row.row.original.product_id} />,
  },
  {
    accessorKey: "revision",
    header: "Revision",
    cell: (row) => <Hex value={row.row.original.revision} />,
  },
  {
    accessorKey: "qitech_machine",
    header: "Assigned Machine",
    cell: (row) => {
      const machine_identification_unique =
        row.row.original.machine_device_identification
          ?.machine_identification_unique;
      if (!machine_identification_unique) {
        return "—";
      }
      const machinePreset = getMachinePreset(machine_identification_unique);
      return machinePreset?.name + " " + machinePreset?.version;
    },
  },
  {
    accessorKey: "qitech_serial",
    header: "Assigned Serial",
    cell: (row) => {
      const serial =
        row.row.original.machine_device_identification
          ?.machine_identification_unique.serial;
      if (!serial) {
        return "—";
      }
      return <Value value={serial} />;
    },
  },
  {
    accessorKey: "qitech_role",
    header: "Assigned Device Role",
    cell: (row) => {
      const role = row.row.original.machine_device_identification?.role;
      const machine_identification_unique =
        row.row.original.machine_device_identification
          ?.machine_identification_unique;
      if (!machine_identification_unique) {
        return "—";
      }
      const machinePreset = getMachinePreset(machine_identification_unique);
      const deviceRole = machinePreset?.device_roles.find(
        (device_role) => device_role.role === role,
      );
      if (!deviceRole) {
        return "UNKNOWN " + role;
      }

      return <DeviceRoleComponent device_role={deviceRole} />;
    },
  },
  {
    accessorKey: "eeprom",
    header: "Edit Assignment",
    cell: (row) => (
      <>
        <DeviceEepromDialog device={row.row.original} />
      </>
    ),
  },
];

export function EthercatPage() {
  const {
    state: { ethercatSetup },
  } = useMainRoom();

  const data = useMemo(() => {
    return ethercatSetup?.content.Data?.devices || [];
  }, [ethercatSetup]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <SectionTitle title="SubDevices">
        <RefreshIndicator event={ethercatSetup} />
      </SectionTitle>
      <p>
        Machine, Machine Serial Number, Role are QiTech specific values that are
        written to the EEPROM to identify machines as a unit.
      </p>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
