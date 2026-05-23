import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const TestMotor: MachineProperties = {
  name: "TestMotor",
  version: "V1",
  slug: "testmotor",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0011,
  },
  device_roles: [
    {
      role: 0,
      role_label: "Bus Coupler",
      allowed_devices: [
        {
          vendor_id: 0x2,
          product_id: 0x44c2c52,
          revision: 0x120000,
        },
      ],
    },
    {
      role: 1,
      role_label: "Stepper Motor",
      allowed_devices: [
        {
          vendor_id: 0x2,
          product_id: 0x1b773052,
          revision: 0x10001e,
        },
      ],
    },
  ],
};
