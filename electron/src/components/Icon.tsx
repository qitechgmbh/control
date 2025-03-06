import { icons as lucideIcons } from "lucide-react";
import React from "react";

// prefix keys with library to avoid conflicts with other icon libraries
export type IconName = `lu:${keyof typeof lucideIcons}`;

type Props = {
  size?: number;
  name?: IconName;
  className?: string;
};

export const Icon = ({ name, size, className }: Props) => {
  if (!name) {
    return null;
  }
  const library = name.split(":")[0];
  const rawIcon = name.split(":")[1];

  if (library === "lu" && rawIcon in lucideIcons) {
    const LucideIcon = lucideIcons[rawIcon as keyof typeof lucideIcons];
    return <LucideIcon size={size} className={className} />;
  }

  console.error(`Icon ${name} not found`, library, rawIcon, lucideIcons);

  return null;
};

export type IconNameMap = {
  [key: string]: IconName;
};
