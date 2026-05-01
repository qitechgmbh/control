import { Aquapath1GraphPage } from "../aquapath1/Aquapath1Graph";

import { useAquapath2 } from "./useAquapath2";
import React from "react";

export function Aquapath2GraphPage() {
  return <Aquapath1GraphPage useHook={useAquapath2} />;
}
