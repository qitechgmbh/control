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
import React, { useMemo, useState } from "react";
import { DeviceEepromDialog } from "./DeviceEepromDialog";
import { getMachineProperties } from "@/machines/properties";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import {
  EthercatDevicesEventData,
  useMainNamespace,
} from "@/client/mainNamespace";
import { useBackendConnected } from "@/client/socketioStore";
import { restartBackendIntoPreop } from "@/helpers/troubleshoot_helpers";
import { toast } from "sonner";
import { TouchButton } from "@/components/touch/TouchButton";

export function createColumns(
  isPreop: boolean,
): ColumnDef<
  NonNullable<EthercatDevicesEventData["Done"]>["devices"][number]
>[] {
  return [
    {
      accessorKey: "subdevice_index",
      header: "Index",
      cell: (row) => (
        <Value
          value={
            row.row.original.device_identification
              .device_hardware_identification.Ethercat?.subdevice_index
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
        if (!machine_identification) return "—";
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
        if (!serial) return "—";
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
        if (!machine_identification) return "—";
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
        <DeviceEepromDialog device={row.row.original} disabled={!isPreop} />
      ),
    },
  ];
}

export function EthercatPage() {
  const { ethercatDevices, ethercatState, ethercatInterfaceDiscovery } =
    useMainNamespace();
  const backendConnected = useBackendConnected();
  const [isRestartPreopLoading, setIsRestartPreopLoading] = useState(false);
  const etherCatState = ethercatState?.data?.State;
  const data = useMemo(() => {
    return ethercatDevices?.data?.Done?.devices || [];
  }, [ethercatDevices]);

  const columns = useMemo(
    () => createColumns(etherCatState === "preop"),
    [etherCatState === "preop"],
  );

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  const handleRestartBackendIntoPreop = async () => {
    setIsRestartPreopLoading(true);
    try {
      const result = await restartBackendIntoPreop();
      if (result.success) {
        toast.success("Backend restarted into Preop mode");
      } else {
        toast.error(`Failed to restart into Preop: ${result.error}`);
      }
    } catch (error) {
      toast.error(`Failed to restart into Preop: ${error}`);
    } finally {
      setIsRestartPreopLoading(false);
    }
  };

  return (
    <Page>
      <SectionTitle title="Interface" />
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

      <SectionTitle title="Prepare SubDevices">
        <div className="flex w-fit items-center gap-1.5 rounded-full bg-neutral-100 p-0.5 px-3">
          <div
            className={`h-2.5 w-2.5 rounded-full ${
              !backendConnected
                ? "bg-red-400"
                : etherCatState === "preop"
                  ? "bg-yellow-400"
                  : etherCatState === "op"
                    ? "bg-green-400"
                    : "bg-neutral-400"
            }`}
          />
          <span className="text-xs text-neutral-500">
            {!backendConnected ? "disconnected" : etherCatState}
          </span>
        </div>
      </SectionTitle>

      <p style={{ lineHeight: "1.6", margin: "1em 0" }}>
        SubDevices have to be in preop before writing to the EEPROM is allowed
      </p>
      {!backendConnected && (
        <span
          style={{
            color: "#fff",
            backgroundColor: "#dc2626",
            padding: "2px 6px",
            borderRadius: "4px",
            fontWeight: "bold",
            display: "inline-block",
            width: "max-content",
            marginLeft: "4px",
          }}
        >
          BACKEND DISCONNECTED — RECONNECTING…
        </span>
      )}
      {etherCatState === "preop" && (
        <span
          style={{
            color: "#542603",
            backgroundColor: "#dea31b",
            padding: "2px 6px",
            borderRadius: "4px",
            fontWeight: "bold",
            display: "inline-block",
            width:
              "max-content" /* Forces the width to match the text exactly */,
            marginLeft: "4px",
          }}
        >
          NOTE: YOU ARE CURRENTLY IN PREOP, MACHINES WILL NOT SHOW UP/WORK! GO
          TO TROUBLESHOOT AND PRESS RESTART BACKEND
        </span>
      )}
      <MyTable
        table={table}
        key={`${data.toString()}-${etherCatState === "preop"}`}
      />
    </Page>
  );
}
