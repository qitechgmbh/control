export function deepEquals<T>(a: T, b: T): boolean {
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
