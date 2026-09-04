import { Page } from "@/components/Page";
import { RefreshIndicator } from "@/components/RefreshIndicator";
import { SectionTitle } from "@/components/SectionTitle";
import { MyTable } from "@/components/Table";
import { Hex, Value } from "@/components/Value";
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import React, { useMemo, useState } from "react";
import {
  isModbusDeviceAssigned,
  ModbusAssignDialog,
} from "./ModbusAssignDialog";
import { getMachineProperties } from "@/machines/properties";
import { ModbusDevice, useMainNamespace } from "@/client/mainNamespace";
import { useClient } from "@/client/useClient";
import { Button } from "@/components/ui/button";
import { Icon } from "@/components/Icon";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function createColumns(): ColumnDef<ModbusDevice>[] {
  return [
    {
      accessorKey: "port",
      header: "Port",
      cell: (row) => <div className="font-mono text-xs">{row.row.original.port}</div>,
    },
    {
      accessorKey: "description",
      header: "Device",
      cell: (row) => {
        const d = row.row.original;
        const parts = [
          d.description,
          d.usb_vid != null && d.usb_pid != null
            ? `${d.usb_vid.toString(16).padStart(4, "0")}:${d.usb_pid.toString(16).padStart(4, "0")}`
            : null,
          d.usb_serial,
        ].filter(Boolean);
        return <div>{parts.length > 0 ? parts.join(" · ") : "—"}</div>;
      },
    },
    {
      accessorKey: "device_node",
      header: "Node",
      cell: (row) => <div>{row.row.original.device_node ?? "—"}</div>,
    },
    {
      accessorKey: "present",
      header: "Present",
      cell: (row) => (
        <div
          className={
            row.row.original.present ? "text-green-600" : "text-neutral-400"
          }
        >
          {row.row.original.present ? "Yes" : "No"}
        </div>
      ),
    },
    {
      accessorKey: "assigned_machine",
      header: "Assigned Machine",
      cell: (row) => {
        const device = row.row.original;
        if (!isModbusDeviceAssigned(device)) return "—";
        const machine_identification =
          device.assignment!.machine_identification_unique.machine_identification;
        const machinePreset = getMachineProperties(machine_identification);
        if (!machinePreset) return "UNKNOWN " + machine_identification.machine;
        return machinePreset.name + " " + machinePreset.version;
      },
    },
    {
      accessorKey: "assigned_serial",
      header: "Assigned Serial",
      cell: (row) => {
        const device = row.row.original;
        if (!isModbusDeviceAssigned(device)) return "—";
        return (
          <Value value={device.assignment!.machine_identification_unique.serial} />
        );
      },
    },
    {
      accessorKey: "slave_id",
      header: "Slave ID",
      cell: (row) => {
        const device = row.row.original;
        if (!isModbusDeviceAssigned(device)) return "—";
        return <Hex value={device.assignment!.slave_id} />;
      },
    },
    {
      accessorKey: "assign",
      header: "Edit Assignment",
      cell: (row) => <ModbusAssignDialog device={row.row.original} />,
    },
  ];
}

export function ModbusPage() {
  const { modbusDevices } = useMainNamespace();
  const client = useClient();
  const [isScanning, setIsScanning] = useState(false);

  const data = useMemo(() => {
    return modbusDevices?.data?.devices || [];
  }, [modbusDevices]);

  const columns = useMemo(() => createColumns(), []);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  const handleRescan = async () => {
    setIsScanning(true);
    try {
      await client.scanModbusDevices();
    } finally {
      setIsScanning(false);
    }
  };

  return (
    <Page>
      <SectionTitle
        title="USB Serial Ports"
        right={
          <Button
            variant="link"
            className="h-auto gap-1.5 p-0 text-base"
            disabled={isScanning}
            onClick={handleRescan}
          >
            {isScanning ? (
              <LoadingSpinner />
            ) : (
              <Icon name="lu:RefreshCw" className="size-4!" />
            )}
            Rescan ports
          </Button>
        }
      >
        <RefreshIndicator ts={modbusDevices?.ts} />
      </SectionTitle>
      <p style={{ lineHeight: "1.6", margin: "1em 0" }}>
        Machine and Serial Number are QiTech specific values that identify
        which machine a Modbus RTU serial port belongs to. Assignments are
        saved to disk and take effect after a backend restart.
      </p>
      <MyTable table={table} key={data.toString()} />
    </Page>
  );
}
