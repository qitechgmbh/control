import { VENDOR_QITECH } from "../../properties";
import { MachineProperties } from "../../types";

export const properties: MachineProperties = {
  name: "FF01",
  version: "V1",
  slug: "ff01",
  icon: "lu:Scale",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0100,
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
      role_label: "Signal Light",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x7d43052,
          revision: 0x120000,
        },
      ],
    },
  ],
};