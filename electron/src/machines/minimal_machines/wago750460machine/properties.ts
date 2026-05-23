import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const wago750460Machine: MachineProperties = {
  name: "WAGO 750-460 4CH RTD",
  version: "V1",
  slug: "wago750460machine",
  icon: "lu:Thermometer",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0044,
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
