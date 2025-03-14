import { ComponentProps } from "react";
import { Input } from "../ui/input";
import React from "react";
import { useClassNameBuilder } from "@/helpers/style";

type Props = {} & ComponentProps<typeof Input>;

export function TouchInput({ className, ...props }: Props) {
  const inputStyle = useClassNameBuilder({
    base: "px-0 py-8 w-min align-middle text-center text-4xl md:text-4xl font-bold w-56",
  });
  return <Input className={inputStyle({ className })} {...props} />;
}
