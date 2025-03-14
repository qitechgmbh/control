import React from "react";

type Props = {
  title: string;
  children?: React.ReactNode;
};

export function SectionTitle({ children, title }: Props) {
  return (
    <div className="flex gap-4">
      <h1 className="text-2xl">{title}</h1>
      {children}
    </div>
  );
}
