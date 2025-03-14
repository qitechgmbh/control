// every device has multiple roles to indentify the function of a subdevice
// each role can only be given once

import { IconName } from "@/components/Icon";

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
  product_id: number;
  revision: number;
};

export type MachineIdentification = {
  vendor: number;
  serial: number;
  machine: number;
};

export type MachinePreset = {
  // displayable name
  name: string;
  // displayable version
  version: string;
  // path for IO routes
  slug: string;
  icon: IconName;
  // needs to be same as in the backend
  vendor_id: 0x0001;
  // needs to be same as in the backend
  machine_id: number;
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

export function getVendorPreset(vendor_id: number): VendorPreset | undefined {
  return vendorPresets.find((v) => v.id === vendor_id);
}

export const machinePresets: MachinePreset[] = [
  {
    name: "Winder",
    version: "V1",
    slug: "winder1",
    icon: "lu:Disc3",
    vendor_id: VENDOR_QITECH,
    machine_id: 0x0001,
    device_roles: [
      {
        role: 0,
        role_label: "Bus Coupler",
        allowed_devices: [
          {
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
            product_id: 0xbb93052,
            revision: 0x160000,
          },
        ],
      },
      {
        role: 3,
        role_label: "1x Pulsetrain Traverse",
        allowed_devices: [
          {
            product_id: 0x9d93052,
            revision: 0x3f80018,
          },
        ],
      },
      {
        role: 4,
        role_label: "2x Pulsetrain",
        allowed_devices: [
          {
            product_id: 0x9da3052,
            revision: 0x160000,
          },
        ],
      },
    ],
  },
];

export const getMachinePreset = (
  machine_identification: MachineIdentification,
) => {
  return machinePresets.find(
    (m) =>
      m.vendor_id === machine_identification.vendor &&
      m.machine_id === machine_identification.machine,
  );
};

export function filterAllowedDevices(
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
        device.product_id === product_id && device.revision === revision,
    ),
  );
}
