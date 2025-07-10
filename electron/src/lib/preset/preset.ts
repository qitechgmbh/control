import { MachineIdentification } from "@/machines/types";

export type Preset<T> = {
  id: number;
  name: string;
  lastModified: Date;
  machine_identification: MachineIdentification;
  schemaVersion: number;
  isLatestPreset?: boolean;
  data: Partial<T>;
};
