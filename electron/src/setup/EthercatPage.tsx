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
import { getMachineProperties } from "@/machines/properties";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import {
  EthercatDevicesEventData,
  useMainNamespace,
} from "@/client/mainNamespace";

export const columns: ColumnDef<
  NonNullable<EthercatDevicesEventData["Done"]>["devices"][number]
>[] = [
  {
    accessorKey: "subdevice_index",
    header: "Index",
    cell: (row) => (
      <Value
        value={
          row.row.original.device_identification.device_hardware_identification
            .Ethercat?.subdevice_index
        }
      />
    ),
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
      const machine_identification =
        row.row.original.device_identification.device_machine_identification
          ?.machine_identification_unique.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      const machinePreset = getMachineProperties(machine_identification);
      return machinePreset?.name + " " + machinePreset?.version;
    },
  },
  {
    accessorKey: "qitech_serial",
    header: "Assigned Serial",
    cell: (row) => {
      const serial =
        row.row.original.device_identification.device_machine_identification
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
      const device_machine_identification =
        row.row.original.device_identification.device_machine_identification;
      const machine_identification =
        device_machine_identification?.machine_identification_unique
          .machine_identification;
      if (!machine_identification) {
        return "—";
      }
      const machinePreset = getMachineProperties(machine_identification);
      const deviceRole = machinePreset?.device_roles.find(
        (device_role) =>
          device_role.role === device_machine_identification.role,
      );
      if (!deviceRole) {
        return "UNKNOWN " + device_machine_identification.role;
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
  const { ethercatDevices, ethercatInterfaceDiscovery } = useMainNamespace();

  const data = useMemo(() => {
    return ethercatDevices?.data?.Done?.devices || [];
  }, [ethercatDevices]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <SectionTitle title="Interface"></SectionTitle>
      <p>
        Ethernet Interface{" "}
        {ethercatInterfaceDiscovery?.data.Discovering ? (
          <span>Discovering...</span>
        ) : ethercatInterfaceDiscovery?.data.Done ? (
          <Value value={ethercatInterfaceDiscovery?.data.Done} />
        ) : (
          <span>Not Discovering</span>
        )}
      </p>
      <SectionTitle title="SubDevices">
        <RefreshIndicator ts={ethercatDevices?.ts} />
      </SectionTitle>
      <p>
        Machine, Machine Serial Number, Role are QiTech specific values that are
        written to the EEPROM to identify machines as a unit.
      </p>

      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
