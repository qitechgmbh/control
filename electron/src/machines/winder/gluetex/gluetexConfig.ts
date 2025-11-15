/**
 * Configuration helper for Gluetex machine settings
 */

const GLUETEX_XL_MODE_KEY = "gluetex_xl_mode";

export const GLUETEX_TRAVERSE_MAX_STANDARD = 180; // mm
export const GLUETEX_TRAVERSE_MAX_XL = 385; // mm

/**
 * Get the XL mode setting from localStorage
 * @returns true if XL mode is enabled, false otherwise
 */
export function getGluetexXLMode(): boolean {
  const stored = localStorage.getItem(GLUETEX_XL_MODE_KEY);
  return stored === "true";
}

/**
 * Set the XL mode setting in localStorage
 * @param enabled - true to enable XL mode, false to disable
 */
export function setGluetexXLMode(enabled: boolean): void {
  localStorage.setItem(GLUETEX_XL_MODE_KEY, enabled.toString());
}

/**
 * Get the maximum traverse limit based on XL mode
 * @returns the maximum traverse limit in mm
 */
export function getGluetexTraverseMax(): number {
  return getGluetexXLMode()
    ? GLUETEX_TRAVERSE_MAX_XL
    : GLUETEX_TRAVERSE_MAX_STANDARD;
}
