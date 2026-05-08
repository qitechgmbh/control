import type { MachineModule } from "../../module";
import { aquapath1 } from "./properties";
import { Aquapath1Page } from "./Aquapath1Page";
import { Aquapath1ControlPage } from "./Aquapath1ControlPage";
import { Aquapath1GraphPage } from "./Aquapath1Graph";
import { Aquapath1SettingsPage } from "./Aquapath1Settings";

const aquapath1Module: MachineModule = {
  slug: "aquapath1",
  properties: aquapath1,
  route: {
    path: "aquapath1/$serial",
    component: Aquapath1Page,
    children: [
      { path: "control", component: Aquapath1ControlPage },
      { path: "graph", component: Aquapath1GraphPage },
      { path: "settings", component: Aquapath1SettingsPage },
    ],
  },
};

export default aquapath1Module;
