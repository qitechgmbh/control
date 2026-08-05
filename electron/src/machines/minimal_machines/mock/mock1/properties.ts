import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const mock1: MachineProperties = {
  name: "Mock",
  version: "V1",
  slug: "mock1",
  icon: "lu:FlaskConical",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0007,
  },
  device_roles: [],
};
