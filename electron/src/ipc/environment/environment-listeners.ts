import { ipcMain } from "electron";
import { ENVIRONMENT_INFO } from "./environment-channels";

// Environment information
export const environmentInfo = {
  qitechOs: process.env.QITECH_OS === "true",
  qitechOsGitTimestamp: process.env.QITECH_OS_GIT_TIMESTAMP
    ? isNaN(Date.parse(process.env.QITECH_OS_GIT_TIMESTAMP))
      ? undefined
      : new Date(process.env.QITECH_OS_GIT_TIMESTAMP)
    : undefined,
  qitechOsGitCommit: process.env.QITECH_OS_GIT_COMMIT,
  qitechOsGitAbbreviation: process.env.QITECH_OS_GIT_ABBREVIATION,
  qitechOsGitUrl: process.env.QITECH_OS_GIT_URL,
};

export function addEnvironmentEventListeners() {
  ipcMain.handle(ENVIRONMENT_INFO, async () => {
    return environmentInfo;
  });
}
