import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const extruder3: MachineProperties = {
  name: "Extruder",
  version: "V3",
  slug: "extruder3",
  icon: "qi:Extruder",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0016,
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
      role_label: "Inverter Interface",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 394604626,
          revision: 1376256,
        },
        {
          vendor_id: 2,
          product_id: 394604626,
          revision: 0x140000,
        },
        {
          vendor_id: 2,
          product_id: 394604626,
          revision: 0x160000,
        },
        {
          vendor_id: 2,
          product_id: 394604626,
          revision: 0x100000,
        },
      ],
    },
    {
      role: 2,
      role_label: "Heating Elements",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 131346514,
          revision: 1179648,
        },
      ],
    },
    {
      role: 3,
      role_label: "Pressure Sensor",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 197996626,
          revision: 1310720,
        },
      ],
    },
    {
      role: 4,
      role_label: "Thermometers",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0xc843052,
          revision: 1441792,
        },
        {
          vendor_id: 2,
          product_id: 0xc843052,
          revision: 0x150000,
        },
      ],
    },
  ],
};
