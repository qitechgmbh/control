export type DeviceRole = {
  role: number;
  role_label: string;
  allowed_devices: EthercatDevice[];
};

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
  name: string;
  vendor_id: 0x0001;
  machine_id: number;
  device_roles: DeviceRole[];
};

export const VENDOR_QITECH = 0x0001;

export const machinePresets: MachinePreset[] = [
  {
    name: "Winder V1",
    vendor_id: VENDOR_QITECH,
    machine_id: 0x0001,
    device_roles: [
      {
        role: 0,
        role_label: "Buskoppler",
        allowed_devices: [
          {
            product_id: 0x44c2c52,
            revision: 0x120000,
          },
        ],
      },
      {
        role: 1,
        role_label: "2x Digitalausgang",
        allowed_devices: [
          {
            product_id: 0x7d23052,
            revision: 0x110000,
          },
        ],
      },
      {
        role: 2,
        role_label: "1x Analogeingang",
        allowed_devices: [
          {
            product_id: 0xbb93052,
            revision: 0x160000,
          },
        ],
      },
      {
        role: 3,
        role_label: "1x Pulszug Traverse",
        allowed_devices: [
          {
            product_id: 0x9d93052,
            revision: 0x3f80018,
          },
        ],
      },
      {
        role: 4,
        role_label: "2x Pulszug",
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
