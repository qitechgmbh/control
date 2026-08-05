import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const bottlecapsTestMachine: MachineProperties = {
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0039, // must match the Rust constant
  },
  name: "Bottlecaps Example",
  version: "V1",
  slug: "bottlecapstest",
  icon: "lu:ToggleLeft",
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
    {
      role: 1,
      role_label: "IP20-EC-DI8-DO8",
      allowed_devices: [
        {
          vendor_id: 0x741,
          product_id: 0x117b6722,
          revision: 0x1,
        },
      ],
    },
  ],
};
