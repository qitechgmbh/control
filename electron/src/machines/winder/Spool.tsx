import React, { useEffect } from "react";

type Props = {
  rpm?: number;
};

export function Spool({ rpm }: Props) {
  const [rotationState, setRotationState] = React.useState({
    rotations: 0,
    lastRpm: undefined as number | undefined,
    lastRpmTime: undefined as number | undefined,
  });

  useEffect(() => {
    if (rpm !== undefined) {
      const now = Date.now();
      // Calculate the new rotation based on the RPM and time elapsed
      const dt = (now - (rotationState.lastRpmTime || now)) / 1000; // Convert ms to seconds
      const newRotations = ((rpm / 60) * dt) % 1; // Convert RPM to rotations per second
      setRotationState({
        rotations: (rotationState.rotations + newRotations) % 1, // Keep it within 0-1
        lastRpm: rpm,
        lastRpmTime: now,
      });
    }
  }, [rpm]);

  const { rotations } = rotationState;

  return (
    <div className="flex w-full justify-center">
      <div
        className="aspect-square h-32"
        style={{
          transform: `rotate(-${rotations * 360}deg)`,
        }}
      >
        <svg
          viewBox="0 0 512 512"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect x="221" y="8" width="70" height="182" fill="black" />
          <rect
            x="490.617"
            y="351"
            width="70"
            height="182"
            transform="rotate(120 490.617 351)"
            fill="black"
          />
          <rect
            x="51"
            y="403.033"
            width="70"
            height="182"
            transform="rotate(-120 51 403.033)"
            fill="black"
          />
          <path
            fillRule="evenodd"
            clipRule="evenodd"
            d="M256.498 511.998C398.159 511.998 512.998 397.159 512.998 255.498C512.998 113.837 398.159 -1.00195 256.498 -1.00195C114.837 -1.00195 -0.00195312 113.837 -0.00195312 255.498C-0.00195312 397.159 114.837 511.998 256.498 511.998ZM255.5 447C361.815 447 448 360.815 448 254.5C448 148.185 361.815 62 255.5 62C149.185 62 63 148.185 63 254.5C63 360.815 149.185 447 255.5 447Z"
            fill="black"
          />

          <path
            d="M255.5 381C324.812 381 381 324.812 381 255.5C381 186.188 324.812 130 255.5 130C186.188 130 130 186.188 130 255.5C130 324.812 186.188 381 255.5 381ZM256 319C290.794 319 319 290.794 319 256C319 221.206 290.794 193 256 193C221.206 193 193 221.206 193 256C193 290.794 221.206 319 256 319Z"
            fill="black"
          />
          <circle
            cx="256"
            cy="255"
            r="50"
            fill="black"
            className="fill-gray-300"
          />
        </svg>
      </div>
    </div>
  );
}
