import React from "react";

type Props = {
  children?: React.ReactNode;
  columns?: number;
};

export function ControlGrid({ children, columns = 3 }: Props) {
  return (
    <div
      id="grid"
      className={`grid w-full auto-cols-fr grid-cols-1 gap-6 lg:grid-cols-2 xl:grid-cols-${columns}`}
    >
      {children}
    </div>
  );
}
