import React from "react";

type Props = {
  children: React.ReactNode;
};

export function Page({ children }: Props) {
  return <div className="flex w-full flex-col gap-2 p-6">{children}</div>;
}
