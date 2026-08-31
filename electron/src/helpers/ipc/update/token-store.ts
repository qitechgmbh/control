import { safeStorage } from "electron";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";

function getHomeDir(): string {
  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  return qitechControlEnv === "control-os"
    ? "/home/qitech"
    : (process.env.HOME ?? "/tmp");
}

function getTokenPath(): string {
  const configDir = path.join(getHomeDir(), ".config", "qitech-control");
  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true });
  }
  return path.join(configDir, "github-token.enc");
}

export function saveToken(token: string): void {
  const trimmed = token.trim();
  if (!trimmed) {
    clearToken();
    return;
  }
  const tokenPath = getTokenPath();
  if (safeStorage.isEncryptionAvailable()) {
    const encrypted = safeStorage.encryptString(trimmed);
    writeFileSync(tokenPath, encrypted, { mode: 0o600 });
  } else {
    // Fallback: store as plain UTF-8 (still restricted to owner only)
    writeFileSync(tokenPath, trimmed, { encoding: "utf8", mode: 0o600 });
  }
}

export function readStoredToken(): string | null {
  const tokenPath = getTokenPath();
  if (!existsSync(tokenPath)) return null;
  try {
    const data = readFileSync(tokenPath);
    if (safeStorage.isEncryptionAvailable()) {
      return safeStorage.decryptString(data);
    }
    return data.toString("utf8");
  } catch {
    return null;
  }
}

export function hasToken(): boolean {
  return readStoredToken() !== null;
}

export function clearToken(): void {
  const tokenPath = getTokenPath();
  if (existsSync(tokenPath)) {
    unlinkSync(tokenPath);
  }
}

/**
 * Returns git `-c` args that inject a HTTPS Authorization header for GitHub.
 * Using `-c http.extraheader=...` keeps the credential out of the clone's
 * on-disk git config (unlike embedding the token in the URL).
 */
export function gitAuthArgs(): string[] {
  const token = readStoredToken();
  if (!token) return [];
  const encoded = Buffer.from(`x-access-token:${token}`).toString("base64");
  return ["-c", `http.extraheader=AUTHORIZATION: basic ${encoded}`];
}

/**
 * Returns the most likely default path for an open-file dialog pointing at
 * removable/USB drives, based on the current platform.
 */
export function usbDefaultPath(): string {
  if (process.platform === "darwin") return "/Volumes";
  if (existsSync("/run/media")) return "/run/media";
  if (existsSync("/media")) return "/media";
  return getHomeDir();
}
