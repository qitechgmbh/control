import { ipcMain } from "electron";
import { UPDATE_EXECUTE, UPDATE_LOG } from "./update-channels";
import { spawn } from "child_process";

type UpdateExecuteListenerParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

export function addUpdateEventListeners() {
  ipcMain.handle(
    UPDATE_EXECUTE,
    (event, params: UpdateExecuteListenerParams) => {
      const {
        githubRepoOwner,
        githubRepoName,
        githubToken,
        tag,
        branch,
        commit,
      } = params;

      // Implement your update logic here
      console.log("Update parameters:", {
        githubRepoOwner,
        githubRepoName,
        githubToken,
        tag,
        branch,
        commit,
      });

      try {
        const cmd1 = "dig";
        const args1 = ["google.com"];

        const process = spawn(cmd1, args1);

        // Stream stdout logs back to renderer
        process.stdout.on("data", (data) => {
          const log = data.toString();
          console.log(log);
          event.sender.send(UPDATE_LOG, log);
        });

        // Stream stderr logs back to renderer
        process.stderr.on("data", (data) => {
          const log = data.toString();
          console.error(log);
          event.sender.send(UPDATE_LOG, log);
        });

        // Handle process completion
        return new Promise((resolve, reject) => {
          process.on("close", (code) => {
            if (code === 0) {
              event.sender.send(UPDATE_LOG, "Command completed successfully");
              resolve({ success: true, error: undefined });
            } else {
              event.sender.send(UPDATE_LOG, `Command failed with code ${code}`);
              reject({ success: false, error: code?.toString() });
            }
          });

          process.on("error", (err) => {
            event.sender.send(UPDATE_LOG, `Command error: ${err.message}`);
            reject({ success: false, error: err.message });
          });
        });
      } catch (error: any) {
        console.error("Error executing command:", error);
        event.sender.send(UPDATE_LOG, `Error: ${error.toString()}`);
        return { success: false, error: error.toString() };
      }
    },
  );
}
