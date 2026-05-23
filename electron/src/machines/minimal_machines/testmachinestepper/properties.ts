import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const testmachinestepper: MachineProperties = {
  name: "TestMachineStepper",
  version: "V1",
  slug: "testmachinestepper",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0037,
  },
  device_roles: [
    {
      role: 0,
      role_label: "Bus Coupler",
      allowed_devices: [
        {
          vendor_id: 0x00000021,
          product_id: 0x07500354,
          revision: 0x2,
        },
      ],
    },
  ],
};
