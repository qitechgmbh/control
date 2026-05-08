import type { MachineModule } from "../../module";
import { wagoPower1 } from "./properties";
import { WagoPower1Page } from "./WagoPower1Page";
import { WagoPower1ControlPage } from "./WagoPower1ControlPage";

const wagoPower1Module: MachineModule = {
  slug: "wago_power1",
  properties: wagoPower1,
  route: {
    path: "wago_power1/$serial",
    component: WagoPower1Page,
    children: [{ path: "control", component: WagoPower1ControlPage }],
  },
};

export default wagoPower1Module;
