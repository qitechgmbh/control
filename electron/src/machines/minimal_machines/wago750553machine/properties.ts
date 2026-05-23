import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const wago750_553Machine: MachineProperties = {
  name: "Wago 750-553 AO",
  version: "V1",
  slug: "wago750553machine",
  icon: "lu:SlidersHorizontal",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0046,
  },
  device_roles: [
    {
      role: 0,
      role_label: "Wago 750-354 Bus Coupler",
      allowed_devices: [
        {
          vendor_id: 0x21,
          product_id: 0x07500354,
          revision: 0x2,
        },
      ],
    },
  ],
};
