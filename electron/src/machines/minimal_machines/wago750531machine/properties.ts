import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const wago750_531Machine: MachineProperties = {
  name: "Wago 750-531 4CH DO",
  version: "V1",
  slug: "wago750531machine",
  icon: "lu:ToggleRight",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0047,
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
