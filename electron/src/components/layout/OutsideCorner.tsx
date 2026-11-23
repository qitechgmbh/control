import React from "react";

export type OutsideCornerProps = {
  rightTop?: boolean;
  rightBottom?: boolean;
  bottomLeft?: boolean;
  bottomRight?: boolean;
};

export function OutsideCorner({
  rightTop,
  rightBottom,
  bottomLeft,
  bottomRight,
}: OutsideCornerProps) {
  return (
    <>
      {rightTop && (
        <div className="absolute -top-4 -right-0 z-0 bg-white">
          <div className="h-4 w-4 rounded-br-full bg-neutral-200"></div>
        </div>
      )}
      {rightBottom && (
        <div className="absolute -right-0 -bottom-4 z-0 bg-white">
          <div className="h-4 w-4 rounded-tr-full bg-neutral-200"></div>
        </div>
      )}
      {bottomLeft && (
        <div className="absolute bottom-0 -left-4 z-0 bg-white">
          <div className="h-4 w-4 rounded-br-full bg-neutral-200"></div>
        </div>
      )}
      {bottomRight && (
        <div className="absolute -right-4 bottom-0 z-0 bg-white">
          <div className="h-4 w-4 rounded-bl-full bg-neutral-200"></div>
        </div>
      )}
    </>
  );
}
