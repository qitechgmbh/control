import React from "react";

type Props = {
  degrees?: number;
};

export function TensionArm({ degrees }: Props) {
  return (
    <div className="flex w-full justify-center">
      <div
        className={`aspect-square h-32`}
        style={{
          transform: `rotate(${degrees ?? 0}deg)`,
        }}
      >
        <svg
          viewBox="0 0 512 512"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            x="230"
            width="52"
            height="301"
            rx="26"
            className="fill-black"
          />
          <circle cx="255.5" cy="386.5" r="125.5" className="fill-gray-200" />
          <circle cx="256" cy="387" r="63" className="fill-black" />
        </svg>
      </div>
    </div>
  );
}
