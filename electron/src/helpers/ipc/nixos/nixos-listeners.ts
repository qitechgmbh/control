import { ipcMain } from "electron";
import {
  NIXOS_LIST_GENERATIONS,
  NIXOS_SET_GENERATION,
  NIXOS_DELETE_GENERATION,
  NIXOS_DELETE_ALL_OLD_GENERATIONS,
  NIXOS_IS_AVAILABLE,
} from "./nixos-channels";
import { NixOSGeneration } from "./nixos-context";
import { run } from "../commands";

export function addNixOSEventListeners() {
  ipcMain.handle(NIXOS_IS_AVAILABLE, async () => {
    try {
      await run("nix --version");
      return true;
    } catch (error) {
      console.warn("NixOS is not available:", error);
      return false;
    }
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
      console.error("Failed to delete all NixOS generations:", error);
      throw error;
    }
  });
}

async function listNixOSGenerations(): Promise<NixOSGeneration[]> {
  const { stdout } = await run("sudo nixos-rebuild list-generations");
  return parseNixOSGenerations(stdout);
}

async function setNixOSGeneration(generationId: string): Promise<void> {
  await run(
    `sudo nix-env --switch-generation ${generationId} -p /nix/var/nix/profiles/system`,
  );
  await run(
    "sudo /nix/var/nix/profiles/system/bin/switch-to-configuration boot",
  );
  await run("sudo reboot");
}

async function deleteNixOSGeneration(generationId: string): Promise<void> {
  await run(
    `sudo nix-env --delete-generations ${generationId} -p /nix/var/nix/profiles/system`,
  );
  await run(`sudo nix store gc`);
  await run(
    `sudo /nix/var/nix/profiles/system/bin/switch-to-configuration boot`,
  );
}

async function deleteAllOldNixOSGeneration(): Promise<void> {
  const generations = await listNixOSGenerations();

  // We will always keep the latest three generation.
  generations.splice(0, 3);

  for (const generation of generations) {
      await deleteNixOSGeneration(generation.id);
  }

  await run(`nix-collect-garbage --delete-old`);
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
