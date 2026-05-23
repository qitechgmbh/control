import { MachineProperties } from "@/machines/types";
import { VENDOR_QITECH } from "@/machines/types";

export const wagoSerial: MachineProperties = {
  name: "WagoSerial",
  version: "V1",
  slug: "wago_serial",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x67,
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
    {
      role: 1,
      role_label: "Serial Interface",
      allowed_devices: [
        {
          vendor_id: 0x00000021,
          product_id: 0x6521772,
          revision: 0x2,
        },
      ],
    },
  ],
};
