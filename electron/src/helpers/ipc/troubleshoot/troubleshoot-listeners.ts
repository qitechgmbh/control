import { ipcMain, dialog } from "electron";
import { spawn, exec, ChildProcess } from "child_process";
import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_RESTART_BACKEND_INTO_PREOP,
  TROUBLESHOOT_EXPORT_LOGS,
} from "./troubleshoot-channels";
import { getRemovableVolumeRoot } from "../export/removable-media";

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

      // 2. Wrap the exec in a typed Promise to match the backend restart pattern
      // This resolves the TS2794 error by explicitly defining the return type
      return await new Promise<{
        success: boolean;
        error?: string;
        filePath?: string;
        isRemovable?: boolean;
        mountPath?: string;
      }>((resolve) => {
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
