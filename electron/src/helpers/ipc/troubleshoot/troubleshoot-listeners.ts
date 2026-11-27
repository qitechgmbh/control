import { ipcMain, dialog } from "electron";
import { spawn, exec, ChildProcess } from "child_process";
import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_RESTART_BACKEND_INTO_PREOP,
  TROUBLESHOOT_EXPORT_LOGS,
  TROUBLESHOOT_GET_ETHERCAT_ENABLED,
  TROUBLESHOOT_SET_ETHERCAT_ENABLED,
  TROUBLESHOOT_GET_DEMO_MACHINES_ENABLED,
  TROUBLESHOOT_SET_DEMO_MACHINES_ENABLED,
} from "./troubleshoot-channels";
import { getRemovableVolumeRoot } from "../export/removable-media";
import { SaveFileResult } from "../export/export-channels";

import fs from "fs";
import path from "path";

export function addTroubleshootEventListeners() {
  ipcMain.handle(TROUBLESHOOT_REBOOT_HMI, async () => {
    try {
      spawn("sudo", ["reboot"], { shell: true });
      return { success: true };
    } catch (error) {
      console.error("Failed to reboot HMI:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  ipcMain.handle(TROUBLESHOOT_RESTART_BACKEND_INTO_PREOP, async () => {
    try {
      const process = spawn(
        "sudo",
        [
          "bash",
          "-c",
          "systemctl set-environment QITECH_MODE=preop && systemctl restart qitech-control-server",
        ],
        { shell: false },
      );

      return new Promise<{ success: boolean; error?: string }>((resolve) => {
        process.on("close", (code) => {
          if (code === 0) resolve({ success: true });
          else
            resolve({
              success: false,
              error: `Process exited with code ${code}`,
            });
        });
        process.on("error", (error) =>
          resolve({ success: false, error: error.message }),
        );
      });
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  ipcMain.handle(TROUBLESHOOT_RESTART_BACKEND, async () => {
    try {
      const process = spawn(
        "sudo",
        [
          "bash",
          "-c",
          "systemctl unset-environment QITECH_MODE && systemctl restart qitech-control-server",
        ],
        { shell: false },
      );

      return new Promise<{ success: boolean; error?: string }>((resolve) => {
        process.on("close", (code) => {
          if (code === 0) resolve({ success: true });
          else
            resolve({
              success: false,
              error: `Process exited with code ${code}`,
            });
        });
        process.on("error", (error) =>
          resolve({ success: false, error: error.message }),
        );
      });
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  // The backend treats anything but ETHERCAT_ENABLED=false as enabled
  ipcMain.handle(TROUBLESHOOT_GET_ETHERCAT_ENABLED, async () => {
    return new Promise<{ success: boolean; enabled?: boolean; error?: string }>(
      (resolve) => {
        exec("systemctl show-environment", (error, stdout) => {
          if (error) {
            resolve({ success: false, error: error.message });
            return;
          }
          const disabled = stdout
            .split("\n")
            .some((line) => line.trim() === "ETHERCAT_ENABLED=false");
          resolve({ success: true, enabled: !disabled });
        });
      },
    );
  });

  ipcMain.handle(
    TROUBLESHOOT_SET_ETHERCAT_ENABLED,
    async (_event, enabled: boolean) => {
      try {
        const value = enabled ? "true" : "false";
        const process = spawn(
          "sudo",
          [
            "bash",
            "-c",
            `systemctl set-environment ETHERCAT_ENABLED=${value} && systemctl restart qitech-control-server`,
          ],
          { shell: false },
        );

        return new Promise<{ success: boolean; error?: string }>((resolve) => {
          process.on("close", (code) => {
            if (code === 0) resolve({ success: true });
            else
              resolve({
                success: false,
                error: `Process exited with code ${code}`,
              });
          });
          process.on("error", (error) =>
            resolve({ success: false, error: error.message }),
          );
        });
      } catch (error) {
        return {
          success: false,
          error: error instanceof Error ? error.message : String(error),
        };
      }
    },
  );

  // The backend adds the demo line only with DEMO_MACHINES=true
  ipcMain.handle(TROUBLESHOOT_GET_DEMO_MACHINES_ENABLED, async () => {
    return new Promise<{ success: boolean; enabled?: boolean; error?: string }>(
      (resolve) => {
        exec("systemctl show-environment", (error, stdout) => {
          if (error) {
            resolve({ success: false, error: error.message });
            return;
          }
          const enabled = stdout
            .split("\n")
            .some((line) => line.trim() === "DEMO_MACHINES=true");
          resolve({ success: true, enabled });
        });
      },
    );
  });

  ipcMain.handle(
    TROUBLESHOOT_SET_DEMO_MACHINES_ENABLED,
    async (_event, enabled: boolean) => {
      // Turning the demo off keeps its data; the Database page can delete it.
      const command = enabled
        ? "systemctl set-environment DEMO_MACHINES=true && systemctl restart qitech-control-server"
        : "systemctl unset-environment DEMO_MACHINES && systemctl restart qitech-control-server";
      return new Promise<{ success: boolean; error?: string }>((resolve) => {
        const process = spawn("sudo", ["bash", "-c", command], {
          shell: false,
        });
        process.on("close", (code) => {
          if (code === 0) resolve({ success: true });
          else
            resolve({
              success: false,
              error: `Process exited with code ${code}`,
            });
        });
        process.on("error", (error) =>
          resolve({ success: false, error: error.message }),
        );
      });
    },
  );

  ipcMain.handle(TROUBLESHOOT_EXPORT_LOGS, async () => {
    try {
      const now = new Date();
      const datePart = now.toISOString().split("T")[0]; // YYYY-MM-DD
      const timePart = now.toTimeString().split(" ")[0].replace(/:/g, "-"); // HH-mm-ss
      const fileName = `journal_${datePart}_${timePart}.log`;

      // 1. Open a Save Dialog so the user can choose the location
      const { canceled, filePath } = await dialog.showSaveDialog({
        title: "Export System Logs",
        defaultPath: fileName,
        filters: [{ name: "Log Files", extensions: ["log"] }],
      });

      if (canceled || !filePath) {
        return { success: false, error: "Export cancelled by user" };
      }

      // Wrap the exec in a typed Promise to match the backend restart pattern
      return await new Promise<SaveFileResult>((resolve) => {
        // Note: journalctl -xb usually requires sudo or journal group membership
        exec(`journalctl -xb > "${filePath}"`, (error, stdout, stderr) => {
          if (error) {
            console.error("Exec error:", error);
            resolve({
              success: false,
              error: error instanceof Error ? error.message : String(error),
            });
            return;
          }

          const mountPath = getRemovableVolumeRoot(filePath);
          resolve({
            success: true,
            filePath,
            isRemovable: mountPath !== null,
            mountPath: mountPath ?? undefined,
          });
        });
      });
    } catch (error) {
      console.error("Failed to export logs: ", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });
}
