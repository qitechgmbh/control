import { ipcMain } from "electron";
import { spawn } from "child_process";
import {
  NIXOS_LIST_GENERATIONS,
  NIXOS_SET_GENERATION,
  NIXOS_DELETE_GENERATION,
} from "./nixos-channels";
import { NixOSGeneration } from "./nixos-context";

export function addNixOSEventListeners() {
  ipcMain.handle(NIXOS_LIST_GENERATIONS, async () => {
    try {
      return await listNixOSGenerations();
    } catch (error) {
      console.error("Failed to list NixOS generations:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        generations: [],
      };
    }
  });

  ipcMain.handle(NIXOS_SET_GENERATION, async (_, generationId: string) => {
    try {
      const result = await setNixOSGeneration(generationId);
      return result;
    } catch (error) {
      console.error("Failed to set NixOS generation:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  ipcMain.handle(NIXOS_DELETE_GENERATION, async (_, generationId: string) => {
    try {
      const result = await deleteNixOSGeneration(generationId);
      return result;
    } catch (error) {
      console.error("Failed to delete NixOS generation:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });
}

async function listNixOSGenerations(): Promise<{
  success: boolean;
  generations: NixOSGeneration[];
  error?: string;
}> {
  return new Promise((resolve) => {
    // List all generations using nix-env
    const process = spawn(
      "sudo",
      ["nix-env", "--list-generations", "-p", "/nix/var/nix/profiles/system"],
      {
        shell: true,
      },
    );

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
        resolve({ success: true, generations });
      } else {
        resolve({
          success: false,
          generations: [],
          error: stderr || `Process exited with code ${code}`,
        });
      }
    });

    process.on("error", (error) => {
      resolve({
        success: false,
        generations: [],
        error: error instanceof Error ? error.message : String(error),
      });
    });
  });
}

async function setNixOSGeneration(generationId: string): Promise<{
  success: boolean;
  error?: string;
}> {
  return new Promise((resolve) => {
    // Switch to the specified generation
    const process = spawn(
      "sudo",
      [
        "nix-env",
        "--switch-generation",
        generationId,
        "-p",
        "/nix/var/nix/profiles/system",
      ],
      {
        shell: true,
      },
    );

    let stderr = "";

    process.stderr?.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve({ success: true });
      } else {
        resolve({
          success: false,
          error: stderr || `Process exited with code ${code}`,
        });
      }
    });

    process.on("error", (error) => {
      resolve({
        success: false,
        error: error instanceof Error ? error.message : String(error),
      });
    });
  });
}

async function deleteNixOSGeneration(generationId: string): Promise<{
  success: boolean;
  error?: string;
}> {
  return new Promise((resolve) => {
    // Delete the specified generation and update bootloader
    const process = spawn(
      "sudo",
      [
        "sh",
        "-c",
        `nix-env --delete-generations ${generationId} -p /nix/var/nix/profiles/system && /nix/var/nix/profiles/system/bin/switch-to-configuration boot`,
      ],
      {
        shell: true,
      },
    );

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
        resolve({ success: true });
      } else {
        resolve({
          success: false,
          error: stderr || stdout || `Process exited with code ${code}`,
        });
      }
    });

    process.on("error", (error) => {
      resolve({
        success: false,
        error: error instanceof Error ? error.message : String(error),
      });
    });
  });
}

function parseNixOSGenerations(output: string): NixOSGeneration[] {
  const lines = output.trim().split("\n");
  const generations: NixOSGeneration[] = [];

  for (const line of lines) {
    // Parse lines like: "   1   2024-01-15 14:30:20   (current)"
    const match = line.match(
      /^\s*(\d+)\s+(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\s*(\(current\))?/,
    );

    if (match) {
      const [, id, date, currentMarker] = match;
      generations.push({
        id,
        name: `Generation ${id}`,
        version: id,
        current: !!currentMarker,
        date,
        path: `/nix/var/nix/profiles/system-${id}-link`,
      });
    }
  }

  return generations.reverse(); // Show newest first
}
