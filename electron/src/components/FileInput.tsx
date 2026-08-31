import React, { useId } from "react";
import { Icon, IconName } from "./Icon";

export type FileInputProps = {
  children: React.ReactNode;
  accept?: string;
  icon?: IconName;
  onFile?: (files: File) => void;
  onFiles?: (files: FileList) => void;
};

export const FileInput = ({
  children,
  icon,
  accept,
  onFiles,
  onFile,
}: FileInputProps) => {
  const id = useId();

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;

    if (!files || files.length == 0) {
      return;
    }

    if (onFile && files.length == 1) {
      onFile(files[0]);
    }

    if (onFiles) {
      onFiles(files);
    }
  };

  return (
    <label
      htmlFor={id}
      className="bg-primary text-primary-foreground item-center flex w-full cursor-pointer flex-row justify-center gap-2 rounded-md px-6 py-6 select-none"
    >
      <Icon name={icon} className="size-6" />
      {children}
      <input
        id={id}
        type="file"
        accept={accept}
        onChange={handleFileChange}
        multiple={!!onFiles}
        className="hidden"
      />
    </label>
  );
};

export type JsonFileInputProps = {
  children: React.ReactNode;
  icon?: IconName;
  onJson: (json: any) => void;
};

export function JsonFileInput({ children, icon, onJson }: JsonFileInputProps) {
  const handleFile = (file: File) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      try {
        const text = e.target?.result as string;
        const json = JSON.parse(text);
        onJson(json);
      } catch (err) {
        console.error("Invalid JSON file:", err);
      }
    };

    reader.readAsText(file);
  };

  return (
    <FileInput accept=".json,application/json" onFile={handleFile} icon={icon}>
      {children}
    </FileInput>
  );
}
