import { useLaser1 as useRealLaser1 } from "./laser1/useLaser1";

export function useSafeLaser1() {
  const { state } = useRealLaser1();

  return {
    state: state ?? {
      laser_state: {
        in_tolerance: true,
      },
    },
  };
}

