import { aquapath2 } from "@/machines/properties";
import { aquapath2SerialRoute } from "@/routes/routes";

import { useAquapathBase } from "../aquapath1/useAquapath";

export function useAquapath2() {
  const { serial: serialString } = aquapath2SerialRoute.useParams();
  return useAquapathBase(serialString, aquapath2);
}
