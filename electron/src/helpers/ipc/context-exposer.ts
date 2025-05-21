import { exposeThemeContext } from "./theme/theme-context";
import { exposeWindowContext } from "./window/window-context";
import { exposeEnvironmentContext } from "./environment/environment-context";
import { exposeUpdateContext } from "./update/update-context";
import { exposeGnomeContext } from "./gnome/gnome-context";

export default function exposeContexts() {
  exposeWindowContext();
  exposeThemeContext();
  exposeEnvironmentContext();
  exposeUpdateContext();
  exposeGnomeContext();
}
