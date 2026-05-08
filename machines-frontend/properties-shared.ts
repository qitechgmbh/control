import {
  MachineIdentification,
  DeviceRole,
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
