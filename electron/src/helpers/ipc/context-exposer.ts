import { exposeThemeContext } from "./theme/theme-context";
import { exposeWindowContext } from "./window/window-context";
import { exposeEnvironmentContext } from "./environment/environment-context";
import { exposeUpdateContext } from "./update/update-context";
import { exposeTroubleshootContext } from "./troubleshoot/troubleshoot-context";
import { exposeNixOSContext } from "./nixos/nixos-context";

export default function exposeContexts() {
  exposeWindowContext();
  exposeThemeContext();
  exposeEnvironmentContext();
  exposeUpdateContext();
  exposeTroubleshootContext();
  exposeNixOSContext();
}
