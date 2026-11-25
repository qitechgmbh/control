export const PRESETS_READ = "presets:read";
export const PRESETS_WRITE = "presets:write";

export type PresetsReadResult =
  | { status: "ok"; content: string }
  | { status: "missing" }
  | { status: "error"; error: string };

export type PresetsWriteResult =
  | { status: "ok" }
  | { status: "error"; error: string };
