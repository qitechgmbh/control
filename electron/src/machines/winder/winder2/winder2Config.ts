/**
 * Configuration helper for Winder2 machine settings
 */

const WINDER2_XL_MODE_KEY = "winder2_xl_mode";

export const WINDER2_TRAVERSE_MAX_STANDARD = 180; // mm
export const WINDER2_TRAVERSE_MAX_XL = 385; // mm

/**
 * Get the XL mode setting from localStorage
 * @returns true if XL mode is enabled, false otherwise
 */
export function getWinder2XLMode(): boolean {
  const stored = localStorage.getItem(WINDER2_XL_MODE_KEY);
  return stored === "true";
}

/**
 * Set the XL mode setting in localStorage
 * @param enabled - true to enable XL mode, false to disable
 */
export function setWinder2XLMode(enabled: boolean): void {
  localStorage.setItem(WINDER2_XL_MODE_KEY, enabled.toString());
}

/**
 * Get the maximum traverse limit based on XL mode
 * @returns the maximum traverse limit in mm
 */
export function getWinder2TraverseMax(): number {
  return getWinder2XLMode()
    ? WINDER2_TRAVERSE_MAX_XL
    : WINDER2_TRAVERSE_MAX_STANDARD;
}
