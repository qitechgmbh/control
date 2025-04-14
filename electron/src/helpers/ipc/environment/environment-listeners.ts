import { ipcMain } from "electron";

// Environment information
const environmentInfo = {
  buildEnv: process.env.QITECH_BUILD_ENV || "standard",
};

// Log environment information to console
console.log("Build env:", environmentInfo.buildEnv);

export function addEnvironmentEventListeners() {
  ipcMain.handle("environment-get-info", async () => {
    return environmentInfo;
  });
}
