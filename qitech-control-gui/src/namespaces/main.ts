/**
 * Main namespace — machine discovery & EtherCAT status
 *
 * Compare to electron/src/client/mainNamespace.ts (~170 lines + socketioStore boilerplate).
 * This entire file is ~60 lines and contains the same information.
 */

import { createNamespace } from "../lib/socketio";

// ---------------------------------------------------------------------------
// Types (mirrors the Rust structs)
// ---------------------------------------------------------------------------

export type MachineObj = {
  machine_identification_unique: {
    machine_identification: { vendor: number; machine: number };
    serial: number;
  };
  error: string | null;
};

export type EthercatDevicesEventData =
  | { Initializing: true }
  | { Done: { devices: EthercatDevice[] } }
  | { Error: string };

export type EthercatDevice = {
  configured_address: number;
  name: string;
  vendor_id: number;
  product_id: number;
  revision: number;
};

export type MainState = {
  machines: MachineObj[] | null;
  ethercatDevices: EthercatDevicesEventData | null;
  ethercatInterface: string | null;
};

// ---------------------------------------------------------------------------
// Hook: useMainNamespace
//
// In React this required:
//   mainNamespace.ts   (~170 lines, Zod schemas, Zustand store, handler, hook)
//   socketioStore.ts   (createNamespaceHookImplementation, ThrottledStoreUpdater, etc.)
//
// In SolidJS: one createNamespace() call does it all.
// ---------------------------------------------------------------------------

export function createMainNamespace() {
  return createNamespace<MainState>(
    "/main",
    (event, set) => {
      if (event.name === "MachinesEvent") {
        set("machines", event.data.machines);
      } else if (event.name === "EthercatDevicesEvent") {
        set("ethercatDevices", event.data);
      } else if (event.name === "EthercatInterfaceDiscoveryEvent") {
        if ("Done" in event.data) set("ethercatInterface", event.data.Done);
      }
    },
    { machines: null, ethercatDevices: null, ethercatInterface: null },
  );
}
