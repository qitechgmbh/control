import { ipcMain } from "electron";

// Environment information
const environmentInfo = {
  deploymentType: process.env.QITECH_DEPLOYMENT_TYPE || 'standard',
  buildEnv: process.env.QITECH_BUILD_ENV || 'standard'
};

// Log environment information to console
console.log("Deployment type:", environmentInfo.deploymentType);
console.log("Build env:", environmentInfo.buildEnv);

export function addEnvironmentEventListeners() {
  ipcMain.handle('environment-get-info', async () => {
    return environmentInfo;
  });
}
