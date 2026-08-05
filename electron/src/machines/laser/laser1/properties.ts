import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const laser1: MachineProperties = {
  name: "Laser",
  version: "V1",
  slug: "laser1",
  icon: "lu:Sun",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0006,
  },
  device_roles: [],
};
