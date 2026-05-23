import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const winder2_spool_7031: MachineProperties = {
  name: "Winder",
  version: "V2_SPOOL_7031",
  slug: "winder2_7031",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0062,
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
      role_label: "2x Digital Output",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x7d23052,
          revision: 0x110000,
        },
        {
          vendor_id: 2,
          product_id: 0x7d23052,
          revision: 0x120000,
        },
      ],
    },
    {
      role: 2,
      role_label: "1x Stepper Spool",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x10001e,
        },
      ],
    },
    {
      role: 3,
      role_label: "1x Stepper Traverse",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x1a0000,
        },
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x190000,
        },
      ],
    },
    {
      role: 4,
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
