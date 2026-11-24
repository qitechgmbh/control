import { exposeThemeContext } from "./theme/theme-context";
import { exposeWindowContext } from "./window/window-context";
import { exposeEnvironmentContext } from "./environment/environment-context";
import { exposeUpdateContext } from "./update/update-context";
import { exposeTroubleshootContext } from "./troubleshoot/troubleshoot-context";
import { exposeNixOSContext } from "./nixos/nixos-context";
import { exposeFileExportContext } from "./export/export-context";
import { exposePresetsContext } from "./presets/presets-context";

export default function exposeContexts() {
  exposeWindowContext();
  exposeThemeContext();
  exposeEnvironmentContext();
  exposeUpdateContext();
  exposeTroubleshootContext();
  exposeNixOSContext();
  exposeFileExportContext();
  exposePresetsContext();
}
