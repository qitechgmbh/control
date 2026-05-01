import { Aquapath1ControlPage } from "../aquapath1/Aquapath1ControlPage";

import { useAquapath2 } from "./useAquapath2";
import React from "react";

export function Aquapath2ControlPage() {
  return <Aquapath1ControlPage useHook={useAquapath2} />;
}
