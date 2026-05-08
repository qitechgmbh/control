import { VENDOR_QITECH } from "../../properties-shared";
import type { MachineProperties } from "../../types";

export const wagoPower1: MachineProperties = {
  name: "WAGO Power",
  version: "V1",
  slug: "wago_power1",
  icon: "lu:PlugZap",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x000a,
  },
  device_roles: [],
};
