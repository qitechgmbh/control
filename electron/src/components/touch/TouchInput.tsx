import { ComponentProps } from "react";
import { Input } from "../ui/input";
import React from "react";
import { cva } from "class-variance-authority";

type Props = {} & ComponentProps<typeof Input>;

export function TouchInput({ className, ...props }: Props) {
  return <Input className={inputStyle({ class: className })} {...props} />;
}

const inputStyle = cva([
  "px-0",
  "py-9",
  "w-min",
  "align-middle",
  "text-center",
  "text-[3rem]",
  "md:text-[3rem]",
  "font-bold w-56",
]);
