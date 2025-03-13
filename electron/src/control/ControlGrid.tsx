import React from "react";

type Props = {
  children?: React.ReactNode;
};

export function ControlGrid({ children }: Props) {
  return (
    <div id="grid" className="grid w-full auto-cols-fr grid-cols-3 gap-6">
      {children}
    </div>
  );
}
