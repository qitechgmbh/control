import { ipcMain } from "electron";
import { UPDATE_END, UPDATE_EXECUTE, UPDATE_LOG } from "./update-channels";
import { spawn } from "child_process";
import { existsSync } from "fs";

type UpdateExecuteListenerParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

export function addUpdateEventListeners() {
  // Remove any existing handlers to prevent duplicates
  ipcMain.removeHandler(UPDATE_EXECUTE);

  ipcMain.handle(
    UPDATE_EXECUTE,
    async (event, params: UpdateExecuteListenerParams) => {
      update(event, params)
        .then(() => {
          event.sender.send(
            UPDATE_END,
            terminalSuccess("Update completed successfully!"),
          );
        })
        .catch((error) => {
          event.sender.send(
            UPDATE_END,
            terminalError(`Update failed: ${error.message}`),
          );
        });
    },
  );
}

async function update(
  event: Electron.IpcMainInvokeEvent,
  params: UpdateExecuteListenerParams,
): Promise<void> {
  return new Promise((resolve, reject) => {
    (async () => {
      try {
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

        const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
        const homeDir =
          qitechControlEnv === "control-os" ? "/home/qitech" : process.env.HOME;
        if (!homeDir) {
          const errorMsg = "Home directory not found";
          event.sender.send(UPDATE_LOG, terminalError(errorMsg));
          reject(new Error(errorMsg));
          return;
        }

        // 1. first make sure the clone path is empty by deleting it if it exists
        const repoDir = `${homeDir}/${githubRepoName}`;
        event.sender.send(
          UPDATE_LOG,
          terminalInfo(`Preparing update directory: ${repoDir}`),
        );

        const clearResult = await clearRepoDirectory(repoDir, event);
        if (!clearResult.success) {
          // Even if clearing fails, we should continue - the git clone might overwrite or fail gracefully
          event.sender.send(
            UPDATE_LOG,
            terminalInfo(
              `Warning: Could not fully clear directory, continuing with clone attempt`,
            ),
          );
        }

        // 2. clone the repository
        const cloneResult = await cloneRepository(
          {
            githubRepoOwner,
            githubRepoName,
            githubToken,
            tag,
            branch,
            commit,
          },
          event,
        );
        if (!cloneResult.success) {
          event.sender.send(UPDATE_LOG, cloneResult.error);
          reject(new Error(cloneResult.error || "Failed to clone repository"));
          return;
        }

        // 3. make the nixos-install.sh script executable
        const scriptPath = `${repoDir}/nixos-install.sh`;

        // Check if the script exists before trying to make it executable
        if (!existsSync(scriptPath)) {
          const errorMsg = `nixos-install.sh script not found at ${scriptPath}`;
          event.sender.send(UPDATE_LOG, terminalError(errorMsg));
          reject(new Error(errorMsg));
          return;
        }

        const chmodResult = await runCommand(
          "chmod",
          ["+x", "nixos-install.sh"],
          repoDir,
          event,
        );
        if (!chmodResult.success) {
          event.sender.send(UPDATE_LOG, chmodResult.error);
          reject(
            new Error(chmodResult.error || "Failed to make script executable"),
          );
          return;
        }

        // 4. run the nixos-install.sh script
        const installResult = await runCommand(
          "./nixos-install.sh",
          [],
          repoDir,
          event,
        );
        if (!installResult.success) {
          event.sender.send(UPDATE_LOG, installResult.error);
          reject(
            new Error(installResult.error || "Failed to run nixos-install.sh"),
          );
          return;
        }

        resolve();
      } catch (error: any) {
        reject(error);
      }
    })();
  });
}

type CloneRepositoryParams = {
  githubRepoOwner: string;
  githubRepoName: string;
  githubToken?: string;
  tag?: string;
  branch?: string;
  commit?: string;
};

async function clearRepoDirectory(
  repoDir: string,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  try {
    // Use Node.js existsSync to check if directory exists (more reliable than shell commands)
    if (!existsSync(repoDir)) {
      event.sender.send(
        UPDATE_LOG,
        terminalInfo(`Directory ${repoDir} does not exist, nothing to delete`),
      );
      return { success: true };
    }

    // If directory exists, delete it regardless of whether it's a git repo or not
    // This ensures we always start with a clean slate
    event.sender.send(
      UPDATE_LOG,
      terminalInfo(`Cleaning directory ${repoDir}`),
    );

    const rmResult = await runCommand("rm", ["-rf", repoDir], "/", event);
    if (!rmResult.success) {
      // If rm fails, try to continue anyway - it might be a permission issue
      // but the clone might still work
      event.sender.send(
        UPDATE_LOG,
        terminalInfo(
          `Warning: Could not delete ${repoDir}, continuing anyway: ${rmResult.error}`,
        ),
      );
    } else {
      event.sender.send(
        UPDATE_LOG,
        terminalSuccess(`Successfully cleaned directory ${repoDir}`),
      );
    }

    // Always return success - we don't want to fail the update just because
    // we couldn't clean the directory
    return { success: true };
  } catch (error: any) {
    // Log the error but don't fail the update process
    event.sender.send(
      UPDATE_LOG,
      terminalInfo(
        `Warning: Error cleaning directory ${repoDir}: ${error.toString()}, continuing anyway`,
      ),
    );
    return { success: true };
  }
}

