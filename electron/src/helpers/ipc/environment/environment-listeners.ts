import { ipcMain } from "electron";
import { ENVIRONMENT_INFO } from "./environment-channels";

// Environment information
const environmentInfo = {
  qitechOs: process.env.QITECH_OS === "true",
  qitechOsGitTimestamp: process.env.QITECH_OS_TIMESTAMP
    ? isNaN(Date.parse(process.env.QITECH_OS_TIMESTAMP))
      ? undefined
      : new Date(process.env.QITECH_OS_TIMESTAMP)
    : undefined,
  qitechOsGitCommit: process.env.QITECH_OS_COMMIT,
  qitechOsGitAbbrevation: process.env.QITECH_OS_ABBREVATION,
  qitechOsGitUrl: process.env.QITECH_OS_URL,
};

export function addEnvironmentEventListeners() {
  ipcMain.handle(ENVIRONMENT_INFO, async () => {
    return environmentInfo;
  });
}
