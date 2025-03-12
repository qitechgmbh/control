import {
  getVendorPreset,
  getMachinePreset,
  MachinePreset,
  VendorPreset,
} from "@/machines/types";
import {
  EthercatSetupEventMachineInfo,
  useSocketioEthercatSetupEvent,
} from "./useSocketio";

type UseMachine = {
  machine_identification: EthercatSetupEventMachineInfo["machine_identification"];
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
    event.data?.machine_infos
      .filter((machine) => machine.error === null)
      .map((machine) => {
        const machinePreset = getMachinePreset(machine.machine_identification);
        const vendorPreset = getVendorPreset(machinePreset!.vendor_id);
        if (!machinePreset || !vendorPreset) {
          return undefined;
        }
        return {
          machine_identification: machine.machine_identification,
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