async function cloneRepository(
  params: CloneRepositoryParams,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  const { githubRepoOwner, githubRepoName, githubToken, tag, branch, commit } =
    params;

  const qitechControlEnv = process.env.QITECH_CONTROL_ENV;
  const homeDir = qitechControlEnv ? "/home/qitech" : process.env.HOME;

  if (!homeDir) {
    return { success: false, error: terminalError("Home directory not found") };
  }

  // Construct repository URL
  const repoUrl = githubToken
    ? `https://${githubToken}@github.com/${githubRepoOwner}/${githubRepoName}.git`
    : `https://github.com/${githubRepoOwner}/${githubRepoName}.git`;

  // Determine clone arguments based on whether tag, branch, or commit is specified
  const cloneArgs = ["clone", repoUrl];

  if (tag) {
    // Clone a specific tag
    cloneArgs.push("--branch", tag, "--single-branch");
    event.sender.send(UPDATE_LOG, terminalInfo(`Cloning tag: ${tag}`));
  } else if (branch) {
    // Clone a specific branch
    cloneArgs.push("--branch", branch, "--single-branch");
    event.sender.send(UPDATE_LOG, terminalInfo(`Cloning branch: ${branch}`));
  } else if (commit) {
    // For commit, we need to clone first, then checkout the specific commit
    event.sender.send(
      UPDATE_LOG,
      terminalInfo(`Cloning repository, will checkout commit: ${commit}`),
    );
  } else {
    return {
      success: false,
      error: terminalError("No specific version specified!"),
    };
  }

  const cmd1 = await runCommand("git", cloneArgs, homeDir, event);

  if (!cmd1.success) {
    // If clone fails, try to provide more helpful error information
    event.sender.send(
      UPDATE_LOG,
      terminalError("Git clone failed. This might be due to:"),
    );
    event.sender.send(
      UPDATE_LOG,
      terminalError("- Network connectivity issues"),
    );
    event.sender.send(
      UPDATE_LOG,
      terminalError("- Invalid GitHub token or repository access"),
    );
    event.sender.send(
      UPDATE_LOG,
      terminalError("- Repository doesn't exist or is private"),
    );

    return {
      success: false,
      error: `Failed to clone repository: ${cmd1.error}`,
    };
  }

  // If commit is specified, checkout the specific commit
  if (commit && cmd1.success) {
    const repoDir = `${homeDir}/${githubRepoName}`;
    const cmd2 = await runCommand("git", ["checkout", commit], repoDir, event);

    if (!cmd2.success) {
      return {
        success: false,
        error: terminalError(`Failed to checkout commit: ${commit}`),
      };
    }

    event.sender.send(
      UPDATE_LOG,
      terminalSuccess(`Successfully checked out commit: ${commit}`),
    );
  }
  event.sender.send(
    UPDATE_LOG,
    terminalSuccess("Repository cloned successfully"),
  );
  return { success: true, error: undefined };
}

async function runCommand(
  cmd: string,
  args: string[],
  workingDir: string,
  event: Electron.IpcMainInvokeEvent,
): Promise<{ success: boolean; error?: string }> {
  try {
    const completeCommand = `${cmd} ${args.join(" ")}`;
    const workingDirText = terminalGray(workingDir);
    event.sender.send(
      UPDATE_LOG,
      `🚀 ${workingDirText} ${terminalColor("blue", completeCommand)}`,
    );

    const childProcess = spawn(cmd, args, {
      cwd: workingDir,
    });

    // Stream stdout logs back to renderer
    childProcess.stdout.on("data", (data) => {
      const log = data.toString();
      console.log(log);
      event.sender.send(UPDATE_LOG, log);
    });

    // Stream stderr logs back to renderer
    childProcess.stderr.on("data", (data) => {
      const log = data.toString();
      console.error(log);
      event.sender.send(UPDATE_LOG, log);
    });

    // Handle process completion
    return new Promise((resolve) => {
      childProcess.on("close", (code) => {
        if (code === 0) {
          event.sender.send(
            UPDATE_LOG,
            terminalSuccess("Command completed successfully"),
          );
          resolve({ success: true, error: undefined });
        } else {
          event.sender.send(
            UPDATE_LOG,
            terminalError(`Command failed with code ${code}`),
          );
          resolve({
            success: false,
            error: `Command failed with code ${code?.toString() ?? "NO_CODE"}`,
          });
        }
      });

      childProcess.on("error", (err) => {
        event.sender.send(
          UPDATE_LOG,
          terminalError(`Command error: ${err.message}`),
        );
        resolve({ success: false, error: err.message });
      });
    });
  } catch (error: any) {
    event.sender.send(UPDATE_LOG, terminalError(`Error: ${error.toString()}`));
    return { success: false, error: error.toString() };
  }
}

function terminalColor(
  color: "blue" | "green" | "red" | "cyan" | "gray",
  text: string,
): string {
  const colors = {
    blue: "\x1b[34m",
    green: "\x1b[32m",
    red: "\x1b[31m",
    cyan: "\x1b[36m",
    gray: "\x1b[90m",
  };
  return `${colors[color]}${text}\x1b[0m`;
}

function terminalError(text: string): string {
  return terminalColor("red", "💥 " + text);
}

function terminalSuccess(text: string): string {
  return terminalColor("green", "✅ " + text);
}

function terminalInfo(text: string): string {
  return terminalColor("cyan", "📝 " + text);
}

function terminalGray(text: string): string {
  return terminalColor("gray", text);
}
