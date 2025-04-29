// every device has multiple roles to indentify the function of a subdevice
// each role can only be given once

import { IconName } from "@/components/Icon";
import { z } from "zod";

// EK1100 should have role 0
export type DeviceRole = {
  role: number;
  // UI purpose
  role_label: string;
  // what kind of subdevices can be assigned to this role
  allowed_devices: EthercatDevice[];
};

// data to identify subdevices
type EthercatDevice = {
  vendor_id: number;
  product_id: number;
  revision: number;
};

export const machineIdentificaiton = z.object({
  vendor: z.number(),
  machine: z.number(),
});

export type MachineIdentification = z.infer<typeof machineIdentificaiton>;

export const machineIdentificationUnique = machineIdentificaiton.extend({
  serial: z.number(),
});

export type MachineIdentificationUnique = z.infer<
  typeof machineIdentificationUnique
>;

export const machineDeviceIdentification = z.object({
  machine_identification_unique: machineIdentificationUnique,
  role: z.number(),
  subdevice_index: z.number(),
});

export type MachineDeviceIdentification = z.infer<
  typeof machineDeviceIdentification
>;

export type MachinePreset = {
  // displayable name
  name: string;
  // displayable version
  version: string;
  // path for IO routes
  slug: string;
  icon: IconName;
  // machine identification
  machine_identification: MachineIdentification;
  // roles and thair allowed devices
  device_roles: DeviceRole[];
};

export const VENDOR_QITECH = 0x0001;

export type VendorPreset = {
  id: number;
  name: string;
};

export const vendorPresets: VendorPreset[] = [
  {
    id: VENDOR_QITECH,
    name: "QiTech Industries GmbH",
  },
];

export function getVendorPreset(vendor: number): VendorPreset | undefined {
  return vendorPresets.find((v) => v.id === vendor);
}

export const winder2: MachinePreset = {
  name: "Winder",
  version: "V2",
  slug: "winder2",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0002,
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
      role_label: "2x Digital Output",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x7d23052,
          revision: 0x110000,
        },
      ],
    },
    {
      role: 2,
      role_label: "1x Analog Input",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0xbb93052,
          revision: 0x160000,
        },
      ],
    },
    {
      role: 3,
      role_label: "1x Stepper Winder",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b813052,
          revision: 0x100034,
        },
      ],
    },
    {
      role: 4,
      role_label: "1x Stepper Traverse",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x1a0000,
        },
      ],
    },
    {
      role: 5,
      role_label: "1x Stepper Puller",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0, // TODO
          revision: 0, // TODO
        },
      ],
    },
  ],
};

export const extruder2: MachinePreset = {
  name: "Extruder",
  version: "V2",
  slug: "extruder2",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0003,
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
      role_label: "1X Serial Interface For Inverter",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 394604626,
          revision: 1376256,
        },
      ],
    },
    {
      role: 2,
      role_label: "1X Analog Channel",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 197996626,
          revision: 1310720,
        },
      ],
    },
  ],
};

export const machinePresets: MachinePreset[] = [winder2, extruder2];

export const getMachinePreset = (
  machine_identification_unique: MachineIdentificationUnique,
) => {
  return machinePresets.find(
    (m) =>
      m.machine_identification.vendor ===
        machine_identification_unique.vendor &&
      m.machine_identification.machine ===
        machine_identification_unique.machine,
  );
};

export function filterAllowedDevices(
  vendor_id: number,
  product_id: number,
  revision: number,
  allowed_devices: DeviceRole[] | undefined,
): boolean[] {
  if (!allowed_devices) {
    return [];
  }
  return allowed_devices.map((role) =>
    role.allowed_devices.some(
      (device) =>
        device.product_id === product_id &&
        device.revision === revision &&
        device.vendor_id === vendor_id,
    ),
  );
}
