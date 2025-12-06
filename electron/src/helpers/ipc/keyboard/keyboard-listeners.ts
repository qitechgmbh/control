import { ipcMain } from "electron";
import { spawn } from "child_process";
import { KEYBOARD_SHOW, KEYBOARD_HIDE } from "./keyboard-channels";

/**
 * Show system on-screen keyboard by detecting the desktop environment
 * and using the appropriate method for that environment.
 * 
 * This function tries to detect which desktop environment is running
 * and uses the appropriate command to show the keyboard.
 */
function showKeyboard() {
  const desktop = process.env.XDG_CURRENT_DESKTOP || process.env.DESKTOP_SESSION || "";
  const desktopLower = desktop.toLowerCase();

  try {
    // GNOME / GNOME Classic
    if (desktopLower.includes("gnome")) {
      // Ensure keyboard is enabled
      spawn("gsettings", [
        "set",
        "org.gnome.desktop.a11y.applications",
        "screen-keyboard-enabled",
        "true",
      ], {
        detached: true,
        stdio: "ignore",
      });
      return;
    }

    // KDE Plasma
    if (desktopLower.includes("kde") || desktopLower.includes("plasma")) {
      // KDE uses qdbus to trigger the keyboard
      spawn("qdbus", [
        "org.kde.plasmashell",
        "/PlasmaShell",
        "org.kde.PlasmaShell.evaluateScript",
        "plasmoid.byName('org.kde.plasma.keyboardlayout').show()",
      ], {
        detached: true,
        stdio: "ignore",
      }).on("error", () => {
        // Fallback: try to enable via kcmshell
        spawn("kcmshell5", ["kcm_keyboard"], {
          detached: true,
          stdio: "ignore",
        });
      });
      return;
    }

    // XFCE
    if (desktopLower.includes("xfce")) {
      // XFCE might use onboard or florence
      spawn("onboard", [], {
        detached: true,
        stdio: "ignore",
      }).on("error", () => {
        spawn("florence", [], {
          detached: true,
          stdio: "ignore",
        });
      });
      return;
    }

    // Generic Linux: Try common on-screen keyboards
    // Try onboard first (most common)
    spawn("onboard", [], {
      detached: true,
      stdio: "ignore",
    }).on("error", () => {
      // Try florence as fallback
      spawn("florence", [], {
        detached: true,
        stdio: "ignore",
      }).on("error", () => {
        // Try matchbox-keyboard as last resort
        spawn("matchbox-keyboard", [], {
          detached: true,
          stdio: "ignore",
        });
      });
    });
  } catch (error) {
    // Silently fail - keyboard might not be available
    console.error("Failed to show keyboard:", error);
  }
}

/**
 * Hide system on-screen keyboard (optional - usually hides automatically)
 */
function hideKeyboard() {
  // Most system keyboards hide automatically when input loses focus
  // This function is kept for potential future use
  try {
    // Try to kill common keyboard processes if needed
    spawn("pkill", ["-f", "onboard"], {
      detached: true,
      stdio: "ignore",
    }).on("error", () => {
      spawn("pkill", ["-f", "florence"], {
        detached: true,
        stdio: "ignore",
      });
    });
  } catch (error) {
    // Silently fail
  }
}

export function addKeyboardEventListeners() {
  ipcMain.handle(KEYBOARD_SHOW, () => {
    showKeyboard();
  });

  ipcMain.handle(KEYBOARD_HIDE, () => {
    hideKeyboard();
  });
}

