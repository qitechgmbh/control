import {
  VendorProperties,
  MachineProperties,
  MachineIdentificationUnique,
} from "@/machines/types";
import { useMainNamespace } from "./mainNamespace";
import {
  getVendorProperties,
  getMachineProperties,
} from "@/machines/properties";


type UseMachine = {
  machine_identification_unique: MachineIdentificationUnique;
  name: MachinePreset["name"];
  version: MachinePreset["version"];
  slug: MachinePreset["slug"];
  vendor: VendorPreset["name"];
  icon: MachinePreset["icon"];
};

// returns only valid machines
export function useMachines(): UseMachine[] {
  const { machines } = useMainNamespace();

  if (machines?.data)
    return (
      machines.data.machines
        .filter((machine) => machine.error === null)
        .map((machine) => {
          const machinePreset = getMachinePreset(
            machine.machine_identification_unique.machine_identification,
          );
          const vendorPreset = getVendorPreset(
            machinePreset!.machine_identification.vendor,
          );
          if (!machinePreset || !vendorPreset) {
            return undefined;
          }
          return {
            machine_identification_unique:
              machine.machine_identification_unique,
            name: machinePreset.name,
            version: machinePreset.version,
            slug: machinePreset.slug,
            vendor: vendorPreset.name,
            icon: machinePreset.icon,
          };
        })
        .filter((machine) => machine !== undefined) || []
    );

  return [];
}
