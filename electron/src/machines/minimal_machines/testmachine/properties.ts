import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const testmachine: MachineProperties = {
  name: "TestMachine",
  version: "V1",
  slug: "testmachine",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0033,
  },
  device_roles: [
    {
      role: 0,
      role_label: "Bus Coupler",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x44c2c52,
          revision: 0x120000,
        },
      ],
    },
    {
      role: 1,
      role_label: "EL2004",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x7d43052,
          revision: 0x120000,
        },
      ],
    },
  ],
};
