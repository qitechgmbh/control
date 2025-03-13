import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { EthercatVendorId, Hex, Value } from "@/components/Value";
import {
  EthercatSetupEvent,
  useSocketioEthercatSetupEvent,
} from "@/hooks/useSocketio";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useMemo } from "react";
import { DeviceEepromDialog } from "./DeviceEepromDialog";
import { getMachinePreset } from "@/machines/types";
import { DeviceRoleComponent } from "@/components/DeviceRole";

export const columns: ColumnDef<EthercatSetupEvent["devices"][0]>[] = [
  {
    accessorKey: "subdevice_index",
    header: "Index",
    cell: (row) => <Value value={row.row.original.subdevice_index} />,
  },
  {
    accessorKey: "configured_address",
    header: "Adresse",
    cell: (row) => <Hex value={row.row.original.configured_address} />,
  },
  {
    accessorKey: "name",
    header: "Gerät",
    cell: (row) => <div>{row.row.original.name}</div>,
  },
  {
    accessorKey: "vendor_id",
    header: "Hersteller",
    cell: (row) => <EthercatVendorId value={row.row.original.vendor_id} />,
  },
  {
    accessorKey: "product_id",
    header: "Produkt ID",
    cell: (row) => <Hex value={row.row.original.product_id} />,
  },
  {
    accessorKey: "revision",
    header: "Revision",
    cell: (row) => <Hex value={row.row.original.revision} />,
  },
  {
    accessorKey: "qitech_machine",
    header: "Maschine",
    cell: (row) => {
      const machine_identification =
        row.row.original.machine_device_identification?.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      const machinePreset = getMachinePreset(machine_identification);
      return machinePreset?.name + " " + machinePreset?.version;
    },
  },
  {
    accessorKey: "qitech_serial",
    header: "Seriennummer",
    cell: (row) => {
      const serial =
        row.row.original.machine_device_identification?.machine_identification
          .serial;
      if (!serial) {
        return "—";
      }
      return <Value value={serial} />;
    },
  },
  {
    accessorKey: "qitech_role",
    header: "Rolle",
    cell: (row) => {
      const role = row.row.original.machine_device_identification?.role;
      const machine_identification =
        row.row.original.machine_device_identification?.machine_identification;
      if (!machine_identification) {
        return "—";
      }
      const machinePreset = getMachinePreset(machine_identification);
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
    header: "Maschinenzuweisung",
    cell: (row) => (
      <>
        <DeviceEepromDialog device={row.row.original} />
      </>
    ),
  },
];

export function EthercatPage() {
  const deviceMessage = useSocketioEthercatSetupEvent();

  const data = useMemo(() => {
    return deviceMessage.data?.devices || [];
  }, [deviceMessage]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <p>
        Maschine, Maschinen Seriennummer, Rolle sind QiTech spezifische Werte
        die in den EEPROM geschrieben werden um Maschinen als Einheit zu
        identifizieren.
      </p>
      <SectionTitle title="Geräte">
        <RefreshIndicator messageResponse={deviceMessage} />
      </SectionTitle>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
