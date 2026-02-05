import { ipcMain } from "electron";
import { ChildProcess, spawn } from "child_process";
import {
    TROUBLESHOOT_GET_LOG_LINES,
    TROUBLESHOOT_ON_LOG_LINE,
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
} from "./troubleshoot-channels";

export function addTroubleshootEventListeners() {
  let numLogLines = 0;

  try {
    const logListeningProcess: ChildProcess = spawn("journalctl", ["--boot", "--lines=all", "--follow", "--unit", "bluetooth.service"]);

    logListeningProcess.stdout?.on("data", (data: Uint8Array) => { // Single Line
      for (let i = 0; i < data.length; i++) {
        if (data[i] === 10) {
          numLogLines++;
        }
      }

      ipcMain.emit(TROUBLESHOOT_ON_LOG_LINE, numLogLines);
    });

    logListeningProcess.on("error", error => {
      console.error("Failed to subscribe to systemd log:", error);
    });

    logListeningProcess.on("exit", code => {
      console.error("Process exited unexpectedly:", code);
    });
  } catch (error) {
      console.error("Failed to subscribe to systemd log:", error);
  }

  ipcMain.handle(TROUBLESHOOT_REBOOT_HMI, async () => {
    try {
      spawn("sudo", ["reboot"], { shell: true });
    } catch (e) {
      console.error("Failed to reboot HMI:", e);
      throw e;
    }
  });

  ipcMain.handle(TROUBLESHOOT_RESTART_BACKEND, () => {
    try {
      const process = spawn(
        "sudo",
        ["systemctl", "restart", "qitech-control-server"],
        { shell: true },
      );

      return new Promise<void>((resolve, reject) => {
        process.on("close", (code) => {
          if (code === 0) {
            resolve();
          } else {
            reject(new Error(`Process exited with code ${code}`));
          }
        });

        process.on("error", (error) => {
          reject(error);
        });
      });
    } catch (e) {
      console.error("Failed to restart backend:", e);
      throw e;
    }
  });

  ipcMain.handle(TROUBLESHOOT_GET_LOG_LINES, (_event, start: number, count: number) => {
      const first = Math.max(0, numLogLines - start);
      const lines: string[] = [];

      return new Promise((resolve, reject) => {
        try {
          const process = spawn("journalctl", ["--boot", `--lines=${first}`, "--unit", "bluetooth.service"]);

          process.stdout?.on("data", (data: Uint8Array) => {
            if (count <= 0) {
              resolve(lines);
              process.kill();
            } else {
              const str = data.toString().trim();

              const newLines = str.split("\n").splice(0, count);
              count -= newLines.length;
              lines.push(...newLines);
            }
          });

          process.on("error", error => {
            console.error("Failed to read systemd log:", error);
            reject(error);
          });

          process.on("exit", () => {
            resolve(lines);
          });

        } catch (error) {
            console.error("Failed to read systemd log:", error);
            reject(error);
        }
      });
  });
}
