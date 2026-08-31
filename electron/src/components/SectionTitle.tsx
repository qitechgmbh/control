import React from "react";

type Props = {
  title: string;
  children?: React.ReactNode;
  right?: React.ReactNode;
};

export function SectionTitle({ children, title, right }: Props) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-4">
        <h1 className="text-2xl">{title}</h1>
        {children}
      </div>
      {right}
    </div>
  );
}
