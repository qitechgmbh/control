/**
 * Test machine namespace
 *
 * Compare to electron/src/machines/testmachine/testMachineNamespace.ts (~77 lines)
 * This is ~25 lines including the types.
 */

import { createNamespace, machineNamespacePath } from "../lib/socketio";

export type TestMachineState = {
  led_on: boolean[] | null;
};

export function createTestMachineNamespace(
  vendor: number,
  machine: number,
  serial: number,
) {
  return createNamespace<TestMachineState>(
    machineNamespacePath(vendor, machine, serial),
    (event, set) => {
      if (event.name === "StateEvent") {
        set("led_on", event.data.led_on);
      }
    },
    { led_on: null },
  );
}
