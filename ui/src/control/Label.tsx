import React from "react";

type Props = {
  label: string;
  children: React.ReactNode;
};

export function Label({ label, children }: Props) {
  return (
    <div className="flex flex-col gap-1">
      <span>{label}</span>
      <div className="flex flex-col flex-wrap gap-4">{children}</div>
    </div>
  );
}
