export const EXPORT_SAVE_FILE = "export:save-file";
export const EXPORT_EJECT_USB = "export:eject-usb";

export type SaveFileParams = {
  suggestedName: string;
  filters?: { name: string; extensions: string[] }[];
  content: string;
  encoding: "utf8" | "base64";
};

export type SaveFileResult = {
  success: boolean;
  error?: string;
  filePath?: string;
  isRemovable?: boolean;
  mountPath?: string;
};
