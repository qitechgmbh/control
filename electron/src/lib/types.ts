export type MachineIdentification = {
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
  machine_identification: MachineIdentification;
  role: number;
  subdevice_index: number;
};
