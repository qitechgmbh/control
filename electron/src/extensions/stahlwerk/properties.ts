import { VENDOR_QITECH } from "../../machines/properties";
import { MachineProperties } from "../../machines/types";

export const ff01_mock: MachineProperties = 
{
  name:     "FF01",
  version:  "V1",
  slug:     "stahlwerk_ff01",
  icon:     "lu:FlaskConical",
  machine_identification: 
  {
    vendor:  VENDOR_QITECH,
    machine: 0x0007,
  },
  device_roles: [],
};

export const machineProperties: MachineProperties[] = [
    ff01_mock
];