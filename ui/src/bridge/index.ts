import type { NativeBridge } from "./types";
import { stubBridge } from "./stub";

let bridge: NativeBridge = stubBridge;

export function setBridge(b: NativeBridge): void {
  bridge = b;
}

export function getBridge(): NativeBridge {
  return bridge;
}

export type {
  NativeBridge,
  EnvironmentInfo,
  NixOSGeneration,
  UpdateExecuteParams,
  UpdateStepParams,
  ThemeMode,
} from "./types";
