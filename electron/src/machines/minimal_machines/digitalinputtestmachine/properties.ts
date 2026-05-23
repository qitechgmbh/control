import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const digitalInputTestMachine: MachineProperties = {
  name: "Digital Input Machine",
  version: "V1",
  slug: "digitalInputTestMachine",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0040,
  },
  device_roles: [],
};
