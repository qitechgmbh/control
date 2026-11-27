export const EXPORT_SAVE_FILE = "export:save-file";
export const EXPORT_EJECT_USB = "export:eject-usb";
export const EXPORT_CHOOSE_PATH = "export:choose-path";
export const EXPORT_WRITE_FILE = "export:write-file";
export const EXPORT_DOWNLOAD = "export:download";

export type FileFilter = { name: string; extensions: string[] };

export type SaveFileParams = {
  suggestedName: string;
  filters?: FileFilter[];
  content: string;
  encoding: "utf8" | "base64";
};

export type SaveFileResult = {
  success: boolean;
  error?: string;
  filePath?: string;
  /** All files written, when one export produced several. */
  filePaths?: string[];
  isRemovable?: boolean;
  mountPath?: string;
};

export type ChoosePathParams = {
  suggestedName: string;
  filters?: FileFilter[];
};

/** `error` is set when the dialog itself failed, not when the user cancelled. */
export type ChoosePathResult =
  | { canceled: true; error?: string }
  | { canceled: false; filePath: string };

/** Extension of a path from `choosePath`; the rest is the export's base path. */
export const EXPORT_EXTENSION = /\.(xlsx|csv|parquet)$/i;

/**
 * What may follow the base path in an export file: an optional per-machine `_suffix`
 * and one of the export extensions. Never a path separator.
 */
export const EXPORT_FILE_SUFFIX = /^(_[A-Za-z0-9-]+)?\.(xlsx|csv|parquet)$/;

/** Write bytes the renderer built (the Excel workbook) to an export file. */
export type WriteFileParams = {
  filePath: string;
  data: Uint8Array;
};

/**
 * Stream a history export (`POST /api/v1/machine/history/export`, request `body`)
 * into an export file, without passing the data through the renderer.
 */
export type DownloadParams = {
  filePath: string;
  body: unknown;
};
