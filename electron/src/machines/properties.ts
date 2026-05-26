import {
  MachineProperties,
  MachineIdentification,
  DeviceRole,
  machineIdentificationEquals,
} from "./types";

import { VENDOR_QITECH } from "./types";
export { VENDOR_QITECH } from "./types";

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

// ─── Main machines ────────────────────────────────────────────────
export { winder2_spool_7031 } from "./winder/winder2_7031/properties";
export { winder2 } from "./winder/winder2/properties";
export { extruder3 } from "./extruder/extruder3/properties";
export { extruder2 } from "./extruder/extruder2/properties";
export { laser1 } from "./laser/laser1/properties";
export { aquapath1 } from "./aquapath/aquapath1/properties";
export { buffer1 } from "./buffer/buffer1/properties";
export { wagoPower1 } from "./wago_power/wago_power1/properties";
export { wagoSerial } from "./wago_serial/properties";

// ─── Test / minimal machines ──────────────────────────────────────
export { mock1 } from "./minimal_machines/mock/mock1/properties";
export { testmachine } from "./minimal_machines/testmachine/properties";
export { digitalInputTestMachine } from "./minimal_machines/digitalinputtestmachine/properties";
export { wago750430DiMachine } from "./minimal_machines/wago750430dimachine/properties";
export { wago8chDioTestMachine } from "./minimal_machines/wago8chdiotestmachine/properties";
export { analogInputTestMachine } from "./minimal_machines/analoginputtestmachine/properties";
export { wagoAiTestMachine } from "./minimal_machines/wagoaitestmachine/properties";
export { wagoDoTestMachine } from "./minimal_machines/wagodotestmachine/properties";
export { ip20TestMachine } from "./minimal_machines/ip20testmachine/properties";
export { testmachinestepper } from "./minimal_machines/testmachinestepper/properties";
export { TestMotor } from "./minimal_machines/motor_test_machine/properties";
export { wago750_531Machine } from "./minimal_machines/wago750531machine/properties";
export { wago750_553Machine } from "./minimal_machines/wago750553machine/properties";
export { wago750_501TestMachine } from "./minimal_machines/wago750501testmachine/properties";
export { wago750460Machine } from "./minimal_machines/wago750460machine/properties";
export { bottlecapsTestMachine } from "./minimal_machines/bottlecaps_test_machine/properties";

// Re-import for the arrays below
import { winder2_spool_7031 as _winder2_spool_7031 } from "./winder/winder2_7031/properties";
import { winder2 as _winder2 } from "./winder/winder2/properties";
import { extruder3 as _extruder3 } from "./extruder/extruder3/properties";
import { extruder2 as _extruder2 } from "./extruder/extruder2/properties";
import { laser1 as _laser1 } from "./laser/laser1/properties";
import { aquapath1 as _aquapath1 } from "./aquapath/aquapath1/properties";
import { buffer1 as _buffer1 } from "./buffer/buffer1/properties";
import { wagoPower1 as _wagoPower1 } from "./wago_power/wago_power1/properties";
import { wagoSerial as _wagoSerial } from "./wago_serial/properties";
import { mock1 as _mock1 } from "./minimal_machines/mock/mock1/properties";
import { testmachine as _testmachine } from "./minimal_machines/testmachine/properties";
import { digitalInputTestMachine as _digitalInputTestMachine } from "./minimal_machines/digitalinputtestmachine/properties";
import { wago750430DiMachine as _wago750430DiMachine } from "./minimal_machines/wago750430dimachine/properties";
import { wago8chDioTestMachine as _wago8chDioTestMachine } from "./minimal_machines/wago8chdiotestmachine/properties";
import { analogInputTestMachine as _analogInputTestMachine } from "./minimal_machines/analoginputtestmachine/properties";
import { wagoAiTestMachine as _wagoAiTestMachine } from "./minimal_machines/wagoaitestmachine/properties";
import { wagoDoTestMachine as _wagoDoTestMachine } from "./minimal_machines/wagodotestmachine/properties";
import { ip20TestMachine as _ip20TestMachine } from "./minimal_machines/ip20testmachine/properties";
import { testmachinestepper as _testmachinestepper } from "./minimal_machines/testmachinestepper/properties";
import { TestMotor as _TestMotor } from "./minimal_machines/motor_test_machine/properties";
import { wago750_531Machine as _wago750_531Machine } from "./minimal_machines/wago750531machine/properties";
import { wago750_553Machine as _wago750_553Machine } from "./minimal_machines/wago750553machine/properties";
import { wago750_501TestMachine as _wago750_501TestMachine } from "./minimal_machines/wago750501testmachine/properties";
import { wago750460Machine as _wago750460Machine } from "./minimal_machines/wago750460machine/properties";
import { bottlecapsTestMachine as _bottlecapsTestMachine } from "./minimal_machines/bottlecaps_test_machine/properties";

// ─── Grouped machine lists ────────────────────────────────────────

/** Production machines shown first in the assignment dropdown. */
export const mainMachineProperties: MachineProperties[] = [
  _aquapath1,
  _buffer1,
  _extruder2,
  _extruder3,
  _laser1,
  _winder2,
  _winder2_spool_7031,
];

/** Test / development machines shown in a separate group. */
export const minimalMachineProperties: MachineProperties[] = [
  _analogInputTestMachine,
  _bottlecapsTestMachine,
  _digitalInputTestMachine,
  _ip20TestMachine,
  _mock1,
  _testmachine,
  _testmachinestepper,
  _TestMotor,
  _wago750430DiMachine,
  _wago750460Machine,
  _wago750_501TestMachine,
  _wago750_531Machine,
  _wago750_553Machine,
  _wago8chDioTestMachine,
  _wagoAiTestMachine,
  _wagoDoTestMachine,
  _wagoPower1,
  _wagoSerial,
];

/** All machines (flat list, kept for backward compatibility). */
export const machineProperties: MachineProperties[] = [
  ...mainMachineProperties,
  ...minimalMachineProperties,
];

// ─── Helpers ──────────────────────────────────────────────────────

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
