import { ipcMain } from "electron";
import { spawn } from "child_process";
import { KEYBOARD_SHOW, KEYBOARD_HIDE } from "./keyboard-channels";

/**
 * Show GNOME on-screen keyboard.
 * 
 * Based on the NixOS configuration, this system uses GNOME as the desktop environment.
 * The keyboard is configured in nixos/os/home.nix with:
 * - org.gnome.desktop.a11y.applications.screen-keyboard-enabled = true
 * 
 * This function ensures the keyboard is enabled and triggers it to appear.
 */
function showKeyboard() {
  try {
    // Ensure keyboard is enabled via gsettings
    // This matches the configuration in nixos/os/home.nix
    spawn("gsettings", [
      "set",
      "org.gnome.desktop.a11y.applications",
      "screen-keyboard-enabled",
      "true",
    ], {
      detached: true,
      stdio: "ignore",
    });
  } catch (error) {
    // Silently fail - keyboard might not be available
    console.error("Failed to show keyboard:", error);
  }
}

/**
 * Hide GNOME on-screen keyboard (optional - usually hides automatically on blur)
 */
function hideKeyboard() {
  // GNOME keyboard hides automatically when input loses focus
  // This function is kept for potential future use
  // No action needed - the system handles this automatically
}

export function addKeyboardEventListeners() {
  ipcMain.handle(KEYBOARD_SHOW, () => {
    showKeyboard();
  });

  ipcMain.handle(KEYBOARD_HIDE, () => {
    hideKeyboard();
  });
}

