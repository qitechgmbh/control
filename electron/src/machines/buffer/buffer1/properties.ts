import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const buffer1: MachineProperties = {
  name: "Buffer",
  version: "V1",
  slug: "buffer1",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0008,
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
      role_label: "1x Stepper Spool",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b813052,
          revision: 0x100034,
        },
      ],
    },
    {
      role: 2,
      role_label: "1x Stepper Puller",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x10001e,
        },
      ],
    },
  ],
};
