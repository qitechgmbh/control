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
        <div className="absolute bg-white -top-4 -right-0 z-0">
          <div className="h-4 w-4 bg-neutral-200 rounded-br-full"></div>
        </div>
      )}
      {rightBottom && (
        <div className="absolute bg-white -bottom-4 -right-0 z-0">
          <div className="h-4 w-4 bg-neutral-200 rounded-tr-full"></div>
        </div>
      )}
      {bottomLeft && (
        <div className="absolute bg-white bottom-0 -left-4 z-0">
          <div className="h-4 w-4 bg-neutral-200 rounded-br-full"></div>
        </div>
      )}
      {bottomRight && (
        <div className="absolute bg-white bottom-0 -right-4 z-0">
          <div className="h-4 w-4 bg-neutral-200 rounded-bl-full"></div>
        </div>
      )}
    </>
  );
}
