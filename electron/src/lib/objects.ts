export function deepEquals(
  a: object | undefined,
  b: object | undefined,
): boolean {
  if (a === b) {
    return true;
  }

  if (!a || typeof a !== "object" || !b || typeof b !== "object") {
    return a === b;
  }

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return false;
    }

    for (let i = 0; i < a.length; i++) {
      if (!deepEquals(a[i], b[i])) {
        return false;
      }
    }

    return true;
  }

  if (Array.isArray(a) || Array.isArray(b)) {
    return false;
  }

  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);
  const aRec = a as Record<string, any>;
  const bRec = b as Record<string, any>;

  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (!(key in bRec)) {
      return false;
    }

    if (!deepEquals(aRec[key], bRec[key])) {
      return false;
    }
  }

  return true;
}

/**
 * Checks if all keys in `subset` match corresponding values in `superset`.
 * Extra keys in `superset` are ignored. Empty objects in `subset` are
 * treated as matching anything.
 */
export function deepSubsetEquals(
  superset: object | undefined,
  subset: object | undefined,
): boolean {
  if (superset === subset) {
    return true;
  }

  if (
    !subset ||
    typeof subset !== "object" ||
    !superset ||
    typeof superset !== "object"
  ) {
    return superset === subset;
  }

  if (Array.isArray(superset) && Array.isArray(subset)) {
    if (superset.length !== subset.length) {
      return false;
    }
    for (let i = 0; i < subset.length; i++) {
      if (!deepSubsetEquals(superset[i], subset[i])) {
        return false;
      }
    }
    return true;
  }

  if (Array.isArray(superset) || Array.isArray(subset)) {
    return false;
  }

  const subRec = subset as Record<string, any>;
  const supRec = superset as Record<string, any>;
  const subKeys = Object.keys(subRec);

  // Empty object in subset matches anything
  if (subKeys.length === 0) {
    return true;
  }

  for (const key of subKeys) {
    if (!(key in supRec)) {
      // Key missing in superset — only ok if subset value is undefined
      if (subRec[key] !== undefined) {
        return false;
      }
      continue;
    }

    if (!deepSubsetEquals(supRec[key], subRec[key])) {
      return false;
    }
  }

  return true;
}
