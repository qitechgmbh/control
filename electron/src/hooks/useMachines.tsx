import {
  getVendorPreset,
  getMachinePreset,
  MachinePreset,
  VendorPreset,
} from "@/machines/types";
import {
  EthercatSetupEventMachine,
  useSocketioEthercatSetupEvent,
} from "./useSocketio";

type UseMachine = {
  machine_identification_unique: EthercatSetupEventMachine["machine_identification_unique"];
  name: MachinePreset["name"];
  version: MachinePreset["version"];
  slug: MachinePreset["slug"];
  vendor: VendorPreset["name"];
  icon: MachinePreset["icon"];
};

// returns only valid mahcines
export function useMachines(): UseMachine[] {
  const event = useSocketioEthercatSetupEvent();
  return (
    event.data?.machines
      .filter((machine) => machine.error === null)
      .map((machine) => {
        const machinePreset = getMachinePreset(
          machine.machine_identification_unique,
        );
        const vendorPreset = getVendorPreset(
          machinePreset!.machine_identification.vendor,
        );
        if (!machinePreset || !vendorPreset) {
          return undefined;
        }
        return {
          machine_identification_unique: machine.machine_identification_unique,
          name: machinePreset.name,
          version: machinePreset.version,
          slug: machinePreset.slug,
          vendor: vendorPreset.name,
          icon: machinePreset.icon,
        };
      })
      .filter((machine) => machine !== undefined) || []
  );
}
