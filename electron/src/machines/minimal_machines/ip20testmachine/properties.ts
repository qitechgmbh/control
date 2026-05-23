import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const ip20TestMachine: MachineProperties = {
  name: "IP20 Test",
  version: "V1",
  slug: "ip20testmachine",
  icon: "lu:ToggleLeft",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0034,
  },
  device_roles: [
    {
      role: 0,
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
