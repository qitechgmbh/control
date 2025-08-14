// every device has multiple roles to identify the function of a subdevice
// each role can only be given once

import { IconName } from "@/components/Icon";
import { rustEnumSchema } from "@/lib/types";
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

export const machineIdentification = z.object({
  vendor: z.number(),
  machine: z.number(),
});

export type MachineIdentification = z.infer<typeof machineIdentification>;

export function machineIdentificationEquals(
  a: MachineIdentification,
  b: MachineIdentification,
): boolean {
  return a.vendor === b.vendor && a.machine === b.machine;
}

export const machineIdentificationUnique = z.object({
  machine_identification: machineIdentification,
  serial: z.number(),
});

export type MachineIdentificationUnique = z.infer<
  typeof machineIdentificationUnique
>;

export const deviceMachineIdentification = z.object({
  machine_identification_unique: machineIdentificationUnique,
  role: z.number(),
});

export type DeviceMachineIdentification = z.infer<
  typeof deviceMachineIdentification
>;

// Define hardware identification schemas
export const deviceHardwareIdentificationEthercatSchema = z.object({
  subdevice_index: z.number().int(),
});

export type DeviceHardwareIdentificationEthercat = z.infer<
  typeof deviceHardwareIdentificationEthercatSchema
>;

export const deviceHardwareIdentificationSchema = rustEnumSchema({
  Ethercat: deviceHardwareIdentificationEthercatSchema,
});

export type DeviceHardwareIdentification = z.infer<
  typeof deviceHardwareIdentificationSchema
>;

export type MachineProperties = {
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

export const deviceIdentification = z.object({
  device_machine_identification: deviceMachineIdentification.nullable(),
  device_hardware_identification: deviceHardwareIdentificationSchema,
});

export type DeviceIdentification = z.infer<typeof deviceIdentification>;
