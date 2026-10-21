export const SEMVER_RE =
  /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z-.]+)?$/;

// Numeric identifiers compare numerically and always sort below
// alphanumeric ones; https://semver.org/#spec-item-11.
function compareIdentifier(a: string, b: string): number {
  const [numA, numB] = [/^\d+$/.test(a), /^\d+$/.test(b)];
  if (numA && numB) return Number(a) - Number(b);
  if (numA !== numB) return numA ? -1 : 1;
  return a < b ? -1 : a > b ? 1 : 0;
}

// Semver precedence for two SEMVER_RE matches. A pre-release
// sorts below its normal version, so "3.0.0" outranks "3.0.0-rc1".
export function compareSemver(a: RegExpExecArray, b: RegExpExecArray): number {
  for (let i = 1; i <= 3; i++) {
    if (a[i] !== b[i]) return Number(a[i]) - Number(b[i]);
  }
  if (!a[4] || !b[4]) return a[4] ? -1 : b[4] ? 1 : 0;
  const [idsA, idsB] = [a[4].split("."), b[4].split(".")];
  for (let i = 0; i < Math.max(idsA.length, idsB.length); i++) {
    if (idsA[i] === undefined) return -1;
    if (idsB[i] === undefined) return 1;
    const cmp = compareIdentifier(idsA[i], idsB[i]);
    if (cmp) return cmp;
  }
  return 0;
}

// Plain "X.Y.Z" only: no pre-release (rc, beta, ...) and no build metadata.
export function isStableRelease(name: string | undefined): boolean {
  return name !== undefined && /^\d+\.\d+\.\d+$/.test(name);
}

// Highest stable tag that is newer than `current`, or null if `current`
// is not itself a stable release or is already up to date.
export function findNewerStableRelease(
  current: string | undefined,
  tagNames: string[],
): string | null {
  if (!isStableRelease(current)) return null;
  let latest = SEMVER_RE.exec(current!)!;
  for (const name of tagNames) {
    if (!isStableRelease(name)) continue;
    const match = SEMVER_RE.exec(name)!;
    if (compareSemver(match, latest) > 0) latest = match;
  }
  return latest[0] === current ? null : latest[0];
}
