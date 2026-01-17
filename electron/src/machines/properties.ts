import {
  MachineProperties,
  MachineIdentification,
  DeviceRole,
  machineIdentificationEquals,
} from "./types";

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
        {
          vendor_id: 2,
          product_id: 0x7d23052,
          revision: 0x120000,
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

export const buffer1: MachineProperties = {
  name: "Buffer",
  version: "V1",
  slug: "buffer1",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0008,
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
      role: 2,
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

export const testmachine: MachineProperties = {
  name: "TestMachine",
  version: "V1",
  slug: "testmachine",
  icon: "lu:Disc3",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0033,
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
      role_label: "EL2004",
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
export const wagoPower1: MachineProperties = {
  name: "WAGO Power",
  version: "V1",
  slug: "wago_power1",
  icon: "lu:PlugZap",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x000a,
  },
  device_roles: [],
};

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

export const ip20TestMachine: MachineProperties = {
  name: "IP20 Test",
  version: "V1",
  slug: "ip20testmachine",
  icon: "lu:ToggleLeft",
  machine_identification: {
    vendor: VENDOR_QITECH,
    machine: 0x0034,
  },
  device_roles: [
    {
      role: 0,
      role_label: "IP20-EC-DI8-DO8",
      allowed_devices: [
        {
          vendor_id: 0x741,
          product_id: 0x117b6722,
          revision: 0x1,
        },
      ],
    },
  ],
};

export const machineProperties: MachineProperties[] = [
  winder2,
  extruder3,
  extruder2,
  laser1,
  mock1,
  buffer1,
  wagoPower1,
  aquapath1,
  testmachine,
  analogInputTestMachine,
  ip20TestMachine,
];

export const getMachineProperties = (
  machine_identification: MachineIdentification,
) => {
  return machineProperties.find((m) =>
    machineIdentificationEquals(
      m.machine_identification,
      machine_identification,
    ),
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
