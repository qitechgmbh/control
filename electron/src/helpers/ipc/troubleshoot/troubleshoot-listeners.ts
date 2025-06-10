import { ipcMain } from "electron";
import { spawn, ChildProcess } from "child_process";
import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_START_LOG_STREAM,
  TROUBLESHOOT_STOP_LOG_STREAM,
  TROUBLESHOOT_LOG_DATA,
} from "./troubleshoot-channels";

let logStreamProcess: ChildProcess | null = null;

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

  ipcMain.handle(TROUBLESHOOT_RESTART_BACKEND, async () => {
    try {
      const process = spawn(
        "sudo",
        ["systemctl", "restart", "qitech-control-server"],
        { shell: true },
      );

      return new Promise<{ success: boolean; error?: string }>((resolve) => {
        process.on("close", (code) => {
          if (code === 0) {
            resolve({ success: true });
          } else {
            resolve({
              success: false,
              error: `Process exited with code ${code}`,
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
    } catch (error) {
      console.error("Failed to restart backend:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  ipcMain.handle(TROUBLESHOOT_START_LOG_STREAM, async (event) => {
    try {
      // Stop any existing log stream
      if (logStreamProcess) {
        logStreamProcess.kill();
        logStreamProcess = null;
      }

      logStreamProcess = spawn(
        "journalctl",
        ["-u", "qitech-control-server", "-n", "10000", "-f"],
        {
          shell: true,
        },
      );

      logStreamProcess.stdout?.on("data", (data) => {
        const logLine = data.toString();
        event.sender.send(TROUBLESHOOT_LOG_DATA, logLine);
      });

      logStreamProcess.stderr?.on("data", (data) => {
        const logLine = data.toString();
        event.sender.send(TROUBLESHOOT_LOG_DATA, logLine);
      });

      logStreamProcess.on("close", (code) => {
        console.log(`Log stream process exited with code ${code}`);
        logStreamProcess = null;
      });

      logStreamProcess.on("error", (error) => {
        console.error("Log stream process error:", error);
        logStreamProcess = null;
      });

      return { success: true };
    } catch (error) {
      console.error("Failed to start log stream:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });

  ipcMain.handle(TROUBLESHOOT_STOP_LOG_STREAM, async () => {
    try {
      if (logStreamProcess) {
        logStreamProcess.kill();
        logStreamProcess = null;
        console.log("Log stream stopped");
      }
      return { success: true };
    } catch (error) {
      console.error("Failed to stop log stream:", error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  });
}
