import { VENDOR_QITECH } from "../../properties-shared";
import type { MachineProperties } from "../../types";

export const aquapath1: MachineProperties = {
  name: "Aquapath",
  version: "V1",
  slug: "aquapath1",
  icon: "lu:Waves",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0009,
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
      role_label: "EL2008",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x7d83052,
          revision: 0x110000,
        },
        {
          vendor_id: 2,
          product_id: 0x7d83052,
          revision: 0x120000,
        },
      ],
    },
    {
      role: 2,
      role_label: "EL4002",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0xfa23052,
          revision: 0x140000,
        },
      ],
    },
    {
      role: 3,
      role_label: "EL3204",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0xc843052,
          revision: 0x160000,
        },
      ],
    },
    {
      role: 4,
      role_label: "EL5152",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x14203052,
          revision: 0x140000,
        },
      ],
    },
  ],
};
