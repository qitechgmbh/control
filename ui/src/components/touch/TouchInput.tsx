import { ComponentProps } from "react";
import { Input } from "../ui/input";
import React from "react";
import { cva } from "class-variance-authority";

type Props = {} & ComponentProps<typeof Input>;

export function TouchInput({ className, ...props }: Props) {
  const inputStyle = cva([
    "px-0",
    "py-8",
    "w-min",
    "align-middle",
    "text-center",
    "text-4xl",
    "md:text-4xl",
    "font-bold w-56",
  ]);
  return <Input className={inputStyle({ className })} {...props} />;
}
