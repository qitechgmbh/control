import { icons as lucideIcons } from "lucide-react";
import React from "react";
import { qitechIcons } from "./QitechIcon";

type QitechIconName = `qi:${keyof typeof qitechIcons}`;

type LucideIconName = `lu:${keyof typeof lucideIcons}`;

// prefix keys with library to avoid conflicts with other icon libraries
export type IconName = QitechIconName | LucideIconName;

type Props = {
  name?: IconName;
  className?: string;
};

export const Icon = ({ name, className }: Props) => {
  if (!name) {
    return null;
  }
  const library = name.split(":")[0];
  const rawIcon = name.split(":")[1];

  if (library === "lu" && rawIcon in lucideIcons) {
    const LucideIcon = lucideIcons[rawIcon as keyof typeof lucideIcons];
    return <LucideIcon className={"size-6 " + className} />;
  }

  if (library === "qi" && rawIcon in qitechIcons) {
    const QitechIcon = qitechIcons[rawIcon as keyof typeof qitechIcons];
    return <QitechIcon className={"size-6 " + className} />;
  }

  console.error(`Icon ${name} not found`, library, rawIcon, lucideIcons);

  return null;
};

export type IconNameMap = {
  [key: string]: IconName;
};
