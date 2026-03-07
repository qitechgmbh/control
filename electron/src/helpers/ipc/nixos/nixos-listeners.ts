import { ipcMain } from "electron";
import { spawn } from "child_process";
import {
  NIXOS_LIST_GENERATIONS,
  NIXOS_SET_GENERATION,
  NIXOS_DELETE_GENERATION,
  NIXOS_DELETE_ALL_OLD_GENERATIONS,
  NIXOS_IS_AVAILABLE,
} from "./nixos-channels";
import { NixOSGeneration } from "./nixos-context";

export function addNixOSEventListeners() {
  ipcMain.handle(NIXOS_IS_AVAILABLE, () => {
    return new Promise((resolve) => {
      const process = spawn("nix", ["--version"]);
      process.on("exit", (code) => resolve(code === 0));
      process.on("error", (error) => {
        console.warn("NixOS is not available:", error);
        resolve(false);
      });
    });
  });

  ipcMain.handle(NIXOS_LIST_GENERATIONS, async () => {
    try {
      return await listNixOSGenerations();
    } catch (error) {
      console.error("Failed to list NixOS generations:", error);
      throw error;
    }
  });

  ipcMain.handle(NIXOS_SET_GENERATION, async (_, generationId: string) => {
    try {
      return await setNixOSGeneration(generationId);
    } catch (error) {
      console.error("Failed to set NixOS generation:", error);
      throw error;
    }
  });

  ipcMain.handle(NIXOS_DELETE_GENERATION, async (_, generationId: string) => {
    try {
      return await deleteNixOSGeneration(generationId);
    } catch (error) {
      console.error("Failed to delete NixOS generation:", error);
      throw error;
    }
  });
  ipcMain.handle(NIXOS_DELETE_ALL_OLD_GENERATIONS, async () => {
    try {
      return await deleteAllOldNixOSGeneration();
    } catch (error) {
      console.error("Failed to delete all  NixOS generations:", error);
      throw error;
    }
  });
}

async function listNixOSGenerations(): Promise<NixOSGeneration[]> {
  return new Promise((resolve, reject) => {
    // List all generations using nixos-rebuild
    const process = spawn("sudo", ["nixos-rebuild", "list-generations"]);

    let stdout = "";
    let stderr = "";

    process.stdout?.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        const generations = parseNixOSGenerations(stdout);
        resolve(generations);
      } else {
        reject(new Error(stderr || `Process exited with code ${code}`));
      }
    });

    process.on("error", reject);
  });
}

async function setNixOSGeneration(generationId: string): Promise<void> {
  return new Promise((resolve, reject) => {
    // Switch to the specified generation using nixos-rebuild
    // Set the generation to be used at next boot, then reboot immediately
    const process = spawn("sudo", [
      "sh",
      "-c",
      `nix-env --switch-generation ${generationId} -p /nix/var/nix/profiles/system && /nix/var/nix/profiles/system/bin/switch-to-configuration boot && reboot`,
    ]);

    let stderr = "";
    let stdout = "";

    process.stdout?.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(
          new Error(stderr || stdout || `Process exited with code ${code}`),
        );
      }
    });

    process.on("error", reject);
  });
}

async function deleteNixOSGeneration(generationId: string): Promise<void> {
  return new Promise((resolve, reject) => {
    // Delete the specified generation using nix-env and update bootloader
    // This is the proper NixOS way to delete specific generations
    const process = spawn("sudo", [
      "sh",
      "-c",
      `nix-env --delete-generations ${generationId} -p /nix/var/nix/profiles/system && nix store gc && /nix/var/nix/profiles/system/bin/switch-to-configuration boot`,
    ]);

    let stderr = "";
    let stdout = "";

    process.stdout?.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(
          new Error(stderr || stdout || `Process exited with code ${code}`),
        );
      }
    });

    process.on("error", reject);
  });
}

async function deleteAllOldNixOSGeneration(): Promise<void> {
  return new Promise((resolve, reject) => {
    const process = spawn("sudo", [
      "sh",
      "-c",
      `nix-collect-garbage --delete-old`,
    ]);

    let stderr = "";
    let stdout = "";

    process.stdout?.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(
          new Error(stderr || stdout || `Process exited with code ${code}`),
        );
      }
    });

    process.on("error", reject);
  });
}

function parseNixOSGenerations(output: string): NixOSGeneration[] {
  const lines = output.trim().split("\n");
  const generations: NixOSGeneration[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Skip the first line (header) and empty lines
    if (i === 0 || !line.trim()) {
      continue;
    }

    // Also skip any line that looks like a header
    if (
      line.includes("Generation") &&
      (line.includes("Build date") ||
        line.includes("NixOS version") ||
        line.includes("Configuration"))
    ) {
      continue;
    }

    // Parse lines from nixos-rebuild list-generations:
    // Format: "ID [current] DATE TIME NIXOS_VERSION [CONFIGURATION] [REVISION] [SPECIALISATION] KERNEL"
    // Example: "62 current 2025-06-10 08:51:35 fix.33_c744e1481fdc0bf25821bd0ee0ae8278f155                            6.14.8"

    // Split the line into parts and extract information
    const parts = line.trim().split(/\s+/);
    if (parts.length < 4) continue;

    const id = parts[0];

    // Skip if first part is not a number (could be header remnant)
    if (!/^\d+$/.test(id)) continue;

    let currentIndex = 1;
    let isCurrent = false;

    // Check if "current" is present
    if (parts[1] === "current") {
      isCurrent = true;
      currentIndex = 2;
    }

    // Extract date and time (should be at currentIndex and currentIndex+1)
    const date = parts[currentIndex];
    const time = parts[currentIndex + 1];
    const dateTime = `${date} ${time}`;

    // The next part should be the NixOS version/name
    const nixosVersion = parts[currentIndex + 2] || `Generation ${id}`;

    // The last part (if it looks like a kernel version) is the kernel
    const lastPart = parts[parts.length - 1];
    const kernelVersion =
      lastPart && /^\d+\.\d+(\.\d+)?/.test(lastPart) ? lastPart : undefined;

    generations.push({
      id,
      name: nixosVersion,
      version: nixosVersion,
      current: isCurrent,
      date: dateTime,
      path: `/nix/var/nix/profiles/system-${id}-link`,
      kernelVersion,
    });
  }

  // Sort by generation ID (numeric) in descending order to show newest first
  return generations.sort((a, b) => parseInt(b.id) - parseInt(a.id));
}
