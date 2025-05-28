import { ipcMain } from "electron";
import { UPDATE_END, UPDATE_EXECUTE, UPDATE_LOG } from "./update-channels";
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
          event.sender.send(
            UPDATE_LOG,
            terminalColor("red", terminalError("Home directory not found")),
          );
          return;
        }

        // 1. first make sure the clone path is empty by deleting it if it containsa .git folder
        const repoDir = `${homeDir}/${githubRepoName}`;
        const clearResult = await clearRepoDirectory(
          `${homeDir}/${githubRepoName}`,
          event,
        );
        if (!clearResult.success) {
          event.sender.send(UPDATE_LOG, clearResult.error);
          return;
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
          return;
        }

        // 3. make the nixos-install.sh script executable
        const chmodResult = await runCommand(
          "chmod",
          ["+x", "nixos-install.sh"],
          repoDir,
          event,
        );
        if (!chmodResult.success) {
          event.sender.send(UPDATE_LOG, chmodResult.error);
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
    // Check if the directory exists
    const lsResult = await runCommand("ls", [".git"], repoDir, event);
    if (lsResult.success) {
      // If it exists, delete the directory
      const rmResult = await runCommand("rm", ["-rf", repoDir], repoDir, event);
      if (!rmResult.success) {
        event.sender.send(UPDATE_LOG, rmResult.error);
        return { success: false, error: rmResult.error };
      }
      event.sender.send(
        UPDATE_LOG,
        terminalSuccess(`Deleted existing repository at ${repoDir}`),
      );
    } else {
      event.sender.send(
        UPDATE_LOG,
        terminalError(
          `No existing repository found at ${repoDir}, nothing to delete`,
        ),
      );
    }
    return { success: true };
  } catch (error: any) {
    event.sender.send(UPDATE_LOG, terminalError(`Error: ${error.toString()}`));
    return { success: false, error: error.toString() };
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
    return {
      success: false,
      error: terminalError("Failed to clone repository"),
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

    const process = spawn(cmd, args, {
      cwd: workingDir,
    });

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
          reject({
            success: false,
            error: terminalError(code?.toString() ?? "NO_CODE"),
          });
        }
      });

      process.on("error", (err) => {
        event.sender.send(
          UPDATE_LOG,
          terminalError(`Command error: ${err.message}`),
        );
        reject({ success: false, error: err.message });
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
