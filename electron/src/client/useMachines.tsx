import {
  MachineProperties,
  MachineIdentificationUnique,
} from "@/machines/types";
import { useMainNamespace } from "./mainNamespace";
import {
  getVendorProperties,
  getMachineProperties,
  VendorProperties,
} from "@/machines/properties";

type UseMachine = {
  machine_identification_unique: MachineIdentificationUnique;
  name: MachineProperties["name"];
  version: MachineProperties["version"];
  slug: MachineProperties["slug"];
  vendor: VendorProperties["name"];
  icon: MachineProperties["icon"];
};

// returns only valid machines
export function useMachines(): UseMachine[] {
  const { machines } = useMainNamespace();

  if (machines?.data)
    return (
      machines.data.machines
        .filter((machine) => machine.error === null)
        .map((machine) => {
          const machinePreset = getMachineProperties(
            machine.machine_identification_unique.machine_identification,
          );
          const vendorPreset = getVendorProperties(
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
