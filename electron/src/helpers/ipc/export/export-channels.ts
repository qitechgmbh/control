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

export type ChoosePathResult =
  | { canceled: true }
  | { canceled: false; filePath: string };

/** Write content the renderer already has to a path from `choosePath`. */
export type WriteFileParams = {
  filePath: string;
  content: string;
  encoding: "utf8" | "base64";
};

/**
 * POST `body` as JSON to `url` and stream the response into `filePath`, without
 * passing the data through the renderer.
 */
export type DownloadParams = {
  filePath: string;
  url: string;
  body: unknown;
};
