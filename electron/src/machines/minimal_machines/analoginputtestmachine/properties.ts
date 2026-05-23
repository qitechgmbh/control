import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const analogInputTestMachine: MachineProperties = {
  name: "AnalogTest",
  version: "V1",
  slug: "analogInputTestMachine",
  icon: "lu:Clock",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0035,
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
      role_label: "EL3021",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0xbcd3052,
          revision: 0x140000,
        },
      ],
    },
  ],
};
