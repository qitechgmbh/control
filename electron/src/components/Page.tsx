import React from "react";

type Props = {
  children: React.ReactNode;
  className?: string;
};

export function Page({ children, className }: Props) {
  return (
    <div className={`flex w-full flex-col gap-6 p-6 ${className || ""}`}>
      {children}
    </div>
  );
}
