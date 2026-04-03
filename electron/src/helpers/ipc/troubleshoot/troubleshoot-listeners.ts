import { ipcMain, dialog } from "electron";
import {
  TROUBLESHOOT_REBOOT_HMI,
  TROUBLESHOOT_RESTART_BACKEND,
  TROUBLESHOOT_EXPORT_LOGS,
} from "./troubleshoot-channels";

import { run } from "../commands";

export function addTroubleshootEventListeners() {
  ipcMain.handle(TROUBLESHOOT_REBOOT_HMI, async () => {
    run("sudo reboot");
  });

  ipcMain.handle(TROUBLESHOOT_RESTART_BACKEND, async () => {
    run("sudo systemctl restart qitech-control-server");
  });

  ipcMain.handle(TROUBLESHOOT_EXPORT_LOGS, async () => {
    const now = new Date();
    const datePart = now.toISOString().split("T")[0]; // YYYY-MM-DD
    const timePart = now.toTimeString().split(" ")[0].replace(/:/g, "-"); // HH-mm-ss
    const fileName = `journal_${datePart}_${timePart}.log`;

    const { canceled, filePath } = await dialog.showSaveDialog({
      title: "Export System Logs",
      defaultPath: fileName,
      filters: [{ name: "Log Files", extensions: ["log"] }],
    });

    if (canceled || !filePath) {
      throw new Error("Export cancelled by user");
    }

    run(`journalctl -xb > "${filePath}"`);
  });
}
