import { Aquapath1SettingsPage } from "../aquapath1/Aquapath1Settings";

import { useAquapath2 } from "./useAquapath2";
import React from "react";

export function Aquapath2SettingsPage() {
  return <Aquapath1SettingsPage useHook={useAquapath2} />;
}
