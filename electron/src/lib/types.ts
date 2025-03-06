export type MachineIdentification = {
  vendor: number;
  serial: number;
  machine: number;
};

export type Option<T> = T | null;

export type MachineDeviceIdentification = {
  machine_identification: MachineIdentification;
  role: number;
  subdevice_index: number;
};
