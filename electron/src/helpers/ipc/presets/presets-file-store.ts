import {
  closeSync,
  existsSync,
  fsyncSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  writeSync,
} from "node:fs";
import path from "node:path";
import type { PresetsReadResult, PresetsWriteResult } from "./presets-channels";

// Presets used to live in Chromium's localStorage, which Chromium silently
// wipes when its LevelDB gets corrupted (e.g. power loss or `reboot` during a
// write). They now live in a plain JSON file that is written atomically
// (temp file + fsync + rename) and keeps the previous version as a backup.

const FILE_NAME = "presets.json";
const BACKUP_SUFFIX = ".bak";
const TMP_SUFFIX = ".tmp";

function fsyncDir(dir: string) {
  try {
    const fd = openSync(dir, "r");
    try {
      fsyncSync(fd);
    } finally {
      closeSync(fd);
    }
  } catch {
    // Not supported on every platform/filesystem; the rename is still atomic.
  }
}

function writeFileDurable(filePath: string, content: string) {
  const fd = openSync(filePath, "w", 0o600);
  try {
    writeSync(fd, content);
    fsyncSync(fd);
  } finally {
    closeSync(fd);
  }
}

/** Returns the content if the file exists and is valid JSON, otherwise null. */
function readValidJson(filePath: string): string | null {
  if (!existsSync(filePath)) {
    return null;
  }
  try {
    const content = readFileSync(filePath, "utf8");
    JSON.parse(content);
    return content;
  } catch (e) {
    console.error(`Preset file ${filePath} is unreadable:`, e);
    return null;
  }
}

export class PresetsFileStore {
  readonly filePath: string;
  readonly backupPath: string;
  readonly tmpPath: string;

  constructor(readonly dir: string) {
    this.filePath = path.join(dir, FILE_NAME);
    this.backupPath = this.filePath + BACKUP_SUFFIX;
    this.tmpPath = this.filePath + TMP_SUFFIX;
  }

  read(): PresetsReadResult {
    const mainExists = existsSync(this.filePath);
    const backupExists = existsSync(this.backupPath);

    if (!mainExists && !backupExists) {
      return { status: "missing" };
    }

    const main = readValidJson(this.filePath);
    if (main !== null) {
      return { status: "ok", content: main };
    }

    const backup = readValidJson(this.backupPath);
    if (backup !== null) {
      console.warn("Preset file is corrupt, falling back to backup");
      return { status: "ok", content: backup };
    }

    // Both files exist but are unreadable. Move them aside so the user can
    // keep working and the data can still be recovered manually.
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    for (const p of [this.filePath, this.backupPath]) {
      if (existsSync(p)) {
        try {
          renameSync(p, `${p}.corrupt-${stamp}`);
        } catch (e) {
          return { status: "error", error: String(e) };
        }
      }
    }
    return { status: "missing" };
  }

  write(content: string): PresetsWriteResult {
    try {
      // Never persist something we could not read back.
      JSON.parse(content);

      mkdirSync(this.dir, { recursive: true });
      writeFileDurable(this.tmpPath, content);

      // Keep the previous good version as backup. Both renames are atomic, so
      // at any point in time at least one complete file exists.
      if (readValidJson(this.filePath) !== null) {
        renameSync(this.filePath, this.backupPath);
      }
      renameSync(this.tmpPath, this.filePath);
      fsyncDir(this.dir);

      return { status: "ok" };
    } catch (e) {
      console.error("Failed to write presets:", e);
      return { status: "error", error: String(e) };
    }
  }
}
