export type MachineIdentificationUnique = {
  vendor: number;
  serial: number;
  machine: number;
};

export type Option<T> = T | null;

export type Result<T, E> =
  | {
      Ok: T;
    }
  | {
      Err: E;
    };

export type MachineDeviceIdentification = {
  machine_identification_unique: MachineIdentificationUnique;
  role: number;
  subdevice_index: number;
};
