import { exposeThemeContext } from "./theme/theme-context";
import { exposeWindowContext } from "./window/window-context";
import { exposeEnvironmentContext } from "./environment/environment-context";

export default function exposeContexts() {
  exposeWindowContext();
  exposeThemeContext();
  exposeEnvironmentContext();
}
