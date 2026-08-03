import { spawn, exec } from "child_process";

/**
 * Given an absolute file path, returns the root path of the removable/USB
 * volume it lives on, or null if the path doesn't match a recognized
 * removable-media mount convention. Linux-only (the only deployment target
 * for this feature) — always returns null on other platforms.
 */
export function getRemovableVolumeRoot(filePath: string): string | null {
  if (process.platform !== "linux") {
    return null;
  }

  // Most specific patterns first so e.g. /media/user/label/file.log doesn't
  // get truncated to /media/user by a looser pattern.
  const patterns = [
    /^(\/run\/media\/[^/]+\/[^/]+)(\/|$)/, // /run/media/<user>/<label>
    /^(\/media\/[^/]+\/[^/]+)(\/|$)/, // /media/<user>/<label>
    /^(\/media\/[^/]+)(\/|$)/, // /media/<label>
  ];

  for (const pattern of patterns) {
    const match = filePath.match(pattern);
    if (match) return match[1];
  }

  return null;
}

function run(cmd: string): Promise<{ success: boolean; error?: string }> {
  return new Promise((resolve) => {
    exec(cmd, (error) => {
      if (error) {
        resolve({
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      } else {
        resolve({ success: true });
      }
    });
  });
}

function resolveBlockDevice(mountPath: string): Promise<string | null> {
  return new Promise((resolve) => {
    // -T treats the argument as a path *within* a filesystem, not just an
    // exact mountpoint, which is more forgiving than a bare positional arg.
    exec(`findmnt -no SOURCE -T "${mountPath}"`, (error, stdout) => {
      resolve(error ? null : stdout.trim() || null);
    });
  });
}

/**
 * Ejects the removable volume mounted at mountPath. mountPath must be a
 * volume root as returned by getRemovableVolumeRoot(), not an arbitrary
 * file path.
 */
export async function ejectVolume(
  mountPath: string,
): Promise<{ success: boolean; error?: string }> {
  if (process.platform !== "linux") {
    return { success: false, error: "Eject is only supported on Linux" };
  }

  try {
    const device = await resolveBlockDevice(mountPath);

    if (device) {
      const unmountResult = await run(`udisksctl unmount -b "${device}"`);
      if (unmountResult.success) {
        // power-off (spin down / release the USB port) is best-effort: the
        // unmount above already guarantees flushed writes, so a power-off
        // failure does not make physical removal unsafe.
        const powerOffResult = await run(`udisksctl power-off -b "${device}"`);
        if (!powerOffResult.success) {
          console.warn(
            "udisksctl power-off failed (non-fatal):",
            powerOffResult.error,
          );
        }
        return { success: true };
      }
      console.warn(
        "udisksctl unmount failed, falling back to sudo umount:",
        unmountResult.error,
      );
    }

    // Fallback for systems without udisks2, or a device that findmnt/udisks
    // can't resolve (e.g. mounted by root outside the user's session).
    return await new Promise<{ success: boolean; error?: string }>(
      (resolve) => {
        const proc = spawn("sudo", ["umount", mountPath], { shell: false });
        proc.on("close", async (code) => {
          if (code === 0) {
            resolve({ success: true });
          } else {
            // Last-resort: sync twice to flush pending writes
            await run("sync");
            await run("sync");
            resolve({
              success: false,
              error: `umount exited with code ${code}`,
            });
          }
        });
        proc.on("error", (error) =>
          resolve({ success: false, error: error.message }),
        );
      },
    );
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
