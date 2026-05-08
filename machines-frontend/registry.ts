import type { MachineModule } from "./module";
import type { MachineProperties, MachineIdentification } from "./types";
import { machineIdentificationEquals } from "./types";

const modules = import.meta.glob<{ default: MachineModule }>(
  "./*/*/index.ts",
  { eager: true },
);

export const allMachineModules: MachineModule[] = Object.values(modules)
  .map((m) => m.default)
  .sort((a, b) => a.slug.localeCompare(b.slug));

export const machineProperties: MachineProperties[] = allMachineModules.map(
  (m) => m.properties,
);

export function getMachineProperties(
  machine_identification: MachineIdentification,
): MachineProperties | undefined {
  return machineProperties.find((m) =>
    machineIdentificationEquals(m.machine_identification, machine_identification),
  );
}
