import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const wagoAiTestMachine: MachineProperties = {
  name: "Wago AI Test",
  version: "V1",
  slug: "wagoaitestmachine",
  icon: "lu:Activity",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0036,
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
