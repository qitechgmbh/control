import { MachineProperties, MachineIdentification, DeviceRole } from "./types";

export const VENDOR_QITECH = 0x0001;

export type VendorProperties = {
  id: number;
  name: string;
};

export const vendorProperties: VendorProperties[] = [
  {
    id: VENDOR_QITECH,
    name: "QiTech Industries GmbH",
  },
];

export function getVendorProperties(
  vendor: number,
): VendorProperties | undefined {
  return vendorProperties.find((v) => v.id === vendor);
}

export const winder2: MachineProperties = {
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
      role_label: "1x Stepper Spool",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b813052,
          revision: 0x100034,
        },
      ],
    },
    {
      role: 3,
      role_label: "1x Stepper Traverse",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x1a0000,
        },
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x190000,
        },
      ],
    },
    {
      role: 4,
      role_label: "1x Stepper Puller",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 0x1b773052,
          revision: 0x10001e,
        },
      ],
    },
  ],
};

export const extruder2: MachineProperties = {
  name: "Extruder",
  version: "V2",
  slug: "extruder2",
  icon: "qi:Extruder",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0004,
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
      role_label: "Digital Input",
      allowed_devices: [
        {
          vendor_id: 2,
          product_id: 65679442,
          revision: 1179648,
        },
      ],
    },
    {
      role: 2,
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
      role: 3,
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
      role: 4,
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
      role: 5,
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

export const laser1: MachineProperties = {
  name: "Laser",
  version: "V1",
  slug: "laser1",
  icon: "lu:Sun",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0006,
  },
  device_roles: [],
};

export const mock1: MachineProperties = {
  name: "Mock",
  version: "V1",
  slug: "mock1",
  icon: "lu:FlaskConical",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0007,
  },
  device_roles: [],
};

export const machineProperties: MachineProperties[] = [
  winder2,
  extruder2,
  laser1,
  mock1,
];

export const getMachineProperties = (
  machine_identification: MachineIdentification,
) => {
  return machineProperties.find(
    (m) =>
      m.machine_identification.vendor === machine_identification.vendor &&
      m.machine_identification.machine === machine_identification.machine,
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
