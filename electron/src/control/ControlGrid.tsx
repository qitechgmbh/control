import { cva } from "class-variance-authority";
import React from "react";

type Props = {
  children?: React.ReactNode;
  columns?: 2 | 3;
};

export function ControlGrid({ children, columns = 3 }: Props) {
  return (
    <div id="grid" className={controlGridStyle({ columns })}>
      {children}
    </div>
  );
}

const controlGridStyle = cva(
  "grid w-full auto-cols-fr grid-cols-1 gap-6 lg:grid-cols-2",
  {
    variants: {
      columns: {
        2: "xl:grid-cols-2",
        3: "xl:grid-cols-3",
      },
    },
    defaultVariants: {
      columns: 3,
    },
  },
);
